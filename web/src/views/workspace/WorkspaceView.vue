<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Close, Delete } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import EnvironmentManagePanel from '@/views/workspace/EnvironmentManagePanel.vue'
import InterfaceCreateForm from '@/views/workspace/InterfaceCreateForm.vue'
import InterfaceDetailPanel from '@/views/workspace/InterfaceDetailPanel.vue'
import ProjectManagePanel from '@/views/workspace/ProjectManagePanel.vue'
import VariableManagePanel from '@/views/workspace/VariableManagePanel.vue'
import { useSaveShortcut } from '@/composables/useInterfaceEditorDirty'
import { useLocale } from '@/composables/useLocale'
import { interfaceStatusKey } from '@/constants/interface-status'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { AppModule, HttpMethod, WorkspaceTab } from '@/types/workspace'

const { t } = useLocale()
const {
  activeModule,
  activeProject,
  selectedFolder,
  selectedFolderId,
  contextMode,
  folderInterfaces,
  loadingInterfaces,
  environments,
  activeEnvironmentId,
  selectEnvironment,
  openCreateApi,
  selectApi,
  submitDeleteInterface,
  workspaceTabs,
  moduleTabs,
  activeModuleTab,
  activeTabId,
  activateTab,
  closeTab,
  closeTabsLeft,
  closeTabsRight,
  closeOtherTabs,
  closeAllTabs,
} = useWorkspaceContext()

const moduleTitleMap: Record<AppModule, string> = {
  api: 'workspace.modules.api',
  'quick-test': 'workspace.modules.quickTest',
  environment: 'workspace.modules.environment',
  project: 'workspace.modules.project',
}

const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  tabId: '',
})

const methodColors: Record<HttpMethod, string> = {
  GET: 'get',
  POST: 'post',
  PUT: 'put',
  DELETE: 'delete',
  PATCH: 'patch',
  HEAD: 'head',
  OPTIONS: 'options',
}

const pageTitle = computed(() => {
  if (activeModuleTab.value) {
    return tabLabel(activeModuleTab.value)
  }
  if (activeModule.value === 'api' && activeProject.value) {
    return t('workspace.apiList')
  }
  return t(moduleTitleMap[activeModule.value])
})

const pageDesc = computed(() => {
  if (activeModule.value === 'api') {
    if (!activeProject.value) return t('workspace.apiDescNoProject')
    if (activeModuleTab.value?.kind === 'folder') return t('workspace.apiDescList')
    if (activeModuleTab.value?.kind === 'api') return t('workspace.apiDescDetail')
    if (activeModuleTab.value?.kind === 'create-api') return ''
    return t('workspace.apiDescSelectFolder')
  }
  if (activeModule.value === 'quick-test') return t('workspace.quickTestDesc')
  if (activeModule.value === 'environment') {
    if (!activeProject.value) return t('workspace.apiDescNoProject')
    if (activeModuleTab.value?.kind === 'env-variables') return t('environment.variableDesc')
    return t('workspace.environmentDesc')
  }
  if (activeModule.value === 'project') {
    return ''
  }
  return t('workspace.projectDesc')
})

const showCreateForm = computed(
  () =>
    activeModule.value === 'api' &&
    contextMode.value === 'project' &&
    activeModuleTab.value?.kind === 'create-api',
)

const showFolderView = computed(
  () =>
    activeModule.value === 'api' &&
    contextMode.value === 'project' &&
    activeModuleTab.value?.kind === 'folder' &&
    !!selectedFolderId.value,
)

const showApiDetail = computed(
  () =>
    activeModule.value === 'api' &&
    contextMode.value === 'project' &&
    activeModuleTab.value?.kind === 'api',
)

const showApiWorkspacePanels = computed(
  () =>
    activeModule.value === 'api' &&
    contextMode.value === 'project' &&
    (showCreateForm.value || showApiDetail.value || showFolderView.value),
)

const showSpaceManage = computed(() => activeModule.value === 'project' && !!activeModuleTab.value)

const showEnvironmentManage = computed(
  () => activeModule.value === 'environment' && activeModuleTab.value?.kind === 'env-list' && !!activeProject.value,
)

