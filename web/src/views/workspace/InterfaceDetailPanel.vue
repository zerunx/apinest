<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchInterfaceDetail } from '@/api/interface'
import InterfaceEditorPanels from '@/interface/InterfaceEditorPanels.vue'
import InterfaceStatusRadio from '@/interface/InterfaceStatusRadio.vue'
import InterfaceUrlBar from '@/interface/InterfaceUrlBar.vue'
import { useInterfaceEditorDirty } from '@/composables/useInterfaceEditorDirty'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext, parseApiId } from '@/composables/useWorkspaceContext'
import {
  buildInterfaceSavePayload,
  createEmptyInterfaceEditorForm,
  defaultResponseExample,
  defaultResponseResult,
  interfaceEditorSnapshot,
} from '@/utils/interface-editor-form'
import { collectFolderOptions } from '@/utils/interface-folder-options'
import { fieldTreeFromApi, responseFieldTreeFromApi } from '@/utils/interface-field-tree'
import {
  enrichResponseExample,
  normalizeResponseExampleForSave,
} from '@/utils/response-example-format'
import { emptyParamRow, ensureTrailingEmptyRow, type ParamRow } from '@/utils/interface-params'
import type { InterfaceDetail } from '@/types/workspace'

type DetailMode = 'doc' | 'edit'

const detailCache = new Map<number, InterfaceDetail>()

const { t } = useLocale()
const {
  activeModuleTab,
  activeWorkspaceId,
  activeProjectId,
  apiTree,
  parseFolderId,
  submitUpdateInterface,
  setTabDirty,
} = useWorkspaceContext()

const mode = ref<DetailMode>('edit')
const loading = ref(false)
const saving = ref(false)
const form = reactive(createEmptyInterfaceEditorForm({ method: 'GET', status: 2 }))

const interfaceId = computed(() => {
  const tab = activeModuleTab.value
  if (!tab?.apiId) return null
  return parseApiId(tab.apiId)
})

const detailTabId = computed(() => {
  const id = interfaceId.value
  return id != null ? `api-${id}` : null
})

const readOnly = computed(() => mode.value === 'doc')
const folderOptions = computed(() => collectFolderOptions(apiTree.value, parseFolderId))

const { beginSuppress, captureBaseline } = useInterfaceEditorDirty({
  getSnapshot: () => interfaceEditorSnapshot(form),
  setDirty: (dirty) => {
    if (detailTabId.value) setTabDirty(detailTabId.value, dirty)
  },
  enabled: () => !readOnly.value && !loading.value && !saving.value,
  watchSource: form,
})

function toParamRows(
  items: { name: string; type: string; required: boolean; description: string; example: string }[],
): ParamRow[] {
  const rows = items.map((item) => ({
    name: item.name,
    type: item.type || 'string',
    required: item.required,
    description: item.description,
    example: item.example,
  }))
  return ensureTrailingEmptyRow(rows.length ? rows : [emptyParamRow()])
}

function applyDetail(detail: InterfaceDetail) {
  form.protocol = 'HTTP'
  form.method = detail.method
  form.url = detail.url
  form.folderId = detail.folder_id
  form.name = detail.name
  form.status = detail.status
  form.requestHeaders = toParamRows(detail.request_headers ?? [])
  form.requestBody = {
    format: detail.request_body?.format || 'json',
    data_type: detail.request_body?.data_type || 'Object',
    raw: detail.request_body?.raw ?? '',
    fields: fieldTreeFromApi(detail.request_body?.fields ?? []),
  }
  form.queryParams = toParamRows(detail.query_params ?? [])
  form.responseHeaders = toParamRows(detail.response_headers ?? [])
  form.responseResults = detail.response_results?.length
    ? detail.response_results.map((item) => ({
        ...item,
        fields: responseFieldTreeFromApi(item.fields ?? []),
      }))
    : [defaultResponseResult(t('workspace.interfaceForm.defaultSuccess'))]
  form.responseExamples = detail.response_examples?.length
    ? detail.response_examples.map((item) =>
        normalizeResponseExampleForSave(enrichResponseExample({ ...item })),
      )
    : [defaultResponseExample(t('workspace.interfaceForm.successExample'))]
}

function writeDetailCache(id: number, detail: InterfaceDetail) {
  detailCache.set(id, detail)
}

function cacheFormAsDetail(id: number) {
  const payload = buildInterfaceSavePayload(form)
  const prev = detailCache.get(id)
  writeDetailCache(id, {
    id,
    project_id: prev?.project_id ?? activeProjectId.value ?? 0,
    folder_id: form.folderId ?? prev?.folder_id ?? 0,
    name: payload.name,
    method: form.method,
    url: payload.url ?? '',
    status: form.status,
    created_at: prev?.created_at ?? '',
    updated_at: prev?.updated_at,
    request_headers: payload.request_headers ?? [],
    request_body: payload.request_body ?? form.requestBody,
    query_params: payload.query_params ?? [],
    response_headers: payload.response_headers ?? [],
    response_results: payload.response_results ?? [],
    response_examples: payload.response_examples ?? [],
  })
}

