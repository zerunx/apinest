<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, reactive, ref, watch } from 'vue'
import { CopyDocument } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { fetchInterfaceDetail } from '@/api/interface'
import InterfaceEditorPanels from '@/interface/InterfaceEditorPanels.vue'
import InterfaceSaveButton from '@/interface/InterfaceSaveButton.vue'
import InterfaceStatusRadio from '@/interface/InterfaceStatusRadio.vue'
import InterfaceUrlBar from '@/interface/InterfaceUrlBar.vue'
import { useInterfaceEditorDirty } from '@/composables/useInterfaceEditorDirty'
import { useLocale } from '@/composables/useLocale'
import { interfaceStatusColor, interfaceStatusKey } from '@/constants/interface-status'
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
import {
  inferProtocol,
  isKnownProtocol,
  isProtocolOnlySelect,
  methodTone,
  normalizeProtocol,
  protocolLabel,
  shortMethodLabel,
} from '@/utils/http-method'
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
  openNextApiInEdit,
} = useWorkspaceContext()

const mode = ref<DetailMode>('doc')
const folderMenuOpen = ref(false)
const loading = ref(false)
const saving = ref(false)
const saveUi = ref<'idle' | 'saving' | 'saved'>('idle')
let saveUiTimer: ReturnType<typeof setTimeout> | null = null

function onFolderMenuVisible(open: boolean) {
  folderMenuOpen.value = open
  if (!open) {
    void nextTick(() => {
      const el = document.activeElement
      if (el instanceof HTMLElement) el.blur()
    })
  }
}

function clearSaveUiTimer() {
  if (saveUiTimer) {
    clearTimeout(saveUiTimer)
    saveUiTimer = null
  }
}
const form = reactive(createEmptyInterfaceEditorForm({ method: 'GET', status: 2 }))
const docMeta = reactive({
  createdByName: '',
  updatedByName: '',
  updatedAt: '',
})

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
const docStatusLabel = computed(() => t(`workspace.statusChip.${interfaceStatusKey(form.status)}`))
const docStatusColor = computed(() => interfaceStatusColor(form.status))
const docProtocol = computed(() => protocolLabel(form.protocol))
const showDocProtocol = computed(() => !isProtocolOnlySelect(form.protocol))

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
  form.protocol = isKnownProtocol(detail.protocol)
    ? normalizeProtocol(detail.protocol)
    : inferProtocol(detail.method, detail.url)
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
  docMeta.createdByName = detail.created_by_name || ''
  docMeta.updatedByName = detail.updated_by_name || ''
  docMeta.updatedAt = detail.updated_at || detail.created_at || ''
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
    protocol: form.protocol,
    url: payload.url ?? '',
    status: form.status,
    created_by: prev?.created_by,
    created_by_name: prev?.created_by_name,
    updated_by: prev?.updated_by,
    updated_by_name: prev?.updated_by_name,
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
  if (openNextApiInEdit.value) {
    mode.value = 'edit'
    openNextApiInEdit.value = false
  } else {
    mode.value = 'doc'
  }
  if (id) void loadDetail()
}, { immediate: true })

async function handleSave(): Promise<boolean> {
  if (!interfaceId.value || readOnly.value || saveUi.value === 'saving' || loading.value) return false
  if (!form.name.trim()) {
    ElMessage.warning(t('workspace.nameRequired'))
    return false
  }
  if (!form.folderId) {
    ElMessage.warning(t('workspace.folderRequired'))
    return false
  }

  clearSaveUiTimer()
  saveUi.value = 'saving'
  saving.value = true
  beginSuppress()
  try {
    await submitUpdateInterface(interfaceId.value, buildInterfaceSavePayload(form))
    cacheFormAsDetail(interfaceId.value)
    await captureBaseline()
    saveUi.value = 'saved'
    saveUiTimer = setTimeout(() => {
      saveUi.value = 'idle'
      saveUiTimer = null
    }, 1400)
    return true
  } catch (error) {
    saveUi.value = 'idle'
    ElMessage.error(error instanceof Error ? error.message : t('workspace.updateApiFailed'))
    return false
  } finally {
    saving.value = false
  }
}

onBeforeUnmount(clearSaveUiTimer)