const showVariableManage = computed(
  () =>
    activeModule.value === 'environment' &&
    activeModuleTab.value?.kind === 'env-variables' &&
    !!activeProject.value,
)

const showQuickTest = computed(
  () => activeModule.value === 'quick-test' && activeModuleTab.value?.kind === 'quick-test',
)

const showApiTabbar = computed(
  () => activeModule.value === 'api' && contextMode.value === 'project',
)

const showTabbar = computed(
  () =>
    showApiTabbar.value ||
    activeModule.value === 'project' ||
    (activeModule.value === 'environment' && !!activeProject.value) ||
    activeModule.value === 'quick-test',
)

const showRuntimeEnvSelect = computed(
  () => !!activeProject.value && (activeModule.value === 'api' || activeModule.value === 'quick-test'),
)

const tabSupportsContextMenu = computed(() => activeModule.value === 'api')

const deletingApiId = ref<number | null>(null)
const tableWrapRef = ref<HTMLElement | null>(null)
const tableWrapHeight = ref(400)
const TABLE_HEADER_HEIGHT = 40
const TABLE_ROW_HEIGHT = 40
let tableResizeObserver: ResizeObserver | null = null

const apiRows = computed(() =>
  folderInterfaces.value.map((item) => ({
    id: item.id,
    name: item.name,
    status: interfaceStatusKey(item.status),
    method: item.method,
    url: item.url,
    group: item.folder_name || selectedFolder.value?.name || '',
    updatedByName: item.updated_by_name || '-',
    createdAt: item.created_at,
    updatedAt: item.updated_at || item.created_at,
  })),
)

function setupTableResizeObserver() {
  tableResizeObserver?.disconnect()
  if (!tableWrapRef.value) return

  tableResizeObserver = new ResizeObserver(([entry]) => {
    tableWrapHeight.value = Math.floor(entry.contentRect.height)
  })
  tableResizeObserver.observe(tableWrapRef.value)
}

const tableContentHeight = computed(
  () => TABLE_HEADER_HEIGHT + apiRows.value.length * TABLE_ROW_HEIGHT,
)

const tableNeedsScroll = computed(() => {
  if (apiRows.value.length === 0) return false
  const measured = tableWrapHeight.value
  if (measured <= TABLE_HEADER_HEIGHT) return false
  return tableContentHeight.value > measured
})

const tableHeight = computed(() => {
  const rowCount = apiRows.value.length
  const contentHeight = tableContentHeight.value
  const measured = tableWrapHeight.value

  if (rowCount === 0) {
    return loadingInterfaces.value && measured > TABLE_HEADER_HEIGHT
      ? measured
      : TABLE_HEADER_HEIGHT
  }

  if (measured <= TABLE_HEADER_HEIGHT) {
    return contentHeight
  }

  return tableNeedsScroll.value ? measured : contentHeight
})

watch(showFolderView, async (visible) => {
  if (!visible) {
    tableResizeObserver?.disconnect()
    return
  }
  await nextTick()
  setupTableResizeObserver()
})

watch([tableNeedsScroll, () => apiRows.value.length], async () => {
  if (!showFolderView.value) return
  await nextTick()
  setupTableResizeObserver()
})

function tabLabel(tab: WorkspaceTab) {
  if (tab.kind === 'create-api') return t('workspace.newApi')
  if (tab.kind === 'workspace-list') return t('workspace.workspaceManage')
  if (tab.kind === 'project-list') return t('workspace.projectManage')
  if (tab.kind === 'member-list') return t('member.title')
  if (tab.kind === 'env-list') return t('workspace.modules.environment')
  if (tab.kind === 'env-variables' && tab.id === 'env-variables-empty') return t('environment.variables')
  if (tab.kind === 'quick-test') return t('workspace.modules.quickTest')
  return tab.label
}

function isTabClosable(tab: WorkspaceTab) {
  return tab.closable !== false && activeModule.value === 'api'
}

function handleAddApi() {
  openCreateApi(selectedFolder.value?.id)
}

