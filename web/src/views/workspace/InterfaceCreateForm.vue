<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import InterfaceEditorPanels from '@/interface/InterfaceEditorPanels.vue'
import InterfaceSaveButton from '@/interface/InterfaceSaveButton.vue'
import InterfaceStatusRadio from '@/interface/InterfaceStatusRadio.vue'
import InterfaceUrlBar from '@/interface/InterfaceUrlBar.vue'
import { useInterfaceEditorDirty } from '@/composables/useInterfaceEditorDirty'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import {
  buildInterfaceSavePayload,
  createEmptyInterfaceEditorForm,
  defaultResponseExample,
  defaultResponseResult,
  interfaceEditorSnapshot,
  resetInterfaceEditorForm,
} from '@/utils/interface-editor-form'
import { collectFolderOptions } from '@/utils/interface-folder-options'

const { t } = useLocale()

const {
  apiTree,
  selectedFolder,
  submitCreateInterface,
  submitUpdateInterface,
  parseFolderId,
  setTabDirty,
  createApiSessionKey,
  CREATE_API_TAB_ID,
} = useWorkspaceContext()

const saving = ref(false)
const saveUi = ref<'idle' | 'saving' | 'saved'>('idle')
let saveUiTimer: ReturnType<typeof setTimeout> | null = null

function clearSaveUiTimer() {
  if (saveUiTimer) {
    clearTimeout(saveUiTimer)
    saveUiTimer = null
  }
}
const form = reactive(createEmptyInterfaceEditorForm({ method: 'POST', status: 3 }))
const savedInterfaceId = ref<number | null>(null)
const folderMenuOpen = ref(false)
const editorTabId = computed(() =>
  savedInterfaceId.value ? `api-${savedInterfaceId.value}` : CREATE_API_TAB_ID,
)

function onFolderMenuVisible(open: boolean) {
  folderMenuOpen.value = open
  if (!open) {
    void nextTick(() => {
      const el = document.activeElement
      if (el instanceof HTMLElement) el.blur()
    })
  }
}

const folderOptions = computed(() => collectFolderOptions(apiTree.value, parseFolderId))

const { beginSuppress, captureBaseline } = useInterfaceEditorDirty({
  getSnapshot: () => interfaceEditorSnapshot(form),
  setDirty: (dirty) => setTabDirty(editorTabId.value, dirty),
  enabled: () => !saving.value,
  watchSource: form,
})

function resolveFolderId(): number | null {
  if (selectedFolder.value) {
    return parseFolderId(selectedFolder.value.id)
  }
  if (folderOptions.value.length > 0) {
    return folderOptions.value[0].folderId
  }
  return null
}

async function resetForNewApi() {
  savedInterfaceId.value = null
  beginSuppress()
  resetInterfaceEditorForm(form, {
    method: 'POST',
    status: 3,
    folderId: resolveFolderId(),
  })
  form.responseResults = [defaultResponseResult(t('workspace.interfaceForm.defaultSuccess'))]
  form.responseExamples = [defaultResponseExample(t('workspace.interfaceForm.successExample'))]
  await captureBaseline()
}

watch(createApiSessionKey, () => {
  void resetForNewApi()
}, { immediate: true })

watch(selectedFolder, (folder) => {
  if (savedInterfaceId.value || !folder) return
  const nextFolderId = parseFolderId(folder.id)
  if (nextFolderId != null && form.folderId !== nextFolderId) {
    form.folderId = nextFolderId
  }
})

async function handleSave(): Promise<boolean> {
  if (saveUi.value === 'saving') return false
  const folderId = form.folderId
  if (!folderId) {
    ElMessage.warning(t('workspace.folderRequired'))
    return false
  }
  if (!form.name.trim()) {
    ElMessage.warning(t('workspace.nameRequired'))
    return false
  }

  clearSaveUiTimer()
  saveUi.value = 'saving'
  saving.value = true
  beginSuppress()
  try {
    const payload = {
      ...buildInterfaceSavePayload(form),
      folder_id: folderId,
    }
    if (savedInterfaceId.value) {
      await submitUpdateInterface(savedInterfaceId.value, payload)
    } else {
      const createdId = await submitCreateInterface(payload)
      savedInterfaceId.value = createdId ?? null
    }
    await captureBaseline()
    saveUi.value = 'saved'
    saveUiTimer = setTimeout(() => {
      saveUi.value = 'idle'
      saveUiTimer = null
    }, 1400)
    return true
  } catch (error) {
    saveUi.value = 'idle'
    ElMessage.error(error instanceof Error ? error.message : t('workspace.createApiFailed'))
    return false
  } finally {
    saving.value = false
  }
}

onBeforeUnmount(clearSaveUiTimer)

defineExpose({
  save: handleSave,
})
</script>

<template>
  <div class="interface-create interface-editor">
    <div class="interface-create__scroll">
      <div class="interface-create__section">
        <div class="interface-editor-head">
          <InterfaceUrlBar
            v-model:protocol="form.protocol"
            v-model:method="form.method"
            v-model:url="form.url"
            create-mode
          />
          <div class="interface-url-actions">
            <InterfaceSaveButton
              :saving="saveUi === 'saving'"
              :saved="saveUi === 'saved'"
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

        <div class="interface-create__status-row">
          <span class="interface-create__label">{{ t('workspace.columns.status') }}</span>
          <InterfaceStatusRadio v-model="form.status" />
        </div>
      </div>

      <InterfaceEditorPanels
        :key="createApiSessionKey"
        :panel-state-key="createApiSessionKey"
        default-open
        v-model:request-headers="form.requestHeaders"
        v-model:query-params="form.queryParams"
        v-model:request-body="form.requestBody"
        v-model:response-headers="form.responseHeaders"
        v-model:response-results="form.responseResults"
        v-model:response-examples="form.responseExamples"
      />
    </div>
  </div>
</template>

<style scoped>
.interface-create {
  display: flex;
  flex-direction: column;
  flex: 1;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.interface-create__scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  overscroll-behavior: contain;
  padding: 12px 0 32px;
}

.interface-create__section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-bottom: 20px;
  padding-left: var(--color-interface-section-padding-x);
  padding-right: var(--color-interface-section-padding-x);
  border-bottom: 1px solid var(--color-border);
}

.interface-create__status-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.interface-create__label {
  color: var(--color-interface-field-text);
  font-size: 14px;
  flex-shrink: 0;
}
</style>
