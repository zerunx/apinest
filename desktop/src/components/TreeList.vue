<template>
  <ul class="tree">
    <TreeNode
      v-for="item in items"
      :key="itemKey(item)"
      :item="item"
      :active-id="activeId"
      @select="$emit('select', $event)"
      @action="$emit('action', $event)"
    />
  </ul>
</template>

<script setup lang="ts">
import type { TreeItem } from '../types'
import TreeNode from './TreeNode.vue'
import type { TreeAction } from './TreeNode.vue'

defineProps<{
  items: TreeItem[]
  activeId: number | null
}>()

defineEmits<{
  select: [id: number]
  action: [TreeAction]
}>()

function itemKey(item: TreeItem) {
  return `${item.type}-${item.id}`
}
</script>

<style scoped>
.tree {
  list-style: none;
  margin: 0;
  padding: 0;
}
</style>