async function loadDetail() {
  if (!interfaceId.value || !activeWorkspaceId.value || !activeProjectId.value) return
  const requestId = interfaceId.value

  const cached = detailCache.get(requestId)
  if (cached) {
    beginSuppress()
    applyDetail(cached)
    await captureBaseline()
  }

  loading.value = true
  beginSuppress()
  try {
    const detail = await fetchInterfaceDetail(
      activeWorkspaceId.value,
      activeProjectId.value,
      requestId,
    )
    if (interfaceId.value !== requestId) return

    const prevCached = detailCache.get(requestId)
    writeDetailCache(requestId, detail)

    // Avoid re-render flash when cache already matches server payload
    if (!prevCached || JSON.stringify(prevCached) !== JSON.stringify(detail)) {
      beginSuppress()
      applyDetail(detail)
    }
  } catch (error) {
    if (interfaceId.value !== requestId) return
    ElMessage.error(error instanceof Error ? error.message : t('workspace.loadApiFailed'))
  } finally {
    if (interfaceId.value === requestId) {
      loading.value = false
      await captureBaseline()
    }
  }
}

watch(interfaceId, (id) => {
  if (id) void loadDetail()
}, { immediate: true })

async function handleSave() {
  if (!interfaceId.value || readOnly.value || saving.value || loading.value) return
  if (!form.name.trim()) {
    ElMessage.warning(t('workspace.nameRequired'))
    return
  }
  if (!form.folderId) {
    ElMessage.warning(t('workspace.folderRequired'))
    return
  }

  saving.value = true
  beginSuppress()
  try {
    await submitUpdateInterface(interfaceId.value, buildInterfaceSavePayload(form))
    cacheFormAsDetail(interfaceId.value)
    await captureBaseline()
    ElMessage.success(t('workspace.updateApiSuccess'))
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.updateApiFailed'))
  } finally {
    saving.value = false
  }
}

defineExpose({
  save: handleSave,
})
</script>

<template>
  <div class="interface-detail interface-editor">
    <div class="interface-detail__header">
      <nav class="interface-detail__subnav">
        <button
          type="button"
          class="interface-detail__subnav-item"
          :class="{ 'interface-detail__subnav-item--active': mode === 'doc' }"
          @click="mode = 'doc'"
        >
          {{ t('workspace.apiSubnav.doc') }}
        </button>
        <button
          type="button"
          class="interface-detail__subnav-item"
          :class="{ 'interface-detail__subnav-item--active': mode === 'edit' }"
          @click="mode = 'edit'"
        >
          {{ t('workspace.apiSubnav.edit') }}
        </button>
      </nav>

      <div v-if="!readOnly" class="interface-detail__toolbar">
        <el-button type="primary" :loading="saving" :disabled="loading" @click="handleSave">
          {{ t('common.save') }}
        </el-button>
      </div>
    </div>

    <div class="interface-detail__scroll">
      <div class="interface-detail__section">
        <div class="interface-detail__row">
          <InterfaceUrlBar
            v-model:protocol="form.protocol"
            v-model:method="form.method"
            v-model:url="form.url"
            :readonly="readOnly"
          />
        </div>

        <div class="interface-detail__row interface-meta-row">
          <el-select
            v-model="form.folderId"
            class="interface-meta-row__folder"
            :placeholder="t('workspace.selectFolder')"
            size="default"
            :disabled="readOnly"
            popper-class="app-action-dropdown"
          >
            <el-option
              v-for="item in folderOptions"
              :key="item.folderId"
              :label="item.label"
              :value="item.folderId"
            />
          </el-select>
          <el-input
            v-model="form.name"
            class="interface-meta-row__name"
            :placeholder="t('workspace.apiNamePlaceholder')"
            :readonly="readOnly"
          />
        </div>

        <div class="interface-detail__status-row">
          <span class="interface-detail__label">{{ t('workspace.columns.status') }}</span>
          <InterfaceStatusRadio v-model="form.status" :readonly="readOnly" />
        </div>
      </div>

      <InterfaceEditorPanels
        :panel-state-key="interfaceId"
        v-model:request-headers="form.requestHeaders"
        v-model:query-params="form.queryParams"
        v-model:request-body="form.requestBody"
        v-model:response-headers="form.responseHeaders"
        v-model:response-results="form.responseResults"
        v-model:response-examples="form.responseExamples"
        :readonly="readOnly"
      />
    </div>
  </div>
</template>

<style scoped>
.interface-detail {
  display: flex;
  flex-direction: column;
  flex: 1;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--color-workspace-content);
}

.interface-detail__subnav {
  display: flex;
  align-items: stretch;
  height: 40px;
  padding: 0 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-workspace-elevated);
}

.interface-detail__subnav-item {
  position: relative;
  padding: 0 16px;
  border: none;
  background: transparent;
  color: var(--color-workspace-tab-text);
  font-size: 14px;
  cursor: pointer;
}

.interface-detail__subnav-item--active {
  color: var(--color-workspace-tab-text-active);
}

.interface-detail__subnav-item--active::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 2px;
  background: var(--color-interface-accent);
}

.interface-detail__toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-workspace-content);
}

.interface-detail__scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  overscroll-behavior: contain;
  padding-bottom: 32px;
  background: var(--color-workspace-content);
}

.interface-detail__section {
  display: flex;
  flex-direction: column;
  gap: 15px;
  padding: 16px var(--color-interface-section-padding-x);
  border-bottom: 1px solid var(--color-border);
}

.interface-detail__row {
  display: flex;
  align-items: center;
  width: 100%;
}

.interface-detail__status-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.interface-detail__label {
  flex-shrink: 0;
  color: var(--color-interface-field-text);
}
</style>
