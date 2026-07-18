<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useTemplateRef } from 'vue'
import { ArrowDown, Folder, Plus, Search, Setting, User } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import {
  clampPanelWidth,
  readWorkspaceLayout,
  writeWorkspaceLayout,
} from '@/utils/workspace-layout-storage'
import type { ApiTreeNode, HttpMethod } from '@/types/workspace'

const { t } = useLocale()
const {
  activeModule,
  activeModuleTab,
  contextMode,
  projects,
  apiTree,
  selectedApiId,
  selectedFolderId,
  loadingProjects,
  loadingApiTree,
  selectProject,
  selectFolder,
  selectApi,
  toggleNode,
  isNodeExpanded,
  openCreateProject,
  openSpaceSectionTab,
  openEnvSectionTab,
  submitCreateFolder,
  submitUpdateFolder,
  submitDeleteFolder,
  submitDeleteInterface,
  submitReorderInterfaces,
  openCreateApi,
  parseFolderId,
  parseApiId,
} = useWorkspaceContext()

const deletingApiId = ref<number | null>(null)

const searchQuery = ref('')

type FlatApiNode = {
  node: ApiTreeNode
  depth: number
  parentFolderId: string | null
}

const dragApiState = ref<{ apiId: string; folderId: string } | null>(null)
const dragOverApiId = ref<string | null>(null)
const reorderingApis = ref(false)

const canDragApis = computed(() => !searchQuery.value.trim())

type FolderDialogMode = 'create-root' | 'create-sub' | 'edit'

const folderDialogVisible = ref(false)
const folderDialogMode = ref<FolderDialogMode>('create-root')
const folderDialogParentId = ref(0)
const folderDialogTargetId = ref(0)
const folderDialogName = ref('')
const folderDialogSubmitting = ref(false)

type DeleteTarget =
  | { kind: 'folder'; folderId: number; name: string }
  | { kind: 'api'; apiId: number; name: string }

const deleteDialogVisible = ref(false)
const deleteTarget = ref<DeleteTarget | null>(null)
const deleteDialogSubmitting = ref(false)

const deleteDialogTitle = computed(() => {
  if (!deleteTarget.value) return ''
  return deleteTarget.value.kind === 'folder' ? t('workspace.deleteFolder') : t('common.delete')
})

const deleteDialogMessage = computed(() => {
  if (!deleteTarget.value) return ''
  return deleteTarget.value.kind === 'folder'
    ? t('workspace.deleteFolderConfirm', { name: deleteTarget.value.name })
    : t('workspace.deleteApiConfirm', { name: deleteTarget.value.name })
})

const folderDialogTitle = computed(() => {
  if (folderDialogMode.value === 'edit') return t('workspace.editFolder')
  if (folderDialogMode.value === 'create-sub') return t('workspace.addSubFolder')
  return t('workspace.createFolder')
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

const filteredProjects = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return projects.value
  return projects.value.filter((item) => item.name.toLowerCase().includes(q))
})

function filterTree(nodes: ApiTreeNode[], query: string): ApiTreeNode[] {
  if (!query) return nodes
  const q = query.toLowerCase()

  return nodes.reduce<ApiTreeNode[]>((acc, node) => {
    if (node.type === 'api') {
      if (node.name.toLowerCase().includes(q)) acc.push(node)
      return acc
    }

    const children = filterTree(node.children ?? [], query)
    if (node.name.toLowerCase().includes(q) || children.length > 0) {
      acc.push({ ...node, children })
    }
    return acc
  }, [])
}

const filteredApiTree = computed(() => filterTree(apiTree.value, searchQuery.value.trim()))

function handleProjectClick(id: number) {
  selectProject(id)
}

function findNodeById(nodes: ApiTreeNode[], id: string): ApiTreeNode | null {
  for (const node of nodes) {
    if (node.id === id) return node
    if (node.children?.length) {
      const found = findNodeById(node.children, id)
      if (found) return found
    }
  }
  return null
}

