<template>
  <div class="shell">
    <aside class="sidebar" :style="{ width: `${sidebarWidth}px` }">
      <div class="project-bar">
        <SelectPicker
          :model-value="projectId"
          :items="projects"
          :placeholder="t('project.empty')"
          :search-placeholder="t('project.search')"
          :empty-result="t('project.none')"
          :create-label="t('create.project')"
          :item-actions="true"
          variant="project"
          @update:model-value="onPickProject"
          @create="openCreateProject"
          @edit="openEditProject"
          @delete="openDeleteProject"
        />
      </div>

      <div class="search-row">
        <input
          v-model="search"
          class="input search"
          :placeholder="t('search.placeholder')"
          @input="reloadTree"
        />
        <SplitCreateButton
          :add-api-label="t('tree.addApi')"
          :add-folder-label="t('tree.addFolder')"
          :import-label="t('importExport.import')"
          :export-label="t('importExport.export')"
          @add-api="openCreateApi(0)"
          @add-folder="openCreateFolder(0, $event, t('tree.addFolder'))"
          @import="openImportDialog"
          @export="exportCurrentProject"
        />
      </div>

      <div class="tree-wrap" @contextmenu.prevent="openTreeBlankContextMenu">
        <TreeList
          v-if="tree.length"
          :items="tree"
          :active-id="activeInterfaceId"
          :expanded-folder-ids="expandedFolderIds"
          :can-drag="canDragApis"
          :dragging-api-id="dragApiState?.apiId ?? null"
          :drag-over-key="dragOverKey"
          :reordering="reorderingApis"
          @select="onSelectApi"
          @action="onTreeAction"
          @toggle-expand="toggleFolderExpand"
          @api-drag-start="onApiDragStart"
          @api-drag-over="onApiDragOver"
          @api-drag-leave="onApiDragLeave"
          @api-drop="onApiDrop"
          @api-drag-end="resetApiDragState"
        />
        <p v-else class="muted empty">{{ t('tree.empty') }}</p>
      </div>
    </aside>

    <div
      class="sidebar-resizer"
      :class="{ 'sidebar-resizer--active': sidebarDragging }"
      @mousedown="startSidebarDrag"
    />

    <main class="main">
      <div class="content-top">
        <div class="req-tabs" ref="tabsEl" @wheel="onTabsWheel">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            type="button"
            class="req-tab"
            :class="{ active: tab.id === activeTabId, dirty: tab.dirty }"
            :title="tab.interfaceId != null ? t('request.locateInTree') : undefined"
            @click="activateTab(tab.id)"
            @dblclick="onTabDblClick(tab, $event)"
            @contextmenu.prevent="openTabContextMenu($event, tab.id)"
          >
            <span class="method" :class="tab.method.toLowerCase()">{{ tab.method }}</span>
            <span class="req-tab-name">{{ tab.name || t('request.untitled') }}</span>
            <span
              class="req-tab-close"
              title="Close"
              @click.stop="requestCloseTab(tab.id)"
              @dblclick.stop
            >
              <span class="req-tab-close-x">×</span>
            </span>
          </button>
          <button type="button" class="tab-add" :title="t('request.newTab')" @click="addTab">
            +
          </button>
        </div>

        <div class="env-wrap">
          <SelectPicker
            :model-value="environmentId"
            :items="environments"
            :placeholder="t('env.label')"
            :search-placeholder="t('env.search')"
            :empty-result="t('env.none')"
            :allow-empty="true"
            :empty-label="t('env.noneSelected')"
            :manage-label="t('env.manage')"
            variant="env"
            align="right"
            @update:model-value="onPickEnv"
            @manage="envManageOpen = true"
          />
        </div>
        <button
          type="button"
          class="code-toggle"
          :class="{ active: codePanelOpen }"
          :title="t('codeSnippet.toggle')"
          @click="toggleCodePanel"
        >
          <svg
            class="code-toggle__icon"
            viewBox="0 0 1024 1024"
            width="16"
            height="16"
            aria-hidden="true"
          >
            <path
              fill="currentColor"
              d="M345.7 904c-3.4 0-6.6-0.7-9.7-2.1-11.6-5.3-16.6-19.1-11.3-30.7l340.8-739.1c3.8-8.2 12-13.5 21-13.5 3.4 0 6.6 0.7 9.7 2.1 11.6 5.4 16.6 19.1 11.3 30.7L366.7 890.6c-3.7 8.2-12 13.4-21 13.4z m410.9-140.9c-13.2 0-23.9-10.7-23.9-23.9 0-6.3 2.6-12.5 7.1-16.9l210.9-211-211-211.1c-4.4-4.4-7-10.5-7-16.9 0-13.2 10.7-23.9 23.9-23.9 6.3 0 12.2 2.5 16.8 7l244.7 244.8-244.8 244.9c-4.5 4.6-10.4 7-16.7 7z m-493 0c-6.3 0-12.2-2.5-16.8-7L2.1 511.4l244.7-244.8c4.5-4.5 10.5-7 16.8-7 13.2 0 23.9 10.7 23.9 23.9 0 6.3-2.5 12.4-7 16.9l-211 211.1 211 211.1c4.4 4.3 7 10.5 7 16.8 0 13-10.7 23.7-23.9 23.7z"
            />
          </svg>
        </button>
      </div>

      <div ref="workRowRef" class="work-row">
        <div class="work">
          <InterfaceEditor
            v-if="workspaceId && projectId && editorReady"
            ref="editorRef"
            :key="editorKey"
            class="work-editor"
            :mode="editorMode"
            :workspace-id="workspaceId"
            :project-id="projectId"
            :tab-id="activeTabId"
            :interface-id="activeInterfaceId"
            :initial="(editorInitial as any)"
            @saved="onInterfaceSaved"
            @state-change="onEditorStateChange"
            @editor-ready="onEditorReady"
            @cancel="onEditorCancel"
          />
          <div v-else-if="workspaceContentReady" class="empty-work">
            <p v-if="!workspaceId || !projectId" class="muted empty-work-hint">
              {{ t('create.needProjectFirst') }}
            </p>
            <div v-else class="empty-work-brand">
              <img class="empty-work-logo" src="/nest.png" alt="" draggable="false" />
              <span class="empty-work-title">{{ t('app.brandName') }}</span>
            </div>
          </div>
        </div>

        <div
          v-if="codePanelOpen"
          class="code-resizer"
          :class="{ 'code-resizer--active': codePanelDragging }"
          @mousedown="startCodePanelDrag"
        />

        <CodeSnippetPanel
          v-if="codePanelOpen"
          :width="codePanelWidth"
          :curl-text="curlText"
          @close="closeCodePanel"
        />
      </div>
    </main>

    <NameDialog
      v-model:open="projectDialogOpen"
      :title="projectDialogTitle"
      :label="t('project.label')"
      :initial="projectDialogInitial"
      :anchor="dialogAnchor"
      @confirm="confirmProjectName"
    />
    <NameDialog
      v-model:open="folderDialogOpen"
      :title="folderDialogTitle"
      :initial="folderDialogInitial"
      :anchor="dialogAnchor"
      @confirm="confirmCreateFolder"
    />
    <NameDialog
      v-model:open="renameDialogOpen"
      :title="t('tree.rename')"
      :initial="renameDialogInitial"
      :anchor="dialogAnchor"
      @confirm="confirmRename"
    />
    <ConfirmDialog
      v-model:open="deleteDialogOpen"
      :title="deleteDialogTitle"
      :message="deleteDialogMessage"
      :anchor="dialogAnchor"
      @confirm="confirmDelete"
    />
    <ConfirmDialog
      v-model:open="projectDeleteOpen"
      :title="t('project.delete')"
      :message="t('project.deleteConfirm', { name: projectDeleteTarget?.name || '' })"
      :anchor="dialogAnchor"
      @confirm="confirmDeleteProject"
    />
    <ConfirmDialog
      v-model:open="saveBeforeCloseOpen"
      :title="t('request.saveBeforeCloseTitle')"
      :message="t('request.saveBeforeClose')"
      variant="primary"
      :confirm-label="t('common.confirm')"
      :close-on-confirm="false"
      @confirm="confirmSaveBeforeClose"
      @cancel="discardSaveBeforeClose"
      @dismiss="abortSaveBeforeClose"
    />
    <EnvironmentManageDialog v-model:open="envManageOpen" />
    <FolderSettingsDialog
      v-model:open="folderSettingsOpen"
      :folder-id="folderSettingsId"
      @saved="reloadTree"
    />
    <ImportApiDialog
      ref="importDialogRef"
      v-model:open="importDialogOpen"
      @submit="onImportSubmit"
    />

    <Teleport to="body">
      <div
        v-if="tabMenu.visible"
        class="req-tab-menu"
        :style="{ left: `${tabMenu.x}px`, top: `${tabMenu.y}px` }"
        @click.stop
        @contextmenu.prevent
      >
        <button
          type="button"
          class="req-tab-menu-item"
          :disabled="tabs.length <= 1"
          @click="runTabMenuAction('others')"
        >
          {{ t('request.tabCloseOthers') }}
        </button>
        <button
          type="button"
          class="req-tab-menu-item"
          :disabled="tabMenuIndex <= 0"
          @click="runTabMenuAction('left')"
        >
          {{ t('request.tabCloseLeft') }}
        </button>
        <button
          type="button"
          class="req-tab-menu-item"
          :disabled="tabMenuIndex < 0 || tabMenuIndex >= tabs.length - 1"
          @click="runTabMenuAction('right')"
        >
          {{ t('request.tabCloseRight') }}
        </button>
        <button type="button" class="req-tab-menu-item" @click="runTabMenuAction('all')">
          {{ t('request.tabCloseAll') }}
        </button>
      </div>
    </Teleport>

    <Teleport to="body">
      <div
        v-if="treeBlankMenu.visible"
        class="req-tab-menu"
        :style="{ left: `${treeBlankMenu.x}px`, top: `${treeBlankMenu.y}px` }"
        @click.stop
        @contextmenu.prevent
      >
        <button type="button" class="req-tab-menu-item" @click="runTreeBlankMenuAction('add-api')">
          {{ t('tree.addApi') }}
        </button>
        <button type="button" class="req-tab-menu-item" @click="runTreeBlankMenuAction('add-folder')">
          {{ t('tree.addFolder') }}
        </button>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useWorkspaceContext } from '../composables/useWorkspaceContext'
