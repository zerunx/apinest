<script setup lang="ts">
import { computed, watch } from 'vue'
import { Delete, Plus } from '@element-plus/icons-vue'
import { useTableRowDrag } from '@/composables/useTableRowDrag'
import { useLocale } from '@/composables/useLocale'
import {
  addChildAtPath,
  addSiblingAtPath,
  areSiblingPaths,
  compactFieldTree,
  ensureTrailingEmptyRoot,
  flattenFieldTree,
  hasFieldContent,
  isTrailingPlaceholderRow,
  removeNodeAtPath,
  reorderSiblingAtPath,
  setAllFieldsRequired,
  siblingParentPath,
  updateNodeAtPath,
  type FieldTreeNode,
} from '@/utils/interface-field-tree'

const props = defineProps<{
  modelValue: FieldTreeNode[]
  readonly?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: FieldTreeNode[]]
}>()

const { t } = useLocale()

const fieldTypeOptions = ['string', 'number', 'boolean', 'object', 'array', 'file']

const flatRows = computed(() => flattenFieldTree(props.modelValue))

const allFilledRequired = computed(
  () => flatRows.value.length > 0 && flatRows.value.every((row) => row.node.required),
)

const requiredIndeterminate = computed(() => {
  const requiredCount = flatRows.value.filter((row) => row.node.required).length
  return requiredCount > 0 && requiredCount < flatRows.value.length
})

const requiredHeaderTooltip = computed(() =>
  allFilledRequired.value
    ? t('workspace.interfaceForm.clearAllRequired')
    : t('workspace.interfaceForm.toggleAllRequired'),
)

function syncTree(tree: FieldTreeNode[]) {
  const normalized = props.readonly ? tree : ensureTrailingEmptyRoot(tree)
  emit('update:modelValue', normalized)
}

function updateRow(path: number[], patch: Partial<FieldTreeNode>) {
  syncTree(updateNodeAtPath(props.modelValue, path, patch))
}

function removeRow(path: number[]) {
  syncTree(removeNodeAtPath(props.modelValue, path))
}

function addChild(path: number[]) {
  syncTree(addChildAtPath(props.modelValue, path))
}

function addSibling(path: number[]) {
  syncTree(addSiblingAtPath(props.modelValue, path))
}

function toggleAllRequired() {
  syncTree(setAllFieldsRequired(props.modelValue, !allFilledRequired.value))
}

function canDragRow(flatIndex: number) {
  if (props.readonly) return false
  const row = flatRows.value[flatIndex]
  if (!row || isTrailingPlaceholderRow(row, flatRows.value)) return false
  return hasFieldContent(row.node)
}

function reorderRows(fromFlatIndex: number, toFlatIndex: number) {
  const from = flatRows.value[fromFlatIndex]
  const to = flatRows.value[toFlatIndex]
  if (!from || !to || !areSiblingPaths(from.path, to.path)) return
  const parentPath = siblingParentPath(from.path)
  const fromIndex = from.path[from.path.length - 1]
  const toIndex = to.path[to.path.length - 1]
  syncTree(reorderSiblingAtPath(props.modelValue, parentPath, fromIndex, toIndex))
}

const {
  handleDragStart,
  handleDragOver,
  handleDragLeave,
  handleDrop,
  handleDragEnd,
  rowClass,
} = useTableRowDrag({
  canDrag: canDragRow,
  onReorder: reorderRows,
})

watch(
  () => props.modelValue,
  (tree) => {
    if (props.readonly || !tree.length) return
    const last = tree[tree.length - 1]
    if (hasFieldContent(last)) {
      syncTree(tree)
    }
  },
  { deep: true },
)

defineExpose({ compact: () => compactFieldTree(props.modelValue) })
</script>