function renderNodes(
  nodes: ApiTreeNode[],
  depth = 0,
  parentFolderId: string | null = null,
): FlatApiNode[] {
  const result: FlatApiNode[] = []

  for (const node of nodes) {
    result.push({
      node,
      depth,
      parentFolderId: node.type === 'api' ? parentFolderId : null,
    })
    if (node.type === 'folder' && isNodeExpanded(node.id) && node.children?.length) {
      result.push(...renderNodes(node.children, depth + 1, node.id))
    }
  }

  return result
}

const flatApiNodes = computed(() => renderNodes(filteredApiTree.value))

const spaceNavItems = computed(() => [
  { id: 'workspace-list', label: t('workspace.workspaceManage'), icon: Folder },
  { id: 'project-list', label: t('workspace.projectManage'), icon: Setting },
  { id: 'member-list', label: t('member.title'), icon: User },
])

const environmentNavItems = computed(() => [
  { id: 'env-list', label: t('workspace.modules.environment'), icon: Setting },
  { id: 'env-variables', label: t('environment.variables'), icon: Folder },
])

const searchPlaceholder = computed(() => {
  if (activeModule.value === 'project') return t('workspace.searchSpaceSection')
  if (activeModule.value === 'environment') return t('workspace.searchEnvironmentSection')
  return contextMode.value === 'workspace'
    ? t('workspace.searchProject')
    : t('workspace.searchApi')
})

const showModuleNav = computed(
  () => activeModule.value === 'project' || activeModule.value === 'environment',
)

function handleSpaceNavClick(id: string) {
  if (id === 'workspace-list') {
    openSpaceSectionTab('workspaces')
    return
  }
  if (id === 'project-list') {
    openSpaceSectionTab('projects')
    return
  }
  openSpaceSectionTab('members')
}

function isModuleNavActive(id: string) {
  const tab = activeModuleTab.value
  if (!tab) return false
  if (id === 'workspace-list') return tab.kind === 'workspace-list'
  if (id === 'project-list') return tab.kind === 'project-list'
  if (id === 'member-list') return tab.kind === 'member-list'
  if (id === 'env-list') return tab.kind === 'env-list'
  if (id === 'env-variables') return tab.kind === 'env-variables'
  return false
}

function handleEnvironmentNavClick(id: string) {
  openEnvSectionTab(id === 'env-list' ? 'list' : 'variables')
}

function handleFolderRowClick(node: ApiTreeNode) {
  selectFolder(node.id)
}

function handleExpandToggle(nodeId: string) {
  toggleNode(nodeId)
}

function handleCreateProject() {
  openCreateProject()
}

function openFolderDialog(mode: FolderDialogMode, options: { parentId?: number; folderId?: number; name?: string } = {}) {
  folderDialogMode.value = mode
  folderDialogParentId.value = options.parentId ?? 0
  folderDialogTargetId.value = options.folderId ?? 0
  folderDialogName.value = options.name ?? ''
  folderDialogVisible.value = true
}

function handleCreateRootFolder() {
  openFolderDialog('create-root')
}

function handleAddSubFolder(folderId: number) {
  openFolderDialog('create-sub', { parentId: folderId })
}

function handleEditFolder(folderId: number, currentName: string) {
  openFolderDialog('edit', { folderId, name: currentName })
}

async function submitFolderDialog() {
  const name = folderDialogName.value.trim()
  if (!name) {
    ElMessage.warning(t('workspace.nameRequired'))
    return
  }

  folderDialogSubmitting.value = true
  try {
    if (folderDialogMode.value === 'edit') {
      await submitUpdateFolder(folderDialogTargetId.value, name)
      ElMessage.success(t('workspace.updateFolderSuccess'))
    } else {
      const parentId = folderDialogMode.value === 'create-sub' ? folderDialogParentId.value : 0
      await submitCreateFolder(name, parentId)
      ElMessage.success(t('workspace.createFolderSuccess'))
    }
    folderDialogVisible.value = false
  } catch (error) {
    const fallback =
      folderDialogMode.value === 'edit'
        ? t('workspace.updateFolderFailed')
        : t('workspace.createFolderFailed')
    ElMessage.error(error instanceof Error ? error.message : fallback)
  } finally {
    folderDialogSubmitting.value = false
  }
}