import TreeList from '../components/TreeList.vue'
import SelectPicker from '../components/SelectPicker.vue'
import NameDialog from '../components/NameDialog.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import EnvironmentManageDialog from '../components/EnvironmentManageDialog.vue'
import ImportApiDialog from '../components/ImportApiDialog.vue'
import SplitCreateButton from '../components/SplitCreateButton.vue'
import InterfaceEditor from '../components/InterfaceEditor.vue'
import CodeSnippetPanel from '../components/CodeSnippetPanel.vue'
import FolderSettingsDialog from '../components/FolderSettingsDialog.vue'
import { useToast } from '../composables/useToast'
import { disposeAllWsTabs, disposeWsTab } from '../composables/wsSessionRegistry'
import type { TreeAction, TreeApiDragPayload } from '../components/TreeNode.vue'
import type { HeaderPair, InterfaceRow, SendResponse, TreeItem } from '../types'
import { useSaveShortcut } from '../composables/useSaveShortcut'
import { useCloseTabShortcut } from '../composables/useCloseTabShortcut'
import { anchorFromEvent, type DialogAnchor } from '../composables/useDialogAnchor'
import { useDragResize } from '../composables/useDragResize'
import { buildCurlFromTabState } from '../utils/build-curl'
import { safeDownloadName } from '../utils/download-json'

const TAB_LAYOUT_KEY = 'request_tabs_layout'

type StoredTabLayout = {
  workspaceId: number
  projectId: number
  activeInterfaceId: number | null
  openInterfaceIds: number[]
}

type RequestTab = {
  id: string
  interfaceId: number | null
  folderId: number | null
  name: string
  method: string
  url: string
  headers: HeaderPair[]
  body: string
  dirty: boolean
  snapshot: string
  mode: 'create' | 'edit' | 'blank'
  detail: Record<string, unknown> | null
  lastResponse: SendResponse | null
  requestPaneTab: 'params' | 'authorization' | 'headers' | 'body' | 'message' | 'metadata'
  responseScrollTop: number
}

function cloneSendResponse(res: SendResponse | null | undefined): SendResponse | null {
  if (!res) return null
  return {
    ...res,
    headers: (res.headers ?? []).map((h) => ({ ...h })),
  }
}

/** Drop base64 file payloads from form-data detail so tab switches stay snappy. */
function stripFormDataFilePayloads<T>(detail: T): T {
  if (!detail || typeof detail !== 'object') return detail
  const root = detail as Record<string, unknown>
  const body = root.request_body
  if (!body || typeof body !== 'object' || Array.isArray(body)) return detail
  const fields = (body as { fields?: unknown }).fields
  if (!Array.isArray(fields)) return detail
  return {
    ...root,
    request_body: {
      ...(body as Record<string, unknown>),
      fields: fields.map((field) => {
        if (!field || typeof field !== 'object' || Array.isArray(field)) return field
        const next = { ...(field as Record<string, unknown>) }
        delete next.file_data
        delete next.fileData
        return next
      }),
    },
  } as T
}

const { t } = useI18n()
const { showToast } = useToast()
const {
  size: sidebarWidth,
  dragging: sidebarDragging,
  startDrag: startSidebarDrag,
} = useDragResize({
  axis: 'x',
  min: 200,
  max: 480,
  initial: 280,
  storageKey: 'main_sidebar_width',
})
const workRowRef = ref<HTMLElement | null>(null)
const {
  size: codePanelWidth,
  dragging: codePanelDragging,
  startDrag: startCodePanelDragBase,
} = useDragResize({
  axis: 'x',
  min: 260,
  max: 720,
  initial: 360,
  invert: true,
  storageKey: 'code_snippet_panel_width',
})
const codePanelOpen = ref(false)
const curlText = ref('')
const {
  projects,
  environments,
  workspaceId,
  projectId,
  environmentId,
  envVarsRevision,
  initWorkspaceContext,
  selectProject,
  selectEnvironment,
  loadProjects,
  refreshAfterSync,
} = useWorkspaceContext()

const tree = ref<TreeItem[]>([])
const search = ref('')
/** Folder ids currently expanded in the sidebar tree (array for reliable Vue updates). */
const expandedFolderIds = ref<number[]>([])
const dragApiState = ref<{ apiId: number; folderId: number } | null>(null)
const dragOverKey = ref<string | null>(null)
const reorderingApis = ref(false)
const canDragApis = computed(() => !search.value.trim() && !reorderingApis.value)
const activeInterfaceId = ref<number | null>(null)
const tabsEl = ref<HTMLElement | null>(null)
const projectDialogOpen = ref(false)
const projectDialogMode = ref<'create' | 'edit'>('create')
const projectDialogInitial = ref('')
const projectEditTarget = ref<{ id: number; name: string } | null>(null)
const projectDeleteOpen = ref(false)
const projectDeleteTarget = ref<{ id: number; name: string } | null>(null)
const projectDialogTitle = computed(() =>
  projectDialogMode.value === 'edit' ? t('project.rename') : t('create.project'),
)
const envManageOpen = ref(false)
const folderDialogOpen = ref(false)
const folderParentId = ref(0)
const folderDialogTitle = ref('')
const folderDialogInitial = ref('')
const renameDialogOpen = ref(false)
const renameDialogInitial = ref('')
const renameTarget = ref<TreeAction | null>(null)
const folderSettingsOpen = ref(false)
const folderSettingsId = ref<number | null>(null)
const deleteDialogOpen = ref(false)
const deleteDialogTitle = ref('')
const deleteDialogMessage = ref('')
const deleteTarget = ref<TreeAction | null>(null)
const saveBeforeCloseOpen = ref(false)
const pendingCloseTabId = ref('')
const dialogAnchor = ref<DialogAnchor | null>(null)
const editorInitial = ref<Record<string, unknown> | null>(null)
const editorKey = ref(0)
const editorRef = ref<InstanceType<typeof InterfaceEditor> | null>(null)
const importDialogOpen = ref(false)
const importDialogRef = ref<{ setSubmitting: (value: boolean) => void } | null>(null)