async function copyText(value: string, event?: Event) {
  const text = value.trim()
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(t('workspace.interfaceForm.rawCopySuccess'))
  } catch {
    ElMessage.error(t('workspace.interfaceForm.rawCopyFailed'))
  } finally {
    ;(event?.currentTarget as HTMLButtonElement | undefined)?.blur()
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
    </div>

    <div class="interface-detail__scroll">
      <div class="interface-detail__section">
        <div v-if="readOnly" class="interface-doc-head">
          <div class="interface-doc-head__title">
            <span v-if="showDocProtocol" class="interface-doc-head__protocol">{{ docProtocol }}</span>
            <span
              class="http-method-badge"
              :class="`http-method-badge--${methodTone(form.method)}`"
            >
              {{ shortMethodLabel(form.method) }}
            </span>
            <div class="interface-doc-head__copyable">
              <span class="interface-doc-head__url">{{ form.url || '/' }}</span>
              <button
                type="button"
                class="interface-doc-head__copy"
                tabindex="-1"
                :title="t('share.public.copyHint')"
                @click="copyText(form.url || '/', $event)"
              >
                <el-icon :size="14"><CopyDocument /></el-icon>
              </button>
            </div>
          </div>
          <div class="interface-doc-head__name-row">
            <span
              class="interface-doc-head__status"
              :style="{ '--status-color': docStatusColor }"
            >
              {{ docStatusLabel }}
            </span>
            <div class="interface-doc-head__copyable">
              <span class="interface-doc-head__name">{{ form.name || '—' }}</span>
              <button
                type="button"
                class="interface-doc-head__copy"
                tabindex="-1"
                :title="t('share.public.copyHint')"
                @click="copyText(form.name, $event)"
              >
                <el-icon :size="14"><CopyDocument /></el-icon>
              </button>
            </div>
          </div>
          <div class="interface-doc-head__meta">
            <span>
              {{ t('workspace.columns.creator') }}：{{ docMeta.createdByName || '—' }}
            </span>
            <span>
              {{ t('workspace.columns.lastEditor') }}：{{ docMeta.updatedByName || '—' }}
            </span>
            <span>
              {{ t('workspace.columns.updatedAt') }}：{{ docMeta.updatedAt || '—' }}
            </span>
          </div>
        </div>

        <template v-else>
          <div class="interface-editor-head">
            <InterfaceUrlBar
              v-model:protocol="form.protocol"
              v-model:method="form.method"
              v-model:url="form.url"
            />
            <div class="interface-url-actions">
              <InterfaceSaveButton
                :saving="saveUi === 'saving'"
                :saved="saveUi === 'saved'"
                :disabled="loading"
                @click="handleSave"
              />
            </div>
            <div class="interface-meta-row">
              <el-select
                v-model="form.folderId"
                class="interface-meta-row__folder"
                :class="{ 'is-dropdown-visible': folderMenuOpen }"
                :placeholder="t('workspace.selectFolder')"
                size="default"
                popper-class="app-action-dropdown"
                @visible-change="onFolderMenuVisible"
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
              />
            </div>
          </div>

          <div class="interface-detail__status-row">
            <span class="interface-detail__label">{{ t('workspace.columns.status') }}</span>
            <InterfaceStatusRadio v-model="form.status" />
          </div>
        </template>
      </div>

      <InterfaceEditorPanels
        :panel-state-key="interfaceId"
        :ready="!loading"
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

.interface-doc-head {
  display: flex;
  flex-direction: column;
  gap: 12px;
  --doc-chip-width: 52px;
  --doc-chip-height: 22px;
}

.interface-doc-head__title {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  flex-wrap: wrap;
}

.interface-doc-head .http-method-badge,
.interface-doc-head__protocol {
  width: var(--doc-chip-width);
  min-width: var(--doc-chip-width);
  height: var(--doc-chip-height);
  padding: 0;
  box-sizing: border-box;
  flex-shrink: 0;
  font-size: 11px;
}

.interface-doc-head__protocol {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-interface-bar-border);
  border-radius: 4px;
  font-weight: 700;
  font-family: Consolas, Monaco, monospace;
  line-height: 1;
  color: var(--color-interface-field-text);
}

.interface-doc-head__url,
.interface-doc-head__name {
  min-width: 0;
  font-size: 14px;
  font-weight: 400;
  color: var(--color-interface-field-text);
  word-break: break-all;
}

.interface-doc-head__url {
  font-family: Consolas, Monaco, monospace;
}

.interface-doc-head__name-row {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  flex-wrap: wrap;
}

.interface-doc-head__name {
  margin: 0;
  line-height: var(--doc-chip-height);
}

.interface-doc-head__copyable {
  position: relative;
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  padding-right: 28px;
}

.interface-doc-head__copy {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-secondary);
  opacity: 0;
  pointer-events: none;
  cursor: pointer;
}

.interface-doc-head__copyable:hover .interface-doc-head__copy {
  opacity: 1;
  pointer-events: auto;
}

.interface-doc-head__copy:hover {
  color: var(--color-interface-accent);
}

.interface-doc-head__status {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--doc-chip-width);
  min-width: var(--doc-chip-width);
  height: var(--doc-chip-height);
  padding: 0 0 1px;
  box-sizing: border-box;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
  color: var(--status-color);
  background: color-mix(in srgb, var(--status-color) 14%, transparent);
  border: 1px solid color-mix(in srgb, var(--status-color) 45%, transparent);
}

.interface-doc-head__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 20px;
  font-size: 13px;
  color: var(--color-text-secondary);
}
</style>