function handleOpenApi(apiId: number) {
  selectApi(`api-${apiId}`)
}

function handleRowClick(row: { id: number }) {
  handleOpenApi(row.id)
}

async function handleDeleteApi(apiId: number, name: string) {
  try {
    await ElMessageBox.confirm(
      t('workspace.deleteApiConfirm', { name }),
      t('common.delete'),
      { type: 'warning' },
    )
  } catch {
    return
  }

  deletingApiId.value = apiId
  try {
    await submitDeleteInterface(apiId)
    ElMessage.success(t('workspace.deleteApiSuccess'))
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.deleteApiFailed'))
  } finally {
    deletingApiId.value = null
  }
}

function handleTabClick(tab: WorkspaceTab) {
  activateTab(tab.id)
}

async function confirmDiscardUnsaved(tab: WorkspaceTab): Promise<boolean> {
  if (!tab.dirty) return true
  try {
    await ElMessageBox.confirm(
      tab.kind === 'create-api'
        ? t('workspace.discardCreateConfirm')
        : t('workspace.unsavedCloseConfirm'),
      tab.kind === 'create-api'
        ? t('workspace.discardCreateTitle')
        : t('workspace.unsavedCloseTitle'),
      {
        type: 'warning',
        confirmButtonText:
          tab.kind === 'create-api'
            ? t('workspace.discardCreateAction')
            : t('workspace.unsavedCloseAction'),
        cancelButtonText: t('common.cancel'),
      },
    )
    return true
  } catch {
    return false
  }
}

async function confirmCloseCreateApi() {
  const createTab = workspaceTabs.value.find((item) => item.id === 'create-api')
  if (createTab && !(await confirmDiscardUnsaved(createTab))) return
  closeTab('create-api')
}

async function handleTabClose(tab: WorkspaceTab, event?: MouseEvent) {
  event?.stopPropagation()
  if (!(await confirmDiscardUnsaved(tab))) return
  closeTab(tab.id)
}

type EditorSaveHandle = { save: () => void | boolean | Promise<void | boolean> }
const createFormRef = ref<EditorSaveHandle | null>(null)
const detailFormRef = ref<EditorSaveHandle | null>(null)

useSaveShortcut(
  () => {
    if (showCreateForm.value) return createFormRef.value?.save()
    if (showApiDetail.value) return detailFormRef.value?.save()
  },
  () => showCreateForm.value || showApiDetail.value,
)

function openTabContextMenu(event: MouseEvent, tab: WorkspaceTab) {
  if (!isTabClosable(tab)) return
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    tabId: tab.id,
  }
}

function hideContextMenu() {
  contextMenu.value.visible = false
}

function runContextAction(action: 'others' | 'left' | 'right' | 'all') {
  const tabId = contextMenu.value.tabId
  hideContextMenu()

  switch (action) {
    case 'others':
      closeOtherTabs(tabId)
      break
    case 'left':
      closeTabsLeft(tabId)
      break
    case 'right':
      closeTabsRight(tabId)
      break
    case 'all':
      closeAllTabs()
      break
  }
}

onMounted(() => {
  document.addEventListener('click', hideContextMenu)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', hideContextMenu)
  tableResizeObserver?.disconnect()
})
</script>