const tabs = ref<RequestTab[]>([])
const activeTabId = ref('')
const tabMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  tabId: '',
})
const treeBlankMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
})

const reqName = ref('')
const reqMethod = ref('GET')
const reqUrl = ref('')
const reqHeaders = ref<HeaderPair[]>(defaultHeaders())
const reqBody = ref('')

const activeTab = computed(() => tabs.value.find((x) => x.id === activeTabId.value) ?? null)
const editorMode = computed(() => (activeTab.value?.mode === 'create' ? 'create' : 'edit'))
const editorReady = computed(
  () =>
    !!workspaceId.value &&
    !!projectId.value &&
    (activeTab.value?.mode === 'create' || activeTab.value?.mode === 'edit'),
)
const tabMenuIndex = computed(() => tabs.value.findIndex((x) => x.id === tabMenu.tabId))

useSaveShortcut(() => editorRef.value?.save(), () => editorReady.value)
useCloseTabShortcut(
  () => {
    const tab = activeTab.value
    if (tab) requestCloseTab(tab.id)
  },
  () => editorReady.value && !!activeTab.value && !saveBeforeCloseOpen.value,
)

let tabSeq = 1
let restoringTabs = false

function defaultHeaders(): HeaderPair[] {
  return []
}

function editorSnapshot(): string {
  return editorRef.value?.getSnapshot() ?? ''
}

function tabIsDirty(tab: RequestTab, snap: string): boolean {
  if (tab.mode === 'create') return true
  if (editorRef.value?.isDirty?.()) return true
  return !!tab.snapshot && snap !== tab.snapshot
}

function tabStateFromEditor(tab: RequestTab): RequestTab | null {
  if (!editorRef.value) return null
  const state = editorRef.value.exportTabState()
  const snap = editorSnapshot()
  return {
    ...tab,
    name: state.name,
    method: state.method,
    url: state.url,
    folderId:
      state.folderId != null && Number.isFinite(Number(state.folderId))
        ? Number(state.folderId)
        : tab.folderId,
    detail: stripFormDataFilePayloads(state.detail),
    headers: defaultHeaders(),
    body: String((state.detail as { request_body?: { raw?: string } }).request_body?.raw ?? ''),
    interfaceId: activeInterfaceId.value,
    lastResponse: cloneSendResponse(state.lastResponse),
    requestPaneTab: state.requestPaneTab ?? tab.requestPaneTab ?? 'body',
    responseScrollTop:
      typeof state.responseScrollTop === 'number' && Number.isFinite(state.responseScrollTop)
        ? Math.max(0, state.responseScrollTop)
        : (tab.responseScrollTop ?? 0),
    dirty: tabIsDirty(tab, snap),
    snapshot: tab.snapshot,
  }
}

function replaceActiveTab(next: RequestTab) {
  const idx = tabs.value.findIndex((x) => x.id === next.id)
  if (idx >= 0) tabs.value[idx] = next
}

function syncTabSnapshotFromEditor() {
  const tab = activeTab.value
  const editor = editorRef.value
  if (!tab || !editor) return
  const snap = editorSnapshot()
  if (tab.snapshot) {
    editor.setBaseline(tab.snapshot)
    replaceActiveTab(tabStateFromEditor(tab) ?? tab)
    return
  }
  editor.syncBaseline()
  const baseline = editor.getSnapshot() || snap
  replaceActiveTab({
    ...(tabStateFromEditor({ ...tab, snapshot: baseline }) ?? tab),
    snapshot: baseline,
    dirty: tab.mode === 'create',
  })
}

function createApiTab(folderId?: number): RequestTab {
  const data = {
    name: t('create.api'),
    method: 'POST',
    url: '',
    headers: defaultHeaders(),
    body: '',
  }
  const resolvedFolderId = folderId ?? 0
  return {
    id: `tab-${tabSeq++}`,
    interfaceId: null,
    folderId: resolvedFolderId,
    ...data,
    dirty: true,
    snapshot: '',
    mode: 'create',
    detail: { folder_id: resolvedFolderId },
    lastResponse: null,
    requestPaneTab: 'body',
    responseScrollTop: 0,
  }
}

function applyTabToForm(tab: RequestTab, options?: { remount?: boolean; skipReveal?: boolean }) {
  reqName.value = tab.name
  reqMethod.value = tab.method
  reqUrl.value = tab.url
  reqHeaders.value = tab.headers.map((h) => ({ ...h }))
  reqBody.value = tab.body
  activeInterfaceId.value = tab.interfaceId
  const folderId =
    tab.folderId ??
    (typeof (tab.detail as { folder_id?: unknown } | null)?.folder_id === 'number'
      ? (tab.detail as { folder_id: number }).folder_id
      : null)
  editorInitial.value = {
    name: tab.name,
    method: tab.method,
    url: tab.url,
    folder_id: folderId,
    status: (tab.detail as { status?: number } | null)?.status ?? 3,
    detail: stripFormDataFilePayloads({
      ...(tab.detail ?? {}),
      ...(folderId != null ? { folder_id: folderId } : {}),
    }),
    lastResponse: cloneSendResponse(tab.lastResponse),
    requestPaneTab: tab.requestPaneTab ?? 'body',
    responseScrollTop: tab.responseScrollTop ?? 0,
  }
  // Remount only when explicitly requested. Default keep-alive avoids flashing
  // method/url/name/folder/body while the editor tears down and remounts.
  if (options?.remount === true) {
    editorKey.value += 1
  }
  // During bulk tab restore, reveal once at the end for the active tab.
  if (!restoringTabs && !options?.skipReveal && tab.interfaceId != null) {
    void revealApiInTree(tab.interfaceId, { folderId: tab.folderId ?? folderId })
  }
}

function onEditorReady() {
  void nextTick()
    .then(() => nextTick())
    .then(() => {
      syncTabSnapshotFromEditor()
      refreshCurlText()
    })
}

async function refreshCurlText() {
  if (!codePanelOpen.value) return
  try {
    const state = editorRef.value?.exportTabState?.() ?? null
    let vars: Record<string, string> = {}
    if (environmentId.value) {
      try {
        const rows = await invoke<[string, string][]>('get_env_vars', {
          environmentId: environmentId.value,
        })
        vars = Object.fromEntries(rows)
      } catch {
        vars = {}
      }
    }
    curlText.value = buildCurlFromTabState(state, vars)
  } catch {
    curlText.value = ''
  }
}

function toggleCodePanel() {
  codePanelOpen.value = !codePanelOpen.value
  if (codePanelOpen.value) {
    void nextTick().then(refreshCurlText)
  }
}

function closeCodePanel() {
  codePanelOpen.value = false
}

function startCodePanelDrag(e: MouseEvent) {
  const bound = workRowRef.value?.getBoundingClientRect().width ?? 900
  const max = Math.max(260, Math.min(720, Math.round(bound * 0.65)))
  startCodePanelDragBase(e, max)
}

function clearTabs() {
  void disposeAllWsTabs()
  tabs.value = []
  activeTabId.value = ''
  activeInterfaceId.value = null
  editorInitial.value = null
  expandedFolderIds.value = []
}

async function persistTabLayout() {
  if (restoringTabs || !workspaceId.value || !projectId.value) return
  const openInterfaceIds = tabs.value
    .filter((tab) => tab.mode === 'edit' && tab.interfaceId != null)
    .map((tab) => tab.interfaceId as number)
  const layout: StoredTabLayout = {
    workspaceId: workspaceId.value,
    projectId: projectId.value,
    activeInterfaceId: activeTab.value?.interfaceId ?? null,
    openInterfaceIds,
  }
  try {
    await invoke('set_setting_value', {
      key: TAB_LAYOUT_KEY,
      value: JSON.stringify(layout),
    })
  } catch {
    // browser preview without Tauri
  }
}