function handleAddApi(node: ApiTreeNode) {
  selectFolder(node.id)
  openCreateApi()
}

function openDeleteDialog(target: DeleteTarget) {
  deleteTarget.value = target
  deleteDialogVisible.value = true
}

function handleDeleteApi(node: ApiTreeNode) {
  const apiId = parseApiId(node.id)
  if (!apiId) return
  openDeleteDialog({ kind: 'api', apiId, name: node.name })
}

function handleDeleteFolder(folderId: number, name: string) {
  openDeleteDialog({ kind: 'folder', folderId, name })
}

async function submitDeleteDialog() {
  const target = deleteTarget.value
  if (!target) return

  deleteDialogSubmitting.value = true
  try {
    if (target.kind === 'folder') {
      await submitDeleteFolder(target.folderId)
      ElMessage.success(t('workspace.deleteFolderSuccess'))
    } else {
      deletingApiId.value = target.apiId
      await submitDeleteInterface(target.apiId)
      ElMessage.success(t('workspace.deleteApiSuccess'))
    }
    deleteDialogVisible.value = false
    deleteTarget.value = null
  } catch (error) {
    const fallback =
      target.kind === 'folder' ? t('workspace.deleteFolderFailed') : t('workspace.deleteApiFailed')
    ElMessage.error(error instanceof Error ? error.message : fallback)
  } finally {
    deleteDialogSubmitting.value = false
    deletingApiId.value = null
  }
}

function handleApiAction(command: string, node: ApiTreeNode) {
  switch (command) {
    case 'edit':
      selectApi(node.id)
      break
    case 'delete':
      handleDeleteApi(node)
      break
  }
}

function getApiIdsInFolder(folderId: string) {
  const folder = findNodeById(apiTree.value, folderId)
  if (!folder?.children) return []
  return folder.children.filter((item) => item.type === 'api').map((item) => item.id)
}

function resetApiDragState() {
  dragApiState.value = null
  dragOverApiId.value = null
}

function handleApiDragStart(event: DragEvent, node: ApiTreeNode, parentFolderId: string | null) {
  if (!canDragApis.value || !parentFolderId) return
  dragApiState.value = { apiId: node.id, folderId: parentFolderId }
  dragOverApiId.value = null
  event.dataTransfer?.setData('text/plain', node.id)
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
  }
}

function handleApiDragOver(event: DragEvent, node: ApiTreeNode, parentFolderId: string | null) {
  const drag = dragApiState.value
  if (
    !drag ||
    node.type !== 'api' ||
    !parentFolderId ||
    parentFolderId !== drag.folderId ||
    node.id === drag.apiId
  ) {
    return
  }

  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
  dragOverApiId.value = node.id
}

function handleApiDragLeave(node: ApiTreeNode) {
  if (dragOverApiId.value === node.id) {
    dragOverApiId.value = null
  }
}

async function handleApiDrop(event: DragEvent, node: ApiTreeNode, parentFolderId: string | null) {
  event.preventDefault()
  const drag = dragApiState.value
  if (
    !drag ||
    node.type !== 'api' ||
    !parentFolderId ||
    parentFolderId !== drag.folderId ||
    node.id === drag.apiId
  ) {
    resetApiDragState()
    return
  }

  const apiIds = getApiIdsInFolder(parentFolderId)
  const fromIndex = apiIds.indexOf(drag.apiId)
  const toIndex = apiIds.indexOf(node.id)
  if (fromIndex === -1 || toIndex === -1) {
    resetApiDragState()
    return
  }

  const nextApiIds = [...apiIds]
  nextApiIds.splice(fromIndex, 1)
  // Drop indicator is below the target row — insert after target, not before.
  const insertIndex = fromIndex < toIndex ? toIndex : toIndex + 1
  nextApiIds.splice(insertIndex, 0, drag.apiId)

  const folderId = parseFolderId(parentFolderId)
  const orderedApiIds = nextApiIds
    .map((id) => parseApiId(id))
    .filter((id): id is number => id !== null)

  resetApiDragState()

  if (!folderId || orderedApiIds.length !== nextApiIds.length) return

  reorderingApis.value = true
  try {
    await submitReorderInterfaces(folderId, orderedApiIds)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.reorderApiFailed'))
  } finally {
    reorderingApis.value = false
  }
}