<template>
  <div class="workspace-main">
    <div v-if="showTabbar" class="workspace-main__tabbar">
      <div class="workspace-main__tabbar-tabs">
        <div
          v-for="tab in moduleTabs"
          :key="tab.id"
          class="workspace-main__tab"
          :class="{ 'workspace-main__tab--active': activeTabId === tab.id }"
          @click="handleTabClick(tab)"
          @contextmenu.prevent="openTabContextMenu($event, tab)"
        >
          <span
            v-if="tab.kind === 'api' && tab.method"
            class="workspace-main__tab-method"
            :class="`workspace-main__tab-method--${methodColors[tab.method]}`"
          >
            {{ tab.method }}
          </span>
          <span class="workspace-main__tab-label">
            {{ tabLabel(tab) }}
            <span v-if="tab.dirty" class="workspace-main__tab-dirty">*</span>
          </span>
          <button
            v-if="isTabClosable(tab)"
            type="button"
            class="workspace-main__tab-close"
            aria-label="Close"
            @click="handleTabClose(tab, $event)"
          >
            <el-icon :size="12"><Close /></el-icon>
          </button>
        </div>
      </div>

      <div v-if="showRuntimeEnvSelect" class="workspace-main__tabbar-env">
        <el-select
          :model-value="activeEnvironmentId"
          class="workspace-main__env-select"
          popper-class="app-action-dropdown"
          :placeholder="t('workspace.noEnvironment')"
          :disabled="!environments.length"
          :suffix-icon="''"
          @change="(id: number) => selectEnvironment(id)"
        >
          <el-option
            v-for="item in environments"
            :key="item.id"
            :label="item.name"
            :value="item.id"
          />
        </el-select>
      </div>
    </div>

    <div v-else class="workspace-main__tabbar workspace-main__tabbar--simple">
      <div class="workspace-main__tab workspace-main__tab--active workspace-main__tab--solo">
        <span class="workspace-main__tab-label">{{ pageTitle }}</span>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="contextMenu.visible && tabSupportsContextMenu"
        class="workspace-main__tab-menu"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
        @click.stop
        @contextmenu.prevent
      >
        <button
          type="button"
          class="workspace-main__tab-menu-item"
          :disabled="moduleTabs.filter((item) => isTabClosable(item)).length <= 1"
          @click="runContextAction('others')"
        >
          {{ t('workspace.tabCloseOthers') }}
        </button>
        <button
          type="button"
          class="workspace-main__tab-menu-item"
          :disabled="workspaceTabs.findIndex((item) => item.id === contextMenu.tabId) <= 0"
          @click="runContextAction('left')"
        >
          {{ t('workspace.tabCloseLeft') }}
        </button>
        <button
          type="button"
          class="workspace-main__tab-menu-item"
          :disabled="
            workspaceTabs.findIndex((item) => item.id === contextMenu.tabId) >=
            workspaceTabs.length - 1
          "
          @click="runContextAction('right')"
        >
          {{ t('workspace.tabCloseRight') }}
        </button>
        <button type="button" class="workspace-main__tab-menu-item" @click="runContextAction('all')">
          {{ t('workspace.tabCloseAll') }}
        </button>
      </div>
    </Teleport>

    <div class="workspace-main__content">
      <template v-if="showSpaceManage">
        <ProjectManagePanel />
      </template>

      <template v-else-if="showVariableManage">
        <VariableManagePanel v-if="activeModuleTab?.environmentId" />
        <div v-else class="workspace-main__placeholder">
          <h2>{{ t('environment.variables') }}</h2>
          <p>{{ t('environment.noEnvironmentForVariables') }}</p>
        </div>
      </template>

      <template v-else-if="showEnvironmentManage">
        <EnvironmentManagePanel />
      </template>

      <template v-else-if="showQuickTest">
        <div class="workspace-main__placeholder">
          <h2>{{ pageTitle }}</h2>
          <p>{{ pageDesc }}</p>
        </div>
      </template>

      <template v-else-if="showApiWorkspacePanels">
        <keep-alive>
          <InterfaceCreateForm
            v-if="showCreateForm"
            ref="createFormRef"
            @request-close="confirmCloseCreateApi"
          />
        </keep-alive>
        <keep-alive>
          <InterfaceDetailPanel v-if="showApiDetail" ref="detailFormRef" />
        </keep-alive>
        <div v-if="showFolderView" class="workspace-main__folder-view">
            <div class="workspace-main__toolbar">
              <el-button type="primary" class="workspace-action-btn" @click="handleAddApi">
                <span class="workspace-action-btn__plus">+</span>
                <span>{{ t('workspace.addApi') }}</span>
              </el-button>
            </div>

            <div
              ref="tableWrapRef"
              class="workspace-main__table-wrap"
              :class="{ 'workspace-main__table-wrap--fill': tableNeedsScroll }"
            >
              <el-table
                v-loading="loadingInterfaces"
                :data="apiRows"
                :height="tableHeight"
                class="workspace-main__table workspace-data-table workspace-data-table--clickable"
                @row-click="handleRowClick"
              >
                <el-table-column
                  type="index"
                  :label="t('workspace.columns.index')"
                  width="72"
                  align="center"
                  fixed="left"
                  label-class-name="workspace-main__col-fit"
                  class-name="workspace-main__nowrap"
                />
                <el-table-column
                  :label="t('workspace.columns.name')"
                  prop="name"
                  width="140"
                  fixed="left"
                  show-overflow-tooltip
                />
                <el-table-column
                  :label="t('workspace.columns.status')"
                  width="88"
                  label-class-name="workspace-main__col-fit"
                  class-name="workspace-main__col-fit"
                >
                  <template #default="{ row }">
                    <span
                      class="workspace-main__status"
                      :class="`workspace-main__status--${row.status}`"
                    >
                      {{ t(`workspace.status.${row.status}`) }}
                    </span>
                  </template>
                </el-table-column>
                <el-table-column
                  :label="t('workspace.columns.protocolMethod')"
                  width="100"
                  align="left"
                  label-class-name="workspace-main__col-fit"
                  class-name="workspace-main__col-fit"
                >
                  <template #default="{ row }">
                    <span
                      class="http-method-badge"
                      :class="`http-method-badge--${row.method.toLowerCase()}`"
                    >
                      {{ row.method }}
                    </span>
                  </template>
                </el-table-column>
                <el-table-column
                  :label="t('workspace.columns.url')"
                  prop="url"
                  min-width="160"
                  show-overflow-tooltip
                />
                <el-table-column
                  :label="t('workspace.columns.group')"
                  prop="group"
                  min-width="100"
                  show-overflow-tooltip
                />
                <el-table-column
                  :label="t('workspace.columns.owner')"
                  prop="updatedByName"
                  min-width="100"
                  show-overflow-tooltip
                />
                <el-table-column
                  :label="t('workspace.columns.createdAt')"
                  prop="createdAt"
                  width="160"
                  label-class-name="workspace-main__nowrap"
                  class-name="workspace-main__nowrap"
                  show-overflow-tooltip
                />
                <el-table-column
                  :label="t('workspace.columns.updatedAt')"
                  prop="updatedAt"
                  width="160"
                  label-class-name="workspace-main__nowrap"
                  class-name="workspace-main__nowrap"
                  show-overflow-tooltip
                />
                <el-table-column
                  :label="t('member.columns.actions')"
                  width="52"
                  fixed="right"
                  label-class-name="workspace-main__nowrap"
                  class-name="workspace-main__nowrap workspace-main__actions-col"
                >
                  <template #default="{ row }">
                    <div class="workspace-main__actions">
                      <button
                        type="button"
                        class="workspace-main__action-btn workspace-main__action-btn--danger"
                        :title="t('common.delete')"
                        :disabled="deletingApiId === row.id"
                        @click.stop="handleDeleteApi(row.id, row.name)"
                      >
                        <el-icon :size="14"><Delete /></el-icon>
                      </button>
                    </div>
                  </template>
                </el-table-column>
              </el-table>
            </div>
            <div class="workspace-main__footer">
              {{ t('workspace.loadedRecords', { count: String(apiRows.length) }) }}
            </div>
          </div>
      </template>

      <template v-else>
        <div class="workspace-main__placeholder">
          <h2>{{ pageTitle }}</h2>
          <p>{{ pageDesc }}</p>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.workspace-main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-workspace-content);
  overflow: hidden;
  font-size: 14px;
}