async function restoreTabLayout() {
  if (!workspaceId.value || !projectId.value) return
  let layout: StoredTabLayout | null = null
  try {
    const raw = await invoke<string | null>('get_setting_value', { key: TAB_LAYOUT_KEY })
    if (raw) layout = JSON.parse(raw) as StoredTabLayout
  } catch {
    return
  }
  if (
    !layout ||
    Number(layout.workspaceId) !== Number(workspaceId.value) ||
    Number(layout.projectId) !== Number(projectId.value) ||
    !Array.isArray(layout.openInterfaceIds) ||
    !layout.openInterfaceIds.length
  ) {
    return
  }

  restoringTabs = true
  let revealId: number | null = null
  let revealFolderId: number | null = null
  try {
    clearTabs()
    for (const id of layout.openInterfaceIds) {
      try {
        await onSelectApi(Number(id), { restore: true })
      } catch {
        // Interface may have been deleted — skip and keep restoring the rest.
      }
    }
    const activeId =
      layout.activeInterfaceId != null ? Number(layout.activeInterfaceId) : null
    if (activeId != null) {
      const tab = tabs.value.find((x) => x.interfaceId === activeId)
      if (tab) {
        if (tab.id !== activeTabId.value) {
          // applyTabToForm skips reveal while restoringTabs — reveal after.
          persistActiveTab()
          activeTabId.value = tab.id
          applyTabToForm(tab, { skipReveal: true })
        }
        revealId = activeId
        revealFolderId = tab.folderId
      }
    } else if (activeInterfaceId.value != null) {
      revealId = activeInterfaceId.value
      revealFolderId = activeTab.value?.folderId ?? null
    }
  } finally {
    restoringTabs = false
    await persistTabLayout()
  }
  // Reveal after restore flag clears and the tree has painted.
  if (revealId != null) {
    await nextTick()
    await revealApiInTree(revealId, { folderId: revealFolderId })
  }
}

async function reloadTree() {
  if (!workspaceId.value || !projectId.value) {
    tree.value = []
    return
  }
  try {
    tree.value = await invoke('get_local_tree', {
      workspaceId: workspaceId.value,
      projectId: projectId.value,
      search: search.value || null,
    })
  } catch {
    tree.value = []
  }
}

function findFolderNode(nodes: TreeItem[], folderId: number): Extract<TreeItem, { type: 'folder' }> | null {
  for (const node of nodes) {
    if (node.type === 'folder') {
      if (node.id === folderId) return node
      const hit = findFolderNode(node.children, folderId)
      if (hit) return hit
    }
  }
  return null
}

/** Ancestor folder ids from root → parent of the API (nested folders included). */
function findFolderPathToApi(
  nodes: TreeItem[],
  apiId: number,
  trail: number[] = [],
): number[] | null {
  const target = Number(apiId)
  for (const node of nodes) {
    if (node.type === 'api' && Number(node.id) === target) return trail
    if (node.type === 'folder') {
      const found = findFolderPathToApi(node.children, target, [...trail, Number(node.id)])
      if (found) return found
    }
  }
  return null
}

/** Path from root → folder (inclusive), used when the API row is filtered out of the tree. */
function findFolderPathToFolder(
  nodes: TreeItem[],
  folderId: number,
  trail: number[] = [],
): number[] | null {
  const target = Number(folderId)
  if (!Number.isFinite(target) || target === 0) return []
  for (const node of nodes) {
    if (node.type !== 'folder') continue
    const nextTrail = [...trail, Number(node.id)]
    if (Number(node.id) === target) return nextTrail
    const found = findFolderPathToFolder(node.children, target, nextTrail)
    if (found) return found
  }
  return null
}

function resolveExpandPath(apiId: number, folderId?: number | null): number[] {
  const byApi = findFolderPathToApi(tree.value, apiId)
  if (byApi) return byApi
  if (folderId != null && Number(folderId) !== 0) {
    return findFolderPathToFolder(tree.value, Number(folderId)) ?? []
  }
  return []
}

function toggleFolderExpand(folderId: number) {
  const id = Number(folderId)
  if (expandedFolderIds.value.some((x) => Number(x) === id)) {
    expandedFolderIds.value = expandedFolderIds.value.filter((x) => Number(x) !== id)
  } else {
    expandedFolderIds.value = [...expandedFolderIds.value, id]
  }
}

function applyExpandedPath(path: number[]) {
  if (!path.length) return
  const next = new Set(expandedFolderIds.value.map(Number))
  for (const id of path) next.add(Number(id))
  expandedFolderIds.value = [...next]
}

function expandFoldersToApi(apiId: number, folderId?: number | null) {
  applyExpandedPath(resolveExpandPath(apiId, folderId))
}

function sleep(ms: number) {
  return new Promise<void>((resolve) => {
    window.setTimeout(resolve, ms)
  })
}

/** Nested folders mount only after parents open — wait until the row exists. */
async function waitForApiRow(apiId: number, timeoutMs = 1500): Promise<HTMLElement | null> {
  const selector = `.tree-wrap .row.api[data-api-id="${apiId}"]`
  const deadline = performance.now() + timeoutMs
  while (performance.now() < deadline) {
    const el = document.querySelector(selector) as HTMLElement | null
    if (el) return el
    await nextTick()
    await sleep(16)
  }
  return document.querySelector(selector) as HTMLElement | null
}

async function expandFoldersToApiAnimated(apiId: number, folderId?: number | null) {
  const path = resolveExpandPath(apiId, folderId)
  if (!path.length) return
  for (const id of path) {
    if (expandedFolderIds.value.some((x) => Number(x) === Number(id))) continue
    expandedFolderIds.value = [...expandedFolderIds.value, Number(id)]
    // Nested TreeNodes only exist after the parent opens — flush before next level.
    await nextTick()
    await nextTick()
    // Match TreeNode FOLDER_MOTION_MS so nested levels cascade cleanly.
    await sleep(180)
  }
  // Guarantee full path in case a tick was skipped.
  applyExpandedPath(path)
  await nextTick()
}

/** Scroll inside `.tree-wrap` — scrollIntoView is unreliable with nested overflow. */
function scrollApiRowIntoView(el: HTMLElement, animated: boolean) {
  const container = el.closest('.tree-wrap') as HTMLElement | null
  if (!container) {
    el.scrollIntoView({ block: 'center', behavior: animated ? 'smooth' : 'auto' })
    return
  }
  const containerRect = container.getBoundingClientRect()
  const elRect = el.getBoundingClientRect()
  const elCenter = elRect.top + elRect.height / 2
  const containerCenter = containerRect.top + containerRect.height / 2
  const nextTop = container.scrollTop + (elCenter - containerCenter)
  container.scrollTo({
    top: Math.max(0, nextTop),
    behavior: animated ? 'smooth' : 'auto',
  })
}

async function revealApiInTree(
  apiId: number | null,
  options?: { animated?: boolean; folderId?: number | null },
) {
  if (apiId == null) return
  const animated = options?.animated === true
  const folderId = options?.folderId
  if (animated) {
    await expandFoldersToApiAnimated(apiId, folderId)
  } else {
    expandFoldersToApi(apiId, folderId)
    await nextTick()
    await nextTick()
  }

  const el = await waitForApiRow(apiId)
  if (!el) return

  // Let enter transition settle so layout height is final before measuring scroll.
  if (animated) await sleep(40)
  scrollApiRowIntoView(el, animated)

  if (!animated) return
  el.classList.remove('reveal-flash')
  void el.offsetWidth
  el.classList.add('reveal-flash')
  window.setTimeout(() => el.classList.remove('reveal-flash'), 900)
}

/** Double-click tab (except close ×) → expand ancestors and scroll the API into view. */
function onTabDblClick(tab: RequestTab, event: MouseEvent) {
  const target = event.target as HTMLElement | null
  if (target?.closest('.req-tab-close')) return
  if (tab.interfaceId == null) return
  if (tab.id !== activeTabId.value) {
    persistActiveTab()
    activeTabId.value = tab.id
    applyTabToForm(tab, { skipReveal: true })
    void persistTabLayout()
    void nextTick().then(refreshCurlText)
  }
  void revealApiInTree(tab.interfaceId, {
    animated: true,
    folderId: tab.folderId,
  })
}

function getSiblingItems(folderId: number): TreeItem[] {
  if (folderId === 0) return tree.value
  return findFolderNode(tree.value, folderId)?.children ?? []
}

function getApiIdsInFolder(folderId: number): number[] {
  return getSiblingItems(folderId)
    .filter((item): item is Extract<TreeItem, { type: 'api' }> => item.type === 'api')
    .map((item) => item.id)
}