function handleFolderAction(command: string, node: ApiTreeNode) {
  const folderId = parseFolderId(node.id)
  if (!folderId) return

  switch (command) {
    case 'add-api':
      handleAddApi(node)
      break
    case 'add-folder':
      handleAddSubFolder(folderId)
      break
    case 'edit':
      handleEditFolder(folderId, node.name)
      break
    case 'delete':
      handleDeleteFolder(folderId, node.name)
      break
  }
}

const panelRef = useTemplateRef<HTMLElement>('panel')
const panelWidth = ref(readWorkspaceLayout().panelWidth)
const isResizing = ref(false)
let resizeStartX = 0
let resizeStartWidth = panelWidth.value
let pendingPanelWidth = panelWidth.value
let resizeRafId = 0

function applyPanelWidth(width: number) {
  pendingPanelWidth = width
  const panel = panelRef.value
  if (panel) {
    panel.style.width = `${width}px`
    return
  }
  panelWidth.value = width
}

function onResizeMove(event: MouseEvent) {
  pendingPanelWidth = clampPanelWidth(resizeStartWidth + event.clientX - resizeStartX)
  if (resizeRafId) return

  resizeRafId = requestAnimationFrame(() => {
    resizeRafId = 0
    applyPanelWidth(pendingPanelWidth)
  })
}

function stopResize() {
  if (resizeRafId) {
    cancelAnimationFrame(resizeRafId)
    resizeRafId = 0
  }

  isResizing.value = false
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', stopResize)

  panelWidth.value = pendingPanelWidth
  writeWorkspaceLayout({ panelWidth: pendingPanelWidth })
}

function startResize(event: MouseEvent) {
  event.preventDefault()
  isResizing.value = true
  resizeStartX = event.clientX
  resizeStartWidth = panelWidth.value
  pendingPanelWidth = panelWidth.value
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', stopResize)
}

onBeforeUnmount(() => {
  stopResize()
})
</script>