.workspace-main__tabbar {
  display: flex;
  align-items: stretch;
  height: 44px;
  flex-shrink: 0;
  background: var(--color-workspace-elevated);
  border-bottom: 1px solid var(--color-border);
}

.workspace-main__tabbar--simple {
  background: var(--color-workspace-content);
}

.workspace-main__tabbar-tabs {
  display: flex;
  align-items: stretch;
  min-width: 0;
  flex: 1;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: thin;
}

.workspace-main__tabbar-tabs::-webkit-scrollbar {
  height: 4px;
}

.workspace-main__tabbar-tabs::-webkit-scrollbar-thumb {
  background: var(--color-workspace-scrollbar);
  border-radius: 2px;
}

.workspace-main__tab {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 44px;
  min-width: 140px;
  max-width: 240px;
  padding: 0 16px;
  border-right: 1px solid var(--color-border);
  font-size: 14px;
  color: var(--color-workspace-tab-text);
  background: var(--color-workspace-elevated);
  cursor: pointer;
  flex-shrink: 0;
  user-select: none;
}

.workspace-main__tab--solo {
  min-width: auto;
  max-width: none;
  border-right: none;
  background: var(--color-workspace-content);
  color: var(--color-workspace-tab-text-active);
}

.workspace-main__tab--active {
  color: var(--color-workspace-tab-text-active);
  background: var(--color-workspace-content);
}

