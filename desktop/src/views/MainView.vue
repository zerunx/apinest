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
          :create-label="t('project.label')"
          variant="project"
          @update:model-value="onPickProject"
          @create="openCreateProject"
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
          :create-title="t('create.folder')"
          :import-label="t('importExport.import')"
          :export-label="t('importExport.export')"
          @create="openCreateFolder(0, $event)"
          @import="openImportDialog"
          @export="exportCurrentProject"
        />
      </div>

      <div class="tree-wrap">
        <TreeList
          v-if="tree.length"
          :items="tree"
          :active-id="activeInterfaceId"
          :can-drag="canDragApis"
          :dragging-api-id="dragApiState?.apiId ?? null"
          :drag-over-key="dragOverKey"
          :reordering="reorderingApis"
          @select="onSelectApi"
          @action="onTreeAction"
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
            @click="activateTab(tab.id)"
            @contextmenu.prevent="openTabContextMenu($event, tab.id)"
          >
            <span class="method" :class="tab.method.toLowerCase()">{{ tab.method }}</span>
            <span class="req-tab-name">{{ tab.name || t('request.untitled') }}</span>
            <span class="req-tab-close" title="Close" @click.stop="requestCloseTab(tab.id)">×</span>
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
            :interface-id="activeInterfaceId"
            :initial="(editorInitial as any)"
            @saved="onInterfaceSaved"
            @state-change="onEditorStateChange"
            @editor-ready="onEditorReady"
            @cancel="onEditorCancel"
          />
          <div v-else class="empty-work">
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
      :title="t('create.project')"
      :label="t('project.label')"
      :anchor="dialogAnchor"
      @confirm="confirmCreateProject"
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
      v-model:open="saveBeforeCloseOpen"
      :title="t('request.saveBeforeCloseTitle')"
      :message="t('request.saveBeforeClose')"
      variant="primary"
      :confirm-label="t('common.confirm')"
      :close-on-confirm="false"
      @confirm="confirmSaveBeforeClose"
      @update:open="onSaveBeforeCloseOpenChange"
    />
    <EnvironmentManageDialog v-model:open="envManageOpen" />
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
import { useToast } from '../composables/useToast'
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
}

function cloneSendResponse(res: SendResponse | null | undefined): SendResponse | null {
  if (!res) return null
  return {
    ...res,
    headers: (res.headers ?? []).map((h) => ({ ...h })),
  }
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
} = useWorkspaceContext()

const tree = ref<TreeItem[]>([])
const search = ref('')
const dragApiState = ref<{ apiId: number; folderId: number } | null>(null)
const dragOverKey = ref<string | null>(null)
const reorderingApis = ref(false)
const canDragApis = computed(() => !search.value.trim() && !reorderingApis.value)
const activeInterfaceId = ref<number | null>(null)
const tabsEl = ref<HTMLElement | null>(null)
const projectDialogOpen = ref(false)
const envManageOpen = ref(false)
const folderDialogOpen = ref(false)
const folderParentId = ref(0)
const folderDialogTitle = ref('')
const folderDialogInitial = ref('')
const renameDialogOpen = ref(false)
const renameDialogInitial = ref('')
const renameTarget = ref<TreeAction | null>(null)
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
let contextBootstrapped = false
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
    detail: state.detail,
    headers: defaultHeaders(),
    body: String((state.detail as { request_body?: { raw?: string } }).request_body?.raw ?? ''),
    interfaceId: activeInterfaceId.value,
    lastResponse: cloneSendResponse(state.lastResponse),
    requestPaneTab: state.requestPaneTab ?? tab.requestPaneTab ?? 'body',
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
  }
}