<template>
  <aside
    ref="panel"
    class="context-panel"
    :class="{ 'context-panel--resizing': isResizing }"
    :style="{ width: `${panelWidth}px` }"
  >
    <div class="context-panel__header">
      <div v-if="!showModuleNav" class="context-panel__search">
        <el-icon :size="16"><Search /></el-icon>
        <input
          v-model="searchQuery"
          type="text"
          class="context-panel__search-input"
          :placeholder="searchPlaceholder"
        />
      </div>
      <button
        v-if="activeModule === 'api' && contextMode === 'workspace'"
        type="button"
        class="context-panel__create-btn"
        :title="t('workspace.createProject')"
        @click="handleCreateProject"
      >
        <el-icon :size="16"><Plus /></el-icon>
      </button>
      <button
        v-else-if="activeModule === 'api'"
        type="button"
        class="context-panel__add-btn"
        :title="t('workspace.createFolder')"
        @click="handleCreateRootFolder"
      >
        <span class="context-panel__add-btn-plus">+</span>
      </button>
    </div>

    <div v-if="activeModule === 'project'" class="context-panel__body">
      <ul class="context-panel__tree">
        <li
          v-for="item in spaceNavItems"
          :key="item.id"
          class="context-panel__tree-item context-panel__tree-item--folder"
          :class="{ 'context-panel__tree-item--selected': isModuleNavActive(item.id) }"
          @click="handleSpaceNavClick(item.id)"
        >
          <span class="context-panel__expand context-panel__expand--leaf" />
          <el-icon :size="14" class="context-panel__folder-icon">
            <component :is="item.icon" />
          </el-icon>
          <span class="context-panel__tree-name">{{ item.label }}</span>
        </li>
      </ul>
    </div>

    <div v-else-if="activeModule === 'environment'" class="context-panel__body">
      <ul class="context-panel__tree">
        <li
          v-for="item in environmentNavItems"
          :key="item.id"
          class="context-panel__tree-item context-panel__tree-item--folder"
          :class="{ 'context-panel__tree-item--selected': isModuleNavActive(item.id) }"
          @click="handleEnvironmentNavClick(item.id)"
        >
          <span class="context-panel__expand context-panel__expand--leaf" />
          <el-icon :size="14" class="context-panel__folder-icon">
            <component :is="item.icon" />
          </el-icon>
          <span class="context-panel__tree-name">{{ item.label }}</span>
        </li>
      </ul>
    </div>

    <!-- Project list (workspace mode) -->
    <div v-else-if="contextMode === 'workspace'" class="context-panel__body" v-loading="loadingProjects">
      <ul class="context-panel__list">
        <li
          v-for="item in filteredProjects"
          :key="item.id"
          class="context-panel__project-item"
          @click="handleProjectClick(item.id)"
        >
          <el-icon :size="16" class="context-panel__project-icon"><Folder /></el-icon>
          <div class="context-panel__project-info">
            <span class="context-panel__project-name">{{ item.name }}</span>
            <span v-if="item.creator_name" class="context-panel__project-desc">
              {{ item.creator_name }}
            </span>
          </div>
        </li>
        <li v-if="filteredProjects.length === 0" class="context-panel__empty">
          {{ t('workspace.noProjects') }}
        </li>
      </ul>
    </div>

    <!-- API tree (project mode) -->
    <div v-else class="context-panel__body" v-loading="loadingApiTree">
      <ul class="context-panel__tree">
        <li
          v-for="{ node, depth, parentFolderId } in flatApiNodes"
          :key="node.id"
          class="context-panel__tree-item"
          :class="{
            'context-panel__tree-item--folder': node.type === 'folder',
            'context-panel__tree-item--api': node.type === 'api',
            'context-panel__tree-item--selected':
              (node.type === 'folder' && selectedFolderId === node.id) ||
              (node.type === 'api' && selectedApiId === node.id),
            'context-panel__tree-item--dragging':
              node.type === 'api' && dragApiState?.apiId === node.id,
            'context-panel__tree-item--drag-over':
              node.type === 'api' && dragOverApiId === node.id,
          }"
          :draggable="node.type === 'api' && canDragApis && !reorderingApis"
          @click="node.type === 'folder' ? handleFolderRowClick(node) : selectApi(node.id)"
          @dragstart="handleApiDragStart($event, node, parentFolderId)"
          @dragover="handleApiDragOver($event, node, parentFolderId)"
          @dragleave="handleApiDragLeave(node)"
          @drop="handleApiDrop($event, node, parentFolderId)"
          @dragend="resetApiDragState"
        >
          <span
            class="context-panel__expand-zone"
            :class="{ 'context-panel__expand-zone--leaf': node.type !== 'folder' }"
            :style="{ width: `${12 + depth * 16 + 16}px` }"
            @click.stop="node.type === 'folder' ? handleExpandToggle(node.id) : undefined"
          >
            <el-icon
              v-if="node.type === 'folder'"
              :size="12"
              class="context-panel__expand-icon"
              :class="{ 'context-panel__expand-icon--open': isNodeExpanded(node.id) }"
            >
              <ArrowDown />
            </el-icon>
          </span>

          <el-icon v-if="node.type === 'folder'" :size="14" class="context-panel__folder-icon">
            <Folder />
          </el-icon>

          <span
            v-if="node.type === 'api' && node.method"
            class="context-panel__method-wrap"
          >
            <span
              class="context-panel__method"
              :class="`context-panel__method--${methodColors[node.method]}`"
            >
              {{ node.method }}
            </span>
          </span>

          <span class="context-panel__tree-name">{{ node.name }}</span>

          <el-dropdown
            v-if="node.type === 'api'"
            trigger="click"
            :show-arrow="false"
            class="context-panel__folder-menu"
            popper-class="folder-action-dropdown"
            @click.stop
            @command="(cmd: string) => handleApiAction(cmd, node)"
          >
            <button
              type="button"
              class="context-panel__folder-menu-btn"
              :class="{ 'context-panel__folder-menu-btn--active': selectedApiId === node.id }"
              @click.stop
            >
              <span class="context-panel__folder-menu-icon" aria-hidden="true" />
            </button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="edit">{{ t('common.edit') }}</el-dropdown-item>
                <el-dropdown-item command="delete">{{ t('common.delete') }}</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>

          <el-dropdown
            v-if="node.type === 'folder'"
            trigger="click"
            :show-arrow="false"
            class="context-panel__folder-menu"
            popper-class="folder-action-dropdown"
            @click.stop
            @command="(cmd: string) => handleFolderAction(cmd, node)"
          >
            <button
              type="button"
              class="context-panel__folder-menu-btn"
              :class="{ 'context-panel__folder-menu-btn--active': selectedFolderId === node.id }"
              @click.stop
            >
              <span class="context-panel__folder-menu-icon" aria-hidden="true" />
            </button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="add-api">{{ t('workspace.addApi') }}</el-dropdown-item>
                <el-dropdown-item command="add-folder">{{ t('workspace.addSubFolder') }}</el-dropdown-item>
                <el-dropdown-item command="edit" divided>{{ t('common.rename') }}</el-dropdown-item>
                <el-dropdown-item command="delete">{{ t('common.delete') }}</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </li>
        <li v-if="flatApiNodes.length === 0" class="context-panel__empty">
          {{ t('workspace.noApis') }}
        </li>
      </ul>
    </div>

    <div
      class="context-panel__resize-handle"
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize panel"
      @mousedown="startResize"
    />

    <el-dialog
      v-model="folderDialogVisible"
      :title="folderDialogTitle"
      width="420px"
      class="folder-dialog"
      align-center
      destroy-on-close
      @keyup.enter="submitFolderDialog"
    >
      <el-input
        v-model="folderDialogName"
        :placeholder="t('workspace.folderNamePlaceholder')"
        autofocus
      />
      <template #footer>
        <el-button @click="folderDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="folderDialogSubmitting" @click="submitFolderDialog">
          {{ t('common.confirm') }}
        </el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="deleteDialogVisible"
      :title="deleteDialogTitle"
      width="420px"
      class="folder-dialog"
      align-center
      destroy-on-close
    >
      <p class="context-panel__delete-message">{{ deleteDialogMessage }}</p>
      <template #footer>
        <el-button @click="deleteDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="danger" :loading="deleteDialogSubmitting" @click="submitDeleteDialog">
          {{ t('common.delete') }}
        </el-button>
      </template>
    </el-dialog>
  </aside>