function takeApiNode(
  folderId: number,
  apiId: number,
): Extract<TreeItem, { type: 'api' }> | null {
  const siblings = getSiblingItems(folderId)
  const idx = siblings.findIndex((item) => item.type === 'api' && item.id === apiId)
  if (idx < 0) return null
  const [node] = siblings.splice(idx, 1)
  if (!node || node.type !== 'api') return null
  if (folderId === 0) {
    tree.value = [...siblings]
  } else {
    const folder = findFolderNode(tree.value, folderId)
    if (folder) folder.children = [...siblings]
    tree.value = [...tree.value]
  }
  return node
}

function insertApiNode(
  folderId: number,
  apiNode: Extract<TreeItem, { type: 'api' }>,
  afterApiId: number | null,
) {
  const siblings = [...getSiblingItems(folderId)]
  const subFolders = siblings.filter((item) => item.type === 'folder')
  const apis = siblings.filter(
    (item): item is Extract<TreeItem, { type: 'api' }> => item.type === 'api',
  )
  let insertIndex = apis.length
  if (afterApiId != null) {
    const idx = apis.findIndex((item) => item.id === afterApiId)
    if (idx >= 0) insertIndex = idx + 1
  }
  apis.splice(insertIndex, 0, apiNode)
  const next = [...subFolders, ...apis]
  if (folderId === 0) {
    tree.value = next
    return
  }
  const folder = findFolderNode(tree.value, folderId)
  if (!folder) return
  folder.children = next
  tree.value = [...tree.value]
}

function reorderApiTreeLocal(folderId: number, orderedApiIds: number[]) {
  const siblings = getSiblingItems(folderId)
  const subFolders = siblings.filter((item) => item.type === 'folder')
  const apiMap = new Map(
    siblings
      .filter((item): item is Extract<TreeItem, { type: 'api' }> => item.type === 'api')
      .map((item) => [item.id, item]),
  )
  const apis = orderedApiIds
    .map((id) => apiMap.get(id))
    .filter((item): item is Extract<TreeItem, { type: 'api' }> => !!item)
  if (apis.length !== orderedApiIds.length) return

  const next = [...subFolders, ...apis]
  if (folderId === 0) {
    tree.value = next
    return
  }
  const folder = findFolderNode(tree.value, folderId)
  if (!folder) return
  folder.children = next
  tree.value = [...tree.value]
}

function resetApiDragState() {
  dragApiState.value = null
  dragOverKey.value = null
}

function onApiDragStart(payload: TreeApiDragPayload) {
  if (!canDragApis.value) {
    resetApiDragState()
    return
  }
  dragApiState.value = { apiId: payload.apiId, folderId: payload.folderId }
  dragOverKey.value = null
}

function onApiDragOver(payload: TreeApiDragPayload) {
  const drag = dragApiState.value
  if (!drag || !payload.targetKey) return
  if (payload.targetKind === 'api' && payload.targetApiId === drag.apiId) return
  dragOverKey.value = payload.targetKey
}

function onApiDragLeave() {
  dragOverKey.value = null
}

function buildOrderedIds(
  folderId: number,
  draggedApiId: number,
  afterApiId: number | null,
  isCrossFolder: boolean,
): number[] {
  const base = isCrossFolder
    ? getApiIdsInFolder(folderId)
    : getApiIdsInFolder(folderId).filter((id) => id !== draggedApiId)

  // Drop indicator is below the target row — insert after target (web convention).
  let insertIndex = base.length
  if (afterApiId != null) {
    const idx = base.indexOf(afterApiId)
    if (idx >= 0) insertIndex = idx + 1
  }
  const next = [...base]
  next.splice(insertIndex, 0, draggedApiId)
  return next
}

async function onApiDrop(payload: TreeApiDragPayload) {
  const drag = dragApiState.value
  if (!drag || !workspaceId.value || !projectId.value) {
    resetApiDragState()
    return
  }

  const toFolderId = payload.folderId
  const afterApiId = payload.targetApiId ?? null
  if (payload.targetKind === 'api' && afterApiId === drag.apiId) {
    resetApiDragState()
    return
  }

  const fromFolderId = drag.folderId
  const isCrossFolder = fromFolderId !== toFolderId
  const targetOrderedIds = buildOrderedIds(
    toFolderId,
    drag.apiId,
    afterApiId,
    isCrossFolder,
  )
  const sourceOrderedIds = isCrossFolder
    ? getApiIdsInFolder(fromFolderId).filter((id) => id !== drag.apiId)
    : []

  // No-op same-folder same position.
  if (!isCrossFolder) {
    const prev = getApiIdsInFolder(fromFolderId)
    if (prev.length === targetOrderedIds.length && prev.every((id, i) => id === targetOrderedIds[i])) {
      resetApiDragState()
      return
    }
  }

  resetApiDragState()
  reorderingApis.value = true

  // Optimistic tree update.
  if (isCrossFolder) {
    const node = takeApiNode(fromFolderId, drag.apiId)
    if (node) insertApiNode(toFolderId, node, afterApiId)
  } else {
    reorderApiTreeLocal(toFolderId, targetOrderedIds)
  }

  try {
    if (isCrossFolder) {
      await invoke('move_reorder_local_interfaces', {
        args: {
          workspace_id: workspaceId.value,
          project_id: projectId.value,
          interface_id: drag.apiId,
          from_folder_id: fromFolderId,
          to_folder_id: toFolderId,
          target_interface_ids: targetOrderedIds,
          source_interface_ids: sourceOrderedIds,
        },
      })
    } else {
      await invoke('reorder_local_interfaces', {
        args: {
          workspace_id: workspaceId.value,
          project_id: projectId.value,
          folder_id: toFolderId,
          interface_ids: targetOrderedIds,
        },
      })
    }
  } catch (e) {
    await reloadTree()
    showToast(String(e) || t('tree.reorderApiFailed'), undefined, 'error')
  } finally {
    reorderingApis.value = false
  }
}

let lastWorkspaceBootKey = ''
let workspaceBootInFlight: Promise<void> | null = null
/** False until the first workspace bootstrap finishes (avoids empty-home flash). */
const workspaceContentReady = ref(false)

function markSessionReady() {
  if (workspaceContentReady.value) return
  workspaceContentReady.value = true
  window.__NEST_MARK_SESSION_READY__?.()
  window.__NEST_MARK_SESSION_READY__ = undefined
}

async function ensureWorkspaceBootstrapped() {
  const ws = workspaceId.value
  const proj = projectId.value
  if (!ws || !proj) {
    tree.value = []
    // Do not mark ready here — project may still be assigned during init.
    return
  }

  const key = `${ws}:${proj}`
  if (workspaceBootInFlight) {
    await workspaceBootInFlight
    if (lastWorkspaceBootKey === key) return
  }

  workspaceBootInFlight = (async () => {
    const prevKey = lastWorkspaceBootKey
    if (key === lastWorkspaceBootKey) {
      await reloadTree()
      return
    }
    lastWorkspaceBootKey = key
    await reloadTree()
    if (!prevKey) {
      // First ready workspace/project in this session — restore last tabs.
      await restoreTabLayout()
      return
    }
    // Switched workspace or project — drop old tabs, restore if saved for this pair.
    clearTabs()
    await restoreTabLayout()
    await persistTabLayout()
  })()

  try {
    await workspaceBootInFlight
  } finally {
    workspaceBootInFlight = null
  }
}

function clearTabsAndPersist() {
  clearTabs()
  void persistTabLayout()
}

function persistActiveTab() {
  const tab = activeTab.value
  if (!tab) return
  const next = tabStateFromEditor(tab)
  if (next) replaceActiveTab(next)
}

function onEditorStateChange() {
  persistActiveTab()
  refreshCurlText()
}

function activateTab(id: string) {
  if (id === activeTabId.value) return
  persistActiveTab()
  activeTabId.value = id
  const tab = tabs.value.find((x) => x.id === id)
  if (tab) applyTabToForm(tab)
  void persistTabLayout()
  void nextTick().then(refreshCurlText)
}

function addTab() {
  openCreateApi()
}

function setDialogAnchor(event?: MouseEvent, anchor?: DialogAnchor | null) {
  if (anchor) {
    dialogAnchor.value = anchor
    return
  }
  if (event) {
    dialogAnchor.value = anchorFromEvent(event)
  }
}

