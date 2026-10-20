<template>
  <div class="tree-list" :class="{ 'tree-list--dragging': draggingApiId != null }">
    <ul class="tree" :class="{ 'tree--dragging': draggingApiId != null }">
      <TreeNode
        v-for="item in items"
        :key="itemKey(item)"
        :item="item"
        :active-id="activeId"
        :expanded-folder-ids="expandedFolderIds"
        :parent-folder-id="0"
        :can-drag="canDrag"
        :dragging-api-id="draggingApiId"
        :drag-over-key="dragOverKey"
        :reordering="reordering"
        @select="$emit('select', $event)"
        @action="$emit('action', $event)"
        @toggle-expand="$emit('toggle-expand', $event)"
        @api-drag-start="$emit('api-drag-start', $event)"
        @api-drag-over="$emit('api-drag-over', $event)"
        @api-drag-leave="$emit('api-drag-leave')"
        @api-drop="$emit('api-drop', $event)"
        @api-drag-end="$emit('api-drag-end')"
      />
    </ul>
    <div
      v-if="dragOverKey === 'root'"
      class="tree-root-drop-line"
      aria-hidden="true"
    />
  </div>
</template>

<script setup lang="ts">
import type { TreeItem } from '../types'
import TreeNode from './TreeNode.vue'
import type { TreeAction, TreeApiDragPayload } from './TreeNode.vue'

defineProps<{
  items: TreeItem[]
  activeId: number | null
  expandedFolderIds: number[]
  canDrag?: boolean
  draggingApiId?: number | null
  dragOverKey?: string | null
  reordering?: boolean
}>()

defineEmits<{
  select: [id: number]
  action: [TreeAction]
  'toggle-expand': [folderId: number]
  'api-drag-start': [payload: TreeApiDragPayload]
  'api-drag-over': [payload: TreeApiDragPayload]
  'api-drag-leave': []
  'api-drop': [payload: TreeApiDragPayload]
  'api-drag-end': []
}>()

function itemKey(item: TreeItem) {
  return `${item.type}-${item.id}`
}
</script>

<style scoped>
.tree-list {
  position: relative;
}

.tree {
  list-style: none;
  margin: 0;
  padding: 0;
}

.tree--dragging,
.tree-list--dragging {
  user-select: none;
}

.tree-root-drop-line {
  height: 2px;
  margin-top: 0;
  background: var(--color-primary);
  border-radius: 0;
  pointer-events: none;
}
</style>