.workspace-main__tab--active::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--color-primary);
}

.workspace-main__tab-method {
  font-size: 12px;
  font-weight: 700;
  font-family: 'Consolas', 'Monaco', monospace;
  flex-shrink: 0;
}

.workspace-main__tab-method--get {
  color: #61affe;
}

.workspace-main__tab-method--post {
  color: #49cc90;
}

.workspace-main__tab-method--put {
  color: #fca130;
}

.workspace-main__tab-method--delete {
  color: #f93e3e;
}

.workspace-main__tab-method--patch {
  color: #50e3c2;
}

.workspace-main__tab-method--head,
.workspace-main__tab-method--options {
  color: var(--color-workspace-tab-text);
}

.workspace-main__tab-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.workspace-main__tab-dirty {
  margin-left: 2px;
  color: #fca130;
}

.workspace-main__tab-close {
  width: 16px;
  height: 16px;
  border: none;
  border-radius: 2px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  flex-shrink: 0;
}

.workspace-main__tab-close:hover {
  background: var(--color-workspace-menu-hover);
  color: var(--color-workspace-tab-text-active);
}

.workspace-main__tabbar-env {
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-left: 1px solid var(--color-border);
  background: var(--color-workspace-elevated);
  flex-shrink: 0;
}

.workspace-main__env-select {
  width: 140px;
}

.workspace-main__env-select :deep(.el-select__wrapper) {
  background: transparent;
  box-shadow: none !important;
  padding: 0 4px;
  min-height: 32px;
}

.workspace-main__env-select :deep(.el-select__selected-item) {
  color: var(--color-workspace-tab-text-active);
  font-size: 14px;
}

.workspace-main__env-select :deep(.el-select__suffix),
.workspace-main__env-select :deep(.el-select__caret) {
  display: none;
}

.workspace-main__tab-menu {
  position: fixed;
  z-index: 3000;
  min-width: 140px;
  padding: 4px 0;
  background: var(--color-workspace-menu-bg);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
}

.workspace-main__tab-menu-item {
  display: block;
  width: 100%;
  padding: 8px 14px;
  border: none;
  background: transparent;
  color: var(--color-workspace-tab-text-active);
  font-size: 14px;
  text-align: left;
  cursor: pointer;
}

.workspace-main__tab-menu-item:hover:not(:disabled) {
  background: var(--color-workspace-menu-hover);
}

.workspace-main__tab-menu-item:disabled {
  color: var(--color-text-secondary);
  cursor: not-allowed;
}

.workspace-main__content {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 0;
  background: var(--color-workspace-content);
  font-size: 14px;
}

.workspace-main__toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 10px 10px 8px;
  background: var(--color-workspace-content);
}