function openCreateProject(event: MouseEvent) {
  if (!workspaceId.value) {
    showToast(t('create.needWorkspaceFirst'), 1500, 'warning')
    return
  }
  setDialogAnchor(event)
  projectDialogMode.value = 'create'
  projectDialogInitial.value = ''
  projectEditTarget.value = null
  projectDialogOpen.value = true
}

function openEditProject(item: { id: number; name: string }, event: MouseEvent) {
  setDialogAnchor(event)
  projectDialogMode.value = 'edit'
  projectDialogInitial.value = item.name
  projectEditTarget.value = item
  projectDialogOpen.value = true
}

function openDeleteProject(item: { id: number; name: string }, event: MouseEvent) {
  setDialogAnchor(event)
  projectDeleteTarget.value = item
  projectDeleteOpen.value = true
}

function openCreateFolder(
  parentId = 0,
  event?: MouseEvent,
  title?: string,
) {
  if (!workspaceId.value || !projectId.value) {
    showToast(t('create.needProjectFirst'), 1500, 'warning')
    return
  }
  setDialogAnchor(event)
  folderParentId.value = parentId
  folderDialogTitle.value =
    title ?? (parentId ? t('tree.addSubFolder') : t('create.folder'))
  folderDialogInitial.value = ''
  folderDialogOpen.value = true
}

async function openCreateApi(folderId?: number) {
  if (!workspaceId.value || !projectId.value) {
    showToast(t('create.needProjectFirst'), 1500, 'warning')
    return
  }
  // folderId 0 / omitted = project root (no folder required)
  const targetFolderId = folderId ?? 0
  persistActiveTab()
  const tab = createApiTab(targetFolderId)
  tabs.value.push(tab)
  activeTabId.value = tab.id
  applyTabToForm(tab)
  void persistTabLayout()
}

async function confirmProjectName(name: string) {
  if (!workspaceId.value) return
  try {
    if (projectDialogMode.value === 'edit' && projectEditTarget.value) {
      await invoke('update_local_project', {
        args: {
          id: projectEditTarget.value.id,
          workspace_id: workspaceId.value,
          name,
        },
      })
      projectDialogOpen.value = false
      projectEditTarget.value = null
      await loadProjects()
      showToast(t('project.updateSuccess'), 1500, 'success')
      return
    }
    const row = await invoke<{ id: number }>('create_local_project', {
      args: { workspace_id: workspaceId.value, name },
    })
    projectDialogOpen.value = false
    await loadProjects()
    await selectProject(row.id)
    await reloadTree()
  } catch (e) {
    showToast(String(e), 2200, 'error')
  }
}

async function confirmDeleteProject() {
  if (!workspaceId.value || !projectDeleteTarget.value) return
  try {
    await invoke('delete_local_project', {
      workspaceId: workspaceId.value,
      projectId: projectDeleteTarget.value.id,
    })
    projectDeleteOpen.value = false
    projectDeleteTarget.value = null
    await refreshAfterSync()
    await reloadTree()
    showToast(t('project.deleteSuccess'), 1500, 'success')
  } catch (e) {
    showToast(String(e), 2200, 'error')
  }
}

async function confirmCreateFolder(name: string) {
  if (!workspaceId.value || !projectId.value) return
  try {
    await invoke('create_local_folder', {
      args: {
        workspace_id: workspaceId.value,
        project_id: projectId.value,
        parent_id: folderParentId.value,
        name,
      },
    })
    folderDialogOpen.value = false
    await reloadTree()
  } catch (e) {
    alert(String(e))
  }
}

function currentProjectName() {
  return projects.value.find((item) => item.id === projectId.value)?.name || 'apinest'
}

function openImportDialog() {
  if (!workspaceId.value || !projectId.value) {
    showToast(t('importExport.needProject'), 1500, 'warning')
    return
  }
  importDialogOpen.value = true
}

async function exportCurrentProject() {
  await runExport()
}

async function exportFolder(folderId: number, folderName: string) {
  await runExport(folderId, folderName)
}

async function runExport(folderId?: number, folderName?: string) {
  if (!workspaceId.value || !projectId.value) {
    showToast(t('importExport.needProject'), 1500, 'warning')
    return
  }
  try {
    const doc = await invoke<unknown>('export_apinest', {
      args: {
        workspace_id: workspaceId.value,
        project_id: projectId.value,
        folder_id: folderId ?? null,
      },
    })
    const base = folderName || currentProjectName()
    const saved = await invoke<boolean>('save_json_file', {
      args: {
        default_name: `${safeDownloadName(base)}.apinest.json`,
        document: doc,
      },
    })
    if (saved) {
      showToast(t('importExport.exportSuccess'), 1500, 'success')
    }
  } catch (e) {
    showToast(String(e), 1800, 'error')
  }
}

async function onImportSubmit(payload: { source: string; document: unknown }) {
  if (!workspaceId.value || !projectId.value) {
    showToast(t('importExport.needProject'), 1500, 'warning')
    return
  }
  importDialogRef.value?.setSubmitting(true)
  try {
    const result = await invoke<{ folders: number; apis: number }>('import_apinest', {
      args: {
        workspace_id: workspaceId.value,
        project_id: projectId.value,
        source: payload.source,
        document: payload.document,
      },
    })
    try {
      await invoke('sync_now', { args: { workspace_id: workspaceId.value } })
    } catch {
      // Local-only projects or offline sessions skip server sync.
    }
    await reloadTree()
    importDialogOpen.value = false
    showToast(
      t('importExport.importSuccess', { folders: result.folders, apis: result.apis }),
      1800,
      'success',
    )
  } catch (e) {
    showToast(String(e), 1800, 'error')
  } finally {
    importDialogRef.value?.setSubmitting(false)
  }
}

async function onTreeAction(action: TreeAction) {
  if (action.anchor) setDialogAnchor(undefined, action.anchor)
  if (action.type === 'folder') {
    if (action.action === 'add-api') {
      await openCreateApi(action.id)
      return
    }
    if (action.action === 'add-subfolder') {
      openCreateFolder(action.id)
      return
    }
    if (action.action === 'export') {
      await exportFolder(action.id, action.name)
      return
    }
    if (action.action === 'settings') {
      folderSettingsId.value = action.id
      folderSettingsOpen.value = true
      return
    }
    if (action.action === 'rename') {
      renameTarget.value = action
      renameDialogInitial.value = action.name
      renameDialogOpen.value = true
      return
    }
    if (action.action === 'delete') {
      deleteTarget.value = action
      deleteDialogTitle.value = t('tree.delete')
      deleteDialogMessage.value = t('tree.deleteFolderConfirm')
      deleteDialogOpen.value = true
    }
    return
  }

  if (action.action === 'edit') {
    await onSelectApi(action.id)
    return
  }
  if (action.action === 'clone') {
    await cloneApi(action.id)
    return
  }
  if (action.action === 'delete') {
    deleteTarget.value = action
    deleteDialogTitle.value = t('tree.delete')
    deleteDialogMessage.value = t('tree.deleteApiConfirm')
    deleteDialogOpen.value = true
  }
}

async function cloneApi(interfaceId: number) {
  if (!workspaceId.value || !projectId.value) return
  try {
    const row = await invoke<InterfaceRow>('get_interface', {
      workspaceId: workspaceId.value,
      projectId: projectId.value,
      interfaceId,
      refresh: false,
    })
    const created = await invoke<InterfaceRow>('create_local_interface', {
      args: {
        workspace_id: workspaceId.value,
        project_id: projectId.value,
        folder_id: row.folder_id,
        name: t('tree.cloneName', { name: row.name }),
        method: row.method || 'GET',
        url: row.url || '',
        status: 3,
        detail: row.detail ?? {},
      },
    })
    await reloadTree()
    await onSelectApi(created.id)
  } catch (e) {
    showToast(String(e) || t('tree.cloneFailed'), 1800, 'error')
  }
}

async function confirmDelete() {
  const target = deleteTarget.value
  if (!target) return
  try {
    if (target.type === 'folder') {
      if (!workspaceId.value || !projectId.value) return
      await invoke('delete_local_folder', {
        workspaceId: workspaceId.value,
        projectId: projectId.value,
        folderId: target.id,
      })
    } else {
      await invoke('delete_local_interface', { interfaceId: target.id })
      const tab = tabs.value.find((x) => x.interfaceId === target.id)
      if (tab) closeTab(tab.id)
    }
    deleteTarget.value = null
    await reloadTree()
  } catch (e) {
    alert(String(e))
  }
}