</template>

<style scoped>
.context-panel {
  position: relative;
  flex-shrink: 0;
  height: 100%;
  min-height: 0;
  border-right: 1px solid var(--color-border);
  background: var(--color-workspace-content);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-size: 14px;
}

.context-panel--resizing {
  user-select: none;
}

.context-panel--resizing .context-panel__body {
  pointer-events: none;
}

.context-panel--resizing .context-panel__tree-item,
.context-panel--resizing .context-panel__project-item {
  transition: none;
}

.context-panel__resize-handle {
  position: absolute;
  top: 0;
  right: -3px;
  width: 6px;
  height: 100%;
  cursor: col-resize;
  z-index: 10;
  touch-action: none;
}

.context-panel__resize-handle::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 1px;
  transform: translateX(-50%);
  background: transparent;
  transition: background-color 0.15s ease;
}

.context-panel__resize-handle:hover::after,
.context-panel--resizing .context-panel__resize-handle::after {
  background: var(--color-primary);
}

.context-panel__header {
  padding: 12px 14px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--color-workspace-content);
}

.context-panel__search {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 12px;
  height: 38px;
  border-radius: 8px;
  background: var(--color-workspace-inset);
  border: 1px solid var(--color-border);
}

.context-panel__search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-text);
  font-size: 14px;
}

.context-panel__search-input::placeholder {
  color: var(--color-text-secondary);
}

.context-panel__create-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 34px;
  height: 34px;
  padding: 0;
  border: 1px dashed var(--color-border);
  border-radius: 8px;
  background: transparent;
  color: var(--color-primary-light);
  cursor: pointer;
  transition: border-color 0.15s ease;
}

.context-panel__create-btn:hover {
  border-color: var(--color-primary);
}

