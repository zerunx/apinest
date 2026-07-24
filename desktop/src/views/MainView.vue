<template>
  <div class="shell">
    <aside class="sidebar" :style="{ width: `${sidebarWidth}px` }">
      <div class="project-bar">
        <SelectPicker
          :model-value="projectId"
          :items="projects"
          :placeholder="t('project.label')"
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
        <button
          type="button"
          class="search-add"
          :title="t('create.folder')"
          @click="openCreateFolder(0, $event)"
        >
          <svg
            class="search-add-icon"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            aria-hidden="true"
          >
            <path d="M12 5v14M5 12h14" />
          </svg>
        </button>
      </div>

      <div class="tree-wrap">
        <TreeList
          v-if="tree.length"
          :items="tree"
          :active-id="activeInterfaceId"
          @select="onSelectApi"
          @action="onTreeAction"
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
            variant="env"
            align="right"
            @update:model-value="onPickEnv"
          />
        </div>
      </div>

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
import InterfaceEditor from '../components/InterfaceEditor.vue'
import type { TreeAction } from '../components/TreeNode.vue'
import type { HeaderPair, InterfaceRow, TreeItem } from '../types'
import { useSaveShortcut } from '../composables/useSaveShortcut'
import { useCloseTabShortcut } from '../composables/useCloseTabShortcut'
import { anchorFromEvent, type DialogAnchor } from '../composables/useDialogAnchor'
import { useDragResize } from '../composables/useDragResize'

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
}

const { t } = useI18n()
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
const {
  projects,
  environments,
  workspaceId,
  projectId,
  environmentId,
  initWorkspaceContext,
  selectProject,
  selectEnvironment,
  loadProjects,
} = useWorkspaceContext()

const tree = ref<TreeItem[]>([])
const search = ref('')
const activeInterfaceId = ref<number | null>(null)
const tabsEl = ref<HTMLElement | null>(null)
const projectDialogOpen = ref(false)
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
  return [{ key: 'Content-Type', value: 'application/json', enabled: true }]
}

function editorSnapshot(): string {
  return editorRef.value?.getSnapshot() ?? ''
}

function tabStateFromEditor(tab: RequestTab): RequestTab | null {
  if (!editorRef.value) return null
  const state = editorRef.value.exportTabState()
  const snap = editorSnapshot()
  const dirty = tab.snapshot ? snap !== tab.snapshot : false
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
    dirty,
    snapshot: dirty ? tab.snapshot : snap,
  }
}

function replaceActiveTab(next: RequestTab) {
  const idx = tabs.value.findIndex((x) => x.id === next.id)
  if (idx >= 0) tabs.value[idx] = next
}

function syncTabSnapshotFromEditor() {
  const tab = activeTab.value
  if (!tab || !editorRef.value) return
  const snap = editorSnapshot()
  if (tab.dirty && tab.snapshot) {
    replaceActiveTab(tabStateFromEditor(tab) ?? tab)
    return
  }
  replaceActiveTab({
    ...(tabStateFromEditor(tab) ?? tab),
    snapshot: snap,
    dirty: false,
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
  const detail = folderId != null ? { folder_id: folderId } : null
  return {
    id: `tab-${tabSeq++}`,
    interfaceId: null,
    folderId: folderId ?? null,
    ...data,
    dirty: false,
    snapshot: '',
    mode: 'create',
    detail,
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
  }
  if (options?.remount !== false) {
    editorKey.value += 1
  }
}

function onEditorReady() {
  void nextTick().then(() => syncTabSnapshotFromEditor())
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
}

function activateTab(id: string) {
  if (id === activeTabId.value) return
  persistActiveTab()
  activeTabId.value = id
  const tab = tabs.value.find((x) => x.id === id)
  if (tab) applyTabToForm(tab)
  void persistTabLayout()
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
    alert(t('create.needWorkspaceFirst'))
    return
  }
  setDialogAnchor(event)
  projectDialogOpen.value = true
}

function openCreateFolder(parentId = 0, event?: MouseEvent) {
  if (!workspaceId.value || !projectId.value) {
    alert(t('create.needProjectFirst'))
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
    alert(t('create.needProjectFirst'))
    return
  }
  let targetFolderId = folderId ?? null
  if (targetFolderId == null) {
    try {
      const folders = await invoke<{ id: number }[]>('list_local_folders', {
        workspaceId: workspaceId.value,
        projectId: projectId.value,
      })
      if (!folders.length) {
        alert(t('create.needFolderFirst'))
        return
      }
      targetFolderId = folders[0].id
    } catch {
      alert(t('create.needFolderFirst'))
      return
    }
  }
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

.search-add {
  width: 28px;
  height: 28px;
  flex-shrink: 0;
  border: 0;
  border-radius: 6px;
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  cursor: pointer;
}

.search-add-icon {
  display: block;
  flex-shrink: 0;
}

.tree-wrap {
  flex: 1;
  overflow: auto;
  padding: 0 6px 12px;
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
  background: var(--color-bg);
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
  border-radius: 8px;
  background: transparent;
  color: var(--color-text-secondary);
  padding: 0 6px 0 10px;
  font-size: 13px;
  white-space: nowrap;
  flex-shrink: 0;
}

.req-tab:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.req-tab.active {
  background: var(--color-tab-active);
  color: var(--color-text);
}

.req-tab.dirty .req-tab-name,
.req-tab.dirty .method {
  font-style: italic;
}

.req-tab-name {
  overflow: hidden;
  text-overflow: ellipsis;
}

.req-tab-close {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: grid;
  place-items: center;
  font-size: 19px;
  line-height: 1;
  opacity: 0.35;
}

.req-tab:hover .req-tab-close,
.req-tab.active .req-tab-close {
  opacity: 0.85;
}

.req-tab-close:hover {
  opacity: 1 !important;
  background: var(--color-hover);
}

.tab-add {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 8px;
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
  background: var(--color-hover);
  color: var(--color-text);
}

.env-wrap {
  flex-shrink: 0;
  max-width: 180px;
}

.work {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--color-content);
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