<template>
  <div class="interface-param-table-wrap">
  <table class="interface-param-table">
    <colgroup>
      <col class="interface-param-table__sort" />
      <col class="interface-param-table__col-name" />
      <col class="interface-param-table__col-type" />
      <col class="interface-param-table__center" />
      <col class="interface-param-table__col-desc" />
      <col class="interface-param-table__col-example" />
      <col v-if="!readonly" class="interface-param-table__actions" />
    </colgroup>
    <thead>
      <tr>
        <th class="interface-param-table__sort" />
        <th class="interface-param-table__col-name">{{ t('workspace.interfaceForm.paramName') }}</th>
        <th class="interface-param-table__col-type">{{ t('workspace.interfaceForm.paramType') }}</th>
        <th class="interface-param-table__center interface-param-table__required-header">
          <template v-if="readonly">{{ t('workspace.interfaceForm.required') }}</template>
          <el-tooltip
            v-else
            :content="requiredHeaderTooltip"
            placement="top"
            :show-after="0"
            :hide-after="0"
          >
            <el-checkbox
              :model-value="allFilledRequired"
              :indeterminate="requiredIndeterminate"
              @change="toggleAllRequired"
            />
          </el-tooltip>
        </th>
        <th class="interface-param-table__col-desc">{{ t('workspace.interfaceForm.description') }}</th>
        <th class="interface-param-table__col-example">{{ t('workspace.interfaceForm.example') }}</th>
        <th v-if="!readonly" class="interface-param-table__actions" />
      </tr>
    </thead>
    <tbody>
      <tr
        v-for="(row, index) in flatRows"
        :key="row.path.join('-')"
        :class="rowClass(index)"
        @dragover="handleDragOver($event, index)"
        @dragleave="handleDragLeave(index)"
        @drop="handleDrop($event, index)"
      >
        <td class="interface-param-table__sort">
          <span
            v-if="canDragRow(index)"
            class="interface-param-table__sort-handle"
            draggable="true"
            :aria-label="t('workspace.interfaceForm.dragToSort')"
            @dragstart="handleDragStart($event, index)"
            @dragend="handleDragEnd"
          >
            <span class="interface-param-table__grip" aria-hidden="true">
              <i /><i /><i /><i /><i /><i />
            </span>
          </span>
        </td>
        <td class="interface-param-table__col-name">
          <div class="interface-param-table__name-cell" :style="{ paddingLeft: `${row.depth * 20}px` }">
            <el-input
              :model-value="row.node.name"
              :placeholder="t('workspace.interfaceForm.paramName')"
              :readonly="readonly"
              @update:model-value="updateRow(row.path, { name: $event })"
            />
          </div>
        </td>
        <td class="interface-param-table__col-type">
          <el-select
            :model-value="row.node.type"
            :disabled="readonly"
            popper-class="app-action-dropdown"
            @update:model-value="updateRow(row.path, { type: $event })"
          >
            <el-option v-for="type in fieldTypeOptions" :key="type" :label="type" :value="type" />
          </el-select>
        </td>
        <td class="interface-param-table__center">
          <el-checkbox
            :model-value="row.node.required"
            :disabled="readonly"
            @update:model-value="updateRow(row.path, { required: $event })"
          />
        </td>
        <td class="interface-param-table__col-desc">
          <el-input
            :model-value="row.node.description"
            :placeholder="t('workspace.interfaceForm.paramDescPlaceholder')"
            :readonly="readonly"
            @update:model-value="updateRow(row.path, { description: $event })"
          />
        </td>
        <td class="interface-param-table__col-example">
          <el-input
            :model-value="row.node.example"
            :placeholder="t('workspace.interfaceForm.example')"
            :readonly="readonly"
            @update:model-value="updateRow(row.path, { example: $event })"
          />
        </td>
        <td v-if="!readonly" class="interface-param-table__actions">
          <div v-if="hasFieldContent(row.node)" class="interface-param-table__field-actions">
            <el-tooltip
              :content="t('workspace.interfaceForm.addSiblingField')"
              placement="top"
              :show-after="0"
              :hide-after="0"
            >
              <button
                type="button"
                class="interface-param-table__icon-btn"
                :aria-label="t('workspace.interfaceForm.addSiblingField')"
                @click="addSibling(row.path)"
              >
                <el-icon :size="14"><Plus /></el-icon>
              </button>
            </el-tooltip>
            <el-tooltip
              :content="t('workspace.interfaceForm.addChildField')"
              placement="top"
              :show-after="0"
              :hide-after="0"
            >
              <button
                type="button"
                class="interface-param-table__icon-btn interface-param-table__icon-btn--boxed"
                :aria-label="t('workspace.interfaceForm.addChildField')"
                @click="addChild(row.path)"
              >
                <el-icon :size="12"><Plus /></el-icon>
              </button>
            </el-tooltip>
            <el-tooltip
              :content="t('common.delete')"
              placement="top"
              :show-after="0"
              :hide-after="0"
            >
              <button
                type="button"
                class="interface-param-table__delete"
                :aria-label="t('common.delete')"
                @click="removeRow(row.path)"
              >
                <el-icon :size="14"><Delete /></el-icon>
              </button>
            </el-tooltip>
          </div>
        </td>
      </tr>
    </tbody>
  </table>
  </div>
</template>