.context-panel__add-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 34px;
  height: 34px;
  padding: 0;
  border: 1px solid var(--color-primary);
  border-radius: 4px;
  background: var(--color-primary);
  color: #fff;
  cursor: pointer;
  transition: background-color 0.15s ease, border-color 0.15s ease;
}

.context-panel__add-btn:hover {
  background: var(--color-primary-light);
  border-color: var(--color-primary-light);
}

.context-panel__add-btn .el-icon {
  color: #fff;
}

.context-panel__body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 0;
  background: var(--color-workspace-content);
}

.context-panel__list,
.context-panel__tree {
  list-style: none;
  margin: 0;
  padding: 0;
}

.context-panel__project-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 16px;
  cursor: pointer;
  border-bottom: 1px solid var(--color-border);
}

.context-panel__project-icon {
  color: var(--color-primary-light);
  margin-top: 2px;
  flex-shrink: 0;
}

.context-panel__project-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.context-panel__project-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-panel__project-desc {
  font-size: 14px;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-panel__tree-item {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding-right: 8px;
  cursor: pointer;
  user-select: none;
  position: relative;
  transition: background-color 0.15s ease;
}

.context-panel__tree-item:hover,
.context-panel__tree-item--selected {
  background: var(--color-workspace-item-selected);
}

.context-panel__tree-item--selected .context-panel__tree-name {
  color: var(--color-text);
  font-weight: 500;
}

.context-panel__tree-item--selected:not(.context-panel__tree-item--folder) .context-panel__tree-name {
  color: var(--color-primary-light);
}

.context-panel__tree-item--api[draggable='true'] {
  cursor: pointer;
}

.context-panel__tree-item--api[draggable='true']:active {
  cursor: grabbing;
}

.context-panel__tree-item--dragging {
  opacity: 0.45;
}

.context-panel__tree-item--drag-over {
  background: var(--color-workspace-item-selected);
  box-shadow: inset 0 -2px 0 var(--color-primary);
}

.context-panel__folder-menu {
  margin-left: auto;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.context-panel__tree-item:hover .context-panel__folder-menu,
.context-panel__tree-item--selected .context-panel__folder-menu {
  opacity: 1;
}

.context-panel__folder-menu-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.context-panel__folder-menu-btn:hover,
.context-panel__folder-menu-btn--active {
  background: var(--color-workspace-control-hover);
}

.context-panel__folder-menu-icon {
  position: relative;
  display: block;
  width: 14px;
  height: 12px;
}

.context-panel__folder-menu-icon::before {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  height: 2px;
  border-radius: 1px;
  background: currentColor;
  box-shadow: 0 5px 0 currentColor, 0 10px 0 currentColor;
}

.context-panel__expand-zone {
  flex-shrink: 0;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.context-panel__expand-zone--leaf {
  cursor: default;
  pointer-events: none;
}

.context-panel__expand-icon {
  transition: transform 0.15s ease;
  transform: rotate(-90deg);
}

.context-panel__expand-icon--open {
  transform: rotate(0deg);
}

.context-panel__folder-icon {
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.context-panel__method-wrap {
  width: 44px;
  min-width: 44px;
  flex-shrink: 0;
  text-align: left;
}

.context-panel__method {
  display: inline-block;
  font-size: 12px;
  font-weight: 700;
  line-height: 1.4;
  font-family: 'Consolas', 'Monaco', monospace;
}

.context-panel__method--get {
  color: #61affe;
}

.context-panel__method--post {
  color: #49cc90;
}

.context-panel__method--put {
  color: #fca130;
}

.context-panel__method--delete {
  color: #f93e3e;
}

.context-panel__method--patch {
  color: #50e3c2;
}

.context-panel__method--head,
.context-panel__method--options {
  color: var(--color-text-secondary);
}

.context-panel__tree-name {
  font-size: 14px;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.context-panel__empty {
  padding: 28px 16px;
  text-align: center;
  font-size: 14px;
  color: var(--color-text-secondary);
}

.context-panel__delete-message {
  margin: 0;
  font-size: 14px;
  line-height: 1.6;
  color: var(--color-text);
}
</style>