function applyTabToForm(tab: RequestTab, options?: { remount?: boolean }) {
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
    status: (tab.detail as { status?: number } | null)?.status ?? 1,
    detail: {
      ...(tab.detail ?? {}),
      ...(folderId != null ? { folder_id: folderId } : {}),
    },
    lastResponse: cloneSendResponse(tab.lastResponse),
    requestPaneTab: tab.requestPaneTab ?? 'body',
  }
  // Remount only when explicitly requested. Default keep-alive avoids flashing
  // method/url/name/folder/body while the editor tears down and remounts.
  if (options?.remount === true) {
    editorKey.value += 1
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
  tabs.value = []
  activeTabId.value = ''
  activeInterfaceId.value = null
  editorInitial.value = null
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
    layout.workspaceId !== workspaceId.value ||
    layout.projectId !== projectId.value ||
    !layout.openInterfaceIds.length
  ) {
    return
  }

  restoringTabs = true
  try {
    clearTabs()
    for (const id of layout.openInterfaceIds) {
      await onSelectApi(id, { restore: true })
    }
    if (layout.activeInterfaceId != null) {
      const tab = tabs.value.find((x) => x.interfaceId === layout.activeInterfaceId)
      if (tab) activateTab(tab.id)
    }
  } finally {
    restoringTabs = false
    await persistTabLayout()
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

async function bootstrapWorkspaceContent(isSwitch: boolean) {
  if (!workspaceId.value || !projectId.value) {
    tree.value = []
    return
  }
  if (isSwitch) {
    clearTabs()
  }
  await reloadTree()
  if (!isSwitch && !contextBootstrapped) {
    await restoreTabLayout()
    contextBootstrapped = true
  }
  if (isSwitch) {
    await persistTabLayout()
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
  projectDialogOpen.value = true
}

function openCreateFolder(parentId = 0, event?: MouseEvent) {
  if (!workspaceId.value || !projectId.value) {
    showToast(t('create.needProjectFirst'), 1500, 'warning')
    return
  }
  setDialogAnchor(event)
  folderParentId.value = parentId
  folderDialogTitle.value = parentId
    ? t('tree.addSubFolder')
    : t('create.folder')
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

async function confirmCreateProject(name: string) {
  if (!workspaceId.value) return
  try {
    const row = await invoke<{ id: number }>('create_local_project', {
      args: { workspace_id: workspaceId.value, name },
    })
    projectDialogOpen.value = false
    await loadProjects()
    await selectProject(row.id)
    await reloadTree()
  } catch (e) {
    alert(String(e))
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
  if (action.action === 'delete') {
    deleteTarget.value = action
    deleteDialogTitle.value = t('tree.delete')
    deleteDialogMessage.value = t('tree.deleteApiConfirm')
    deleteDialogOpen.value = true
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
        snapshot: snap,
        dirty: false,
      })
      // Keep editor mounted: do not rewrite editorInitial / remount key (avoids button flash).
      editorRef.value.syncBaseline()
    }
    void persistTabLayout()
    await reloadTree()
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

function onSaveBeforeCloseOpenChange(open: boolean) {
  if (open) return
  if (!pendingCloseTabId.value) return
  const id = pendingCloseTabId.value
  pendingCloseTabId.value = ''
  closeTab(id)
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
  tabMenu.visible = true
  tabMenu.x = event.clientX
  tabMenu.y = event.clientY
  tabMenu.tabId = tabId
}

function hideTabContextMenu() {
  tabMenu.visible = false
}

function closeOtherTabs(tabId: string) {
  if (!tabs.value.some((x) => x.id === tabId)) return
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
  const removedActive = tabs.value.slice(0, index).some((x) => x.id === activeTabId.value)
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
  const removedActive = tabs.value.slice(index + 1).some((x) => x.id === activeTabId.value)
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

watch([workspaceId, projectId], async ([ws, proj], [prevWs, prevProj]) => {
  if (!ws || !proj) {
    tree.value = []
    return
  }
  const isSwitch = prevWs != null && (prevWs !== ws || prevProj !== proj)
  await bootstrapWorkspaceContent(isSwitch)
})

watch([environmentId, envVarsRevision], () => {
  void refreshCurlText()
})

onMounted(async () => {
  document.addEventListener('click', hideTabContextMenu)
  await initWorkspaceContext()
  if (workspaceId.value && projectId.value && !contextBootstrapped) {
    await bootstrapWorkspaceContent(false)
  }
})

onBeforeUnmount(() => {
  document.removeEventListener('click', hideTabContextMenu)
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
      enabled: true,
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
  width: 5px;
  cursor: col-resize;
  position: relative;
  background: transparent;
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
  min-height: 42px;
  padding: 6px 10px 6px 8px;
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
  height: 26px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 0;
  border-radius: var(--radius);
  background: transparent;
  color: var(--color-text-secondary);
  padding: 0 6px 0 10px;
  font-size: 13px;
  white-space: nowrap;
  flex-shrink: 0;
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
}

.req-tab-close {
  position: relative;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  display: inline-grid;
  place-items: center;
  font-size: 16px;
  line-height: 0;
  padding: 0;
  opacity: 0.35;
  flex-shrink: 0;
  transform: translateY(-1px);
}

.req-tab:hover .req-tab-close,
.req-tab.active:not(.dirty) .req-tab-close {
  opacity: 0.85;
}

.req-tab.dirty:not(:hover) .req-tab-close {
  color: transparent;
  opacity: 1;
  transform: none;
}

.req-tab.dirty:not(:hover) .req-tab-close::before {
  content: '';
  position: absolute;
  left: 50%;
  top: 50%;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-primary);
  transform: translate(-50%, -50%);
}

.req-tab-close:hover {
  opacity: 1 !important;
  background: var(--color-hover);
}

.tab-add {
  width: 26px;
  height: 26px;
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