.workspace-main__content:has(.interface-detail),
.workspace-main__content:has(.interface-create) {
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.workspace-main__content :is(.interface-detail, .interface-create) {
  flex: 1;
  min-height: 0;
}

.workspace-main__content:has(.workspace-main__folder-view) {
  overflow: hidden;
}

.workspace-main__folder-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.workspace-main__folder-view .workspace-main__footer {
  margin-top: auto;
}

.workspace-main__table-wrap {
  flex: 0 1 auto;
  min-width: 0;
  width: 100%;
  overflow: hidden;
}

.workspace-main__table-wrap--fill {
  flex: 1;
  min-height: 0;
}

.workspace-main__table {
  width: 100%;
  --el-table-bg-color: var(--color-workspace-content);
  --el-table-tr-bg-color: var(--color-workspace-content);
  --el-table-header-bg-color: var(--color-workspace-content);
  --el-table-row-hover-bg-color: var(--color-workspace-row-hover);
}

.workspace-main__table :deep(.el-table__body tr) {
  cursor: pointer;
}

.workspace-main__table :deep(.el-table__body-wrapper) {
  background: var(--color-workspace-content);
  cursor: default;
}

.workspace-main__table :deep(th.el-table__cell),
.workspace-main__table :deep(td.el-table__cell) {
  background-color: var(--color-workspace-content) !important;
}

.workspace-main__table :deep(.el-table__body tr) {
  background: var(--color-workspace-content);
}

.workspace-main__table :deep(.el-table__body tr:hover > td.el-table__cell) {
  background-color: var(--color-workspace-row-hover) !important;
}

.workspace-main__table :deep(.el-table__fixed),
.workspace-main__table :deep(.el-table__fixed-right) {
  background-color: var(--color-workspace-content);
}

.workspace-main__table :deep(.el-table__fixed-right-patch),
.workspace-main__table :deep(.el-table__fixed-left-patch) {
  background-color: var(--color-workspace-content);
}

.workspace-main__table :deep(td.el-table-fixed-column--left),
.workspace-main__table :deep(th.el-table-fixed-column--left),
.workspace-main__table :deep(td.el-table-fixed-column--right),
.workspace-main__table :deep(th.el-table-fixed-column--right) {
  background-color: var(--color-workspace-content) !important;
}

.workspace-main__table :deep(.el-table__body tr:hover > td.el-table-fixed-column--left),
.workspace-main__table :deep(.el-table__body tr:hover > td.el-table-fixed-column--right) {
  background-color: var(--color-workspace-row-hover) !important;
}

.workspace-main__table :deep(.el-table__cell) {
  padding: 0 12px;
  height: 40px;
}

.workspace-main__table :deep(.el-table__header .el-table__cell) {
  height: 40px;
}

.workspace-main__table :deep(.el-table__body tr) {
  height: 40px;
}

.workspace-main__table :deep(.el-table__inner-wrapper::before) {
  display: none;
}

.workspace-main__table :deep(.el-table__border-left-patch),
.workspace-main__table :deep(.el-table__border-bottom-patch) {
  display: none;
}

.workspace-main__table :deep(.workspace-main__nowrap .cell) {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.workspace-main__table :deep(.workspace-main__col-fit .cell) {
  white-space: nowrap;
  overflow: visible;
}

.workspace-main__actions {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.workspace-main__action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.workspace-main__action-btn:hover:not(:disabled) {
  background: var(--color-workspace-control-hover);
  color: var(--color-text);
}

.workspace-main__action-btn--danger:hover:not(:disabled) {
  color: #f48771;
}

.workspace-main__action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.workspace-main__table :deep(.workspace-main__actions-col .cell) {
  overflow: visible;
}

.workspace-main__footer {
  padding: 12px 20px;
  font-size: 14px;
  color: var(--color-text-secondary);
  border-top: 1px solid var(--color-border);
  background: var(--color-workspace-content);
}

.workspace-main__status {
  font-size: 14px;
  font-weight: 500;
}

.workspace-main__status--published {
  color: #49cc90;
}

.workspace-main__status--testing {
  color: #fca130;
}

.workspace-main__status--developing {
  color: #61affe;
}

.workspace-main__status--abnormal {
  color: #f93e3e;
}

.workspace-main__status--maintenance {
  color: #fca130;
}

.workspace-main__status--deprecated {
  color: #909399;
}

.workspace-main__placeholder {
  padding: 48px 32px;
  text-align: center;
}

.workspace-main__placeholder h2 {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}

.workspace-main__placeholder p {
  margin: 0;
  font-size: 14px;
  color: var(--color-text-secondary);
  line-height: 1.6;
}
</style>