async function confirmRename(name: string) {
  const target = renameTarget.value
  if (!target) return
  try {
    if (target.type === 'folder') {
      await invoke('rename_local_folder', { args: { id: target.id, name } })
    } else {
      await invoke('rename_local_interface', { args: { id: target.id, name } })
    }
    renameDialogOpen.value = false
    renameTarget.value = null
    await reloadTree()
  } catch (e) {
    alert(String(e))
  }
}

async function onInterfaceSaved(id: number) {
  const active = activeTab.value
  if (active && (active.mode === 'create' || active.interfaceId === id)) {
    activeInterfaceId.value = id
    if (editorRef.value) {
      const state = editorRef.value.exportTabState()
      const snap = editorRef.value.getSnapshot()
      const folderId =
        state.folderId != null && Number.isFinite(Number(state.folderId))
          ? Number(state.folderId)
          : active.folderId
      replaceActiveTab({
        ...active,
        interfaceId: id,
        mode: 'edit',
        name: state.name,
        method: state.method,
        url: state.url,
        folderId,
        detail: state.detail as Record<string, unknown>,
        lastResponse: cloneSendResponse(state.lastResponse),
        requestPaneTab: state.requestPaneTab ?? active.requestPaneTab ?? 'body',
        responseScrollTop:
          typeof state.responseScrollTop === 'number' && Number.isFinite(state.responseScrollTop)
            ? Math.max(0, state.responseScrollTop)
            : (active.responseScrollTop ?? 0),
        snapshot: snap,
        dirty: false,
      })
      // Keep editor mounted: do not rewrite editorInitial / remount key (avoids button flash).
      editorRef.value.syncBaseline()
    }
    void persistTabLayout()
    await reloadTree()
    void revealApiInTree(id, {
      folderId: activeTab.value?.folderId ?? active.folderId,
    })
    return
  }
  await reloadTree()
  await onSelectApi(id)
}

function onEditorCancel() {
  const tab = activeTab.value
  if (tab?.mode === 'create') requestCloseTab(tab.id)
}

function requestCloseTab(id: string) {
  if (saveBeforeCloseOpen.value) return
  const tab = tabs.value.find((x) => x.id === id)
  if (!tab) return
  if (id === activeTabId.value) {
    persistActiveTab()
  }
  const current = tabs.value.find((x) => x.id === id)
  if (!current?.dirty) {
    closeTab(id)
    return
  }
  pendingCloseTabId.value = id
  saveBeforeCloseOpen.value = true
}

/** ESC / overlay: only close the prompt, keep the unsaved tab. */
function abortSaveBeforeClose() {
  pendingCloseTabId.value = ''
}

/** Cancel button: discard changes and close the tab. */
function discardSaveBeforeClose() {
  const id = pendingCloseTabId.value
  pendingCloseTabId.value = ''
  if (id) closeTab(id)
}

async function confirmSaveBeforeClose() {
  const id = pendingCloseTabId.value
  if (!id) return
  if (activeTabId.value !== id) {
    activeTabId.value = id
    const tab = tabs.value.find((x) => x.id === id)
    if (tab) applyTabToForm(tab, { remount: false })
    await nextTick()
    await nextTick()
  }
  const saved = (await editorRef.value?.save()) ?? false
  if (!saved) return
  pendingCloseTabId.value = ''
  saveBeforeCloseOpen.value = false
  closeTab(id)
}

function closeTab(id: string) {
  const idx = tabs.value.findIndex((x) => x.id === id)
  if (idx < 0) return
  void disposeWsTab(id)
  tabs.value.splice(idx, 1)
  if (!tabs.value.length) {
    clearTabsAndPersist()
    return
  }
  if (activeTabId.value === id) {
    const next = tabs.value[Math.max(0, idx - 1)]
    activeTabId.value = next.id
    applyTabToForm(next)
  }
  void persistTabLayout()
}

function openTabContextMenu(event: MouseEvent, tabId: string) {
  hideTreeBlankContextMenu()
  tabMenu.visible = true
  tabMenu.x = event.clientX
  tabMenu.y = event.clientY
  tabMenu.tabId = tabId
}

function hideTabContextMenu() {
  tabMenu.visible = false
}

function openTreeBlankContextMenu(event: MouseEvent) {
  hideTabContextMenu()
  treeBlankMenu.visible = true
  treeBlankMenu.x = event.clientX
  treeBlankMenu.y = event.clientY
}

function hideTreeBlankContextMenu() {
  treeBlankMenu.visible = false
}

function runTreeBlankMenuAction(action: 'add-api' | 'add-folder') {
  hideTreeBlankContextMenu()
  if (action === 'add-api') {
    void openCreateApi(0)
    return
  }
  openCreateFolder(0, undefined, t('tree.addFolder'))
}

function closeOtherTabs(tabId: string) {
  if (!tabs.value.some((x) => x.id === tabId)) return
  for (const tab of tabs.value) {
    if (tab.id !== tabId) void disposeWsTab(tab.id)
  }
  tabs.value = tabs.value.filter((x) => x.id === tabId)
  if (activeTabId.value !== tabId) {
    const tab = tabs.value[0]
    activeTabId.value = tab.id
    applyTabToForm(tab)
  }
  void persistTabLayout()
}

function closeTabsLeft(tabId: string) {
  const index = tabs.value.findIndex((x) => x.id === tabId)
  if (index <= 0) return
  const removed = tabs.value.slice(0, index)
  const removedActive = removed.some((x) => x.id === activeTabId.value)
  for (const tab of removed) void disposeWsTab(tab.id)
  tabs.value = tabs.value.slice(index)
  if (removedActive) {
    const tab = tabs.value[0]
    activeTabId.value = tab.id
    applyTabToForm(tab)
  }
  void persistTabLayout()
}

function closeTabsRight(tabId: string) {
  const index = tabs.value.findIndex((x) => x.id === tabId)
  if (index < 0 || index >= tabs.value.length - 1) return
  const removed = tabs.value.slice(index + 1)
  const removedActive = removed.some((x) => x.id === activeTabId.value)
  for (const tab of removed) void disposeWsTab(tab.id)
  tabs.value = tabs.value.slice(0, index + 1)
  if (removedActive) {
    const tab = tabs.value[tabs.value.length - 1]
    activeTabId.value = tab.id
    applyTabToForm(tab)
  }
  void persistTabLayout()
}

function closeAllTabs() {
  clearTabsAndPersist()
}

function runTabMenuAction(action: 'others' | 'left' | 'right' | 'all') {
  const tabId = tabMenu.tabId
  hideTabContextMenu()
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

function onTabsWheel(e: WheelEvent) {
  const el = tabsEl.value
  if (!el) return
  if (!e.shiftKey) return
  if (Math.abs(e.deltaY) < Math.abs(e.deltaX)) return
  e.preventDefault()
  el.scrollLeft += e.deltaY
}

watch([workspaceId, projectId], () => {
  void ensureWorkspaceBootstrapped()
})

watch([environmentId, envVarsRevision], () => {
  void refreshCurlText()
})

onMounted(async () => {
  document.addEventListener('click', hideTabContextMenu)
  document.addEventListener('click', hideTreeBlankContextMenu)
  await initWorkspaceContext()
  await ensureWorkspaceBootstrapped()
  // Let the restored editor paint under the splash before revealing.
  await nextTick()
  await nextTick()
  markSessionReady()
})

onBeforeUnmount(() => {
  document.removeEventListener('click', hideTabContextMenu)
  document.removeEventListener('click', hideTreeBlankContextMenu)
})

async function onPickProject(id: number | null) {
  if (id == null) return
  await selectProject(id)
}

async function onPickEnv(id: number | null) {
  await selectEnvironment(id)
}

async function onSelectApi(id: number, options?: { restore?: boolean }) {
  if (!workspaceId.value || !projectId.value) return

  const row = await invoke<InterfaceRow>('get_interface', {
    workspaceId: workspaceId.value,
    projectId: projectId.value,
    interfaceId: id,
    refresh: false,
  })

  const existing = tabs.value.find((x) => x.interfaceId === id)
  if (existing) {
    existing.folderId = row.folder_id
    existing.detail = {
      ...((existing.detail as Record<string, unknown> | null) ?? {}),
      folder_id: row.folder_id,
    }
    if (existing.id !== activeTabId.value) {
      activateTab(existing.id)
    } else if (!existing.dirty) {
      applyTabToForm(existing)
    }
    if (!options?.restore && !restoringTabs) void persistTabLayout()
    return
  }

  let headers = defaultHeaders()
  const rowDetail = (row.detail as Record<string, unknown> | null) ?? null
  if (rowDetail && Array.isArray(rowDetail.request_headers)) {
    headers = (rowDetail.request_headers as Array<Record<string, unknown>>).map((h) => ({
      key: String(h.name ?? ''),
      value: String(h.example ?? ''),
      enabled: h.enabled !== false,
    }))
    if (!headers.length) headers = defaultHeaders()
  }
  const body = (rowDetail?.request_body as { raw?: string } | undefined)?.raw ?? ''

  if (!options?.restore) persistActiveTab()
  const data = {
    name: row.name,
    method: row.method || 'GET',
    url: row.url || '',
    headers,
    body,
  }
  const detail = {
    ...(rowDetail ?? {}),
    folder_id: row.folder_id,
  }
  const tab: RequestTab = {
    id: `tab-${tabSeq++}`,
    interfaceId: id,
    folderId: row.folder_id,
    ...data,
    dirty: false,
    snapshot: '',
    mode: 'edit',
    detail,
    lastResponse: null,
    requestPaneTab: 'body',
    responseScrollTop: 0,
  }
  tabs.value.push(tab)
  activeTabId.value = tab.id
  applyTabToForm(tab)
  if (!options?.restore && !restoringTabs) void persistTabLayout()
}
</script>

<style scoped>
.shell {
  height: 100%;
  display: flex;
  min-height: 0;
  min-width: 0;
  overflow-x: hidden;
}

.sidebar {
  flex-shrink: 0;
  background: var(--color-sidebar);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.sidebar-resizer {
  flex-shrink: 0;
  /* Wide hit target; negative margins keep the flex gap at 2px. */
  width: 6px;
  margin: 0 -2px;
  cursor: col-resize;
  position: relative;
  background: transparent;
  z-index: 2;
}

.sidebar-resizer::before {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 1px;
  transform: translateX(-50%);
  background: var(--color-border);
}

:root:not([data-theme='dark']) .sidebar-resizer::before,
:root[data-theme='light'] .sidebar-resizer::before {
  /* Flat hairline — avoid a recessed/groove look in light theme */
  background: color-mix(in srgb, var(--color-border) 55%, #fff);
}

.sidebar-resizer:hover::before,
.sidebar-resizer--active::before {
  width: 2px;
  background: var(--color-primary);
}

.project-bar {
  padding: 8px 6px 0;
}

.search-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 6px 10px 10px;
}

.search {
  flex: 1;
  min-width: 0;
  margin: 0;
  height: 29px;
  padding: 0 10px;
  font-size: 13px;
  box-sizing: border-box;
}

.tree-wrap {
  flex: 1;
  overflow: auto;
  padding: 0 6px 12px;
  background: var(--color-sidebar);
  scrollbar-gutter: stable;
}

.empty {
  padding: 12px;
}

.main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--color-content);
}

.content-top {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px 6px 10px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-sidebar);
}

.req-tabs {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 2px;
  overflow-x: auto;
  scrollbar-width: none;
}

.req-tabs::-webkit-scrollbar {
  display: none;
}

.req-tab {
  position: relative;
  max-width: 200px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  border: 0;
  border-radius: var(--radius);
  background: transparent;
  color: var(--color-text-secondary);
  padding: 0 6px 0 8px;
  font-size: 13px;
  white-space: nowrap;
  flex-shrink: 0;
}

.req-tab .method {
  min-width: 36px;
  flex-shrink: 0;
  line-height: 1;
}

.req-tab:hover:not(.active) {
  background: color-mix(in srgb, var(--color-text) 10%, var(--color-sidebar));
  color: var(--color-text);
}

.req-tab.active {
  background: var(--color-tab-active);
  color: var(--color-text);
}

.req-tab-name {
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
  user-select: none;
}

.req-tab-close {
  /*
   * Optical offsets (saved for revert):
   *   --tab-close-bg-nudge:  1px
   *   --tab-close-dot-nudge: 1px
   *   --tab-close-x-nudge:   0px   (× looked fine in tab; glyph sits low in box)
   * Center preview (box geometric middle):
   *   bg/dot 0; × -1px so glyph strokes sit on box center
   */
  --tab-close-bg-nudge: 0px;
  --tab-close-dot-nudge: 0px;
  --tab-close-x-nudge: -1px;
  position: relative;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
  line-height: 1;
  padding: 0;
  opacity: 0.35;
  flex-shrink: 0;
  isolation: isolate;
}

.req-tab-close-x {
  display: block;
  line-height: 1;
  transform: translateY(var(--tab-close-x-nudge));
}

.req-tab:hover .req-tab-close,
.req-tab.active:not(.dirty) .req-tab-close {
  opacity: 0.85;
}

.req-tab.dirty:not(:hover) .req-tab-close {
  color: transparent;
  opacity: 1;
}

.req-tab.dirty:not(:hover) .req-tab-close::before {
  content: '';
  position: absolute;
  left: 50%;
  top: calc(50% + var(--tab-close-dot-nudge));
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-primary);
  transform: translate(-50%, -50%);
}

.req-tab-close:hover {
  opacity: 1 !important;
  color: var(--color-text);
}

.req-tab-close:hover::after {
  content: '';
  position: absolute;
  left: 0;
  top: var(--tab-close-bg-nudge);
  width: 18px;
  height: 18px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--color-text) 16%, transparent);
  pointer-events: none;
  z-index: -1;
}

.tab-add {
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: var(--radius);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 18px;
  line-height: 1;
  flex-shrink: 0;
  font-weight: 400;
  display: inline-grid;
  place-items: center;
  padding: 0;
}

.tab-add:hover {
  background: color-mix(in srgb, var(--color-text) 10%, var(--color-sidebar));
  color: var(--color-text);
}

.env-wrap {
  flex-shrink: 0;
  max-width: 180px;
}

.code-toggle {
  width: 30px;
  height: 30px;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  padding: 0;
  cursor: pointer;
}

.code-toggle:hover {
  background: color-mix(in srgb, var(--color-text) 10%, var(--color-sidebar));
  color: var(--color-text);
}

.code-toggle.active {
  background: var(--color-active);
  color: var(--color-primary);
}

.work-row {
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
  align-items: stretch;
  overflow: hidden;
}

.work {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--color-content);
}

.code-resizer {
  flex-shrink: 0;
  width: 5px;
  cursor: col-resize;
  position: relative;
  background: transparent;
  z-index: 2;
}

.code-resizer::before {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 1px;
  transform: translateX(-50%);
  background: var(--color-border);
}

.code-resizer:hover::before,
.code-resizer--active::before {
  width: 2px;
  background: var(--color-primary);
}

.work-editor {
  flex: 1;
  min-height: 0;
}

.empty-work {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.empty-work-hint {
  margin: 0;
  font-size: 14px;
}

.empty-work-brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
}

.empty-work-logo {
  width: 64px;
  height: 64px;
  opacity: 0.88;
  user-select: none;
}

.empty-work-title {
  font-size: 22px;
  font-weight: 600;
  letter-spacing: 0.03em;
  color: var(--color-text-secondary);
}

.req-tab-menu {
  position: fixed;
  z-index: 3000;
  min-width: 140px;
  padding: 4px 0;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  box-shadow: var(--shadow-menu);
}

.req-tab-menu-item {
  display: block;
  width: 100%;
  padding: 8px 14px;
  border: none;
  background: transparent;
  color: var(--color-text);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.req-tab-menu-item:hover:not(:disabled) {
  background: var(--color-menu-hover);
}

.req-tab-menu-item:disabled {
  color: var(--color-text-secondary);
  cursor: not-allowed;
}
</style>
