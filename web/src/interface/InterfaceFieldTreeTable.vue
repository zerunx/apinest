<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Delete, Plus } from '@element-plus/icons-vue'
import { useTableRowDrag } from '@/composables/useTableRowDrag'
import { useLocale } from '@/composables/useLocale'
import InterfaceDocCopyCell from '@/interface/InterfaceDocCopyCell.vue'
import InterfaceTypeSelect from '@/interface/InterfaceTypeSelect.vue'
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

const collapsedKeys = ref(new Set<string>())

function pathKey(path: number[]) {
  return path.join('-')
}

function isCollapsed(path: number[]) {
  return collapsedKeys.value.has(pathKey(path))
}

function toggleCollapsed(path: number[]) {
  const key = pathKey(path)
  const next = new Set(collapsedKeys.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  collapsedKeys.value = next
}

function expandPath(path: number[]) {
  const key = pathKey(path)
  if (!collapsedKeys.value.has(key)) return
  const next = new Set(collapsedKeys.value)
  next.delete(key)
  collapsedKeys.value = next
}

const allRows = computed(() => flattenFieldTree(props.modelValue))
const displayRows = computed(() =>
  flattenFieldTree(
    props.readonly ? compactFieldTree(props.modelValue) : props.modelValue,
    0,
    [],
    collapsedKeys.value,
  ),
)

function requiredLabel(required: boolean) {
  return required
    ? t('workspace.interfaceForm.requiredYes')
    : t('workspace.interfaceForm.requiredNo')
}

function typeLabel(type: string) {
  return `[${type || 'string'}]`
}

function nativeValue(event: Event) {
  return (event.target as HTMLInputElement | HTMLSelectElement).value
}

function nativeChecked(event: Event) {
  return (event.target as HTMLInputElement).checked
}

const allFilledRequired = computed(
  () => allRows.value.length > 0 && allRows.value.every((row) => row.node.required),
)

const requiredIndeterminate = computed(() => {
  const requiredCount = allRows.value.filter((row) => row.node.required).length
  return requiredCount > 0 && requiredCount < allRows.value.length
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
  expandPath(path)
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
  const row = displayRows.value[flatIndex]
  if (!row || isTrailingPlaceholderRow(row, displayRows.value)) return false
  return hasFieldContent(row.node)
}

function reorderRows(fromFlatIndex: number, toFlatIndex: number) {
  const from = displayRows.value[fromFlatIndex]
  const to = displayRows.value[toFlatIndex]
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
    if (!hasFieldContent(last)) return
    const next = ensureTrailingEmptyRoot(tree)
    if (next === tree || next.length === tree.length) return
    emit('update:modelValue', next)
  },
)

defineExpose({ compact: () => compactFieldTree(props.modelValue) })
</script>

<template>
  <div
    class="interface-param-table-wrap"
    :class="{ 'interface-param-table-wrap--readonly': readonly }"
  >
  <table
    class="interface-param-table"
    :class="{ 'interface-param-table--readonly': readonly }"
  >
    <colgroup>
      <col v-if="!readonly" class="interface-param-table__sort" />
      <col class="interface-param-table__col-name" />
      <col class="interface-param-table__col-type" />
      <col class="interface-param-table__center" />
      <col class="interface-param-table__col-desc" />
      <col class="interface-param-table__col-example" />
      <col v-if="!readonly" class="interface-param-table__actions" />
    </colgroup>
    <thead>
      <tr>
        <th v-if="!readonly" class="interface-param-table__sort" />
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
        v-for="(row, index) in displayRows"
        :key="row.path.join('-')"
        :class="rowClass(index)"
        @dragover="handleDragOver($event, index)"
        @dragleave="handleDragLeave(index)"
        @drop="handleDrop($event, index)"
      >
        <td v-if="!readonly" class="interface-param-table__sort">
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
          <div class="interface-param-table__name-cell">
            <span
              v-if="row.depth > 0"
              class="interface-param-table__tree-indent"
              aria-hidden="true"
            >
              <i
                v-for="step in row.depth"
                :key="step"
                class="interface-param-table__tree-line"
              />
            </span>
            <button
              v-if="row.hasChildren"
              type="button"
              class="interface-param-table__tree-toggle"
              :aria-label="
                isCollapsed(row.path)
                  ? t('workspace.interfaceForm.expandField')
                  : t('workspace.interfaceForm.collapseField')
              "
              :aria-expanded="!isCollapsed(row.path)"
              @click.stop="toggleCollapsed(row.path)"
            />
            <span v-else class="interface-param-table__tree-spacer" aria-hidden="true" />
            <InterfaceDocCopyCell v-if="readonly" :value="row.node.name" />
            <input
              v-else
              class="interface-param-table__native-input"
              :value="row.node.name"
              :placeholder="t('workspace.interfaceForm.paramName')"
              @input="updateRow(row.path, { name: nativeValue($event) })"
            />
          </div>
        </td>
        <td class="interface-param-table__col-type">
          <span v-if="readonly" class="interface-param-table__text">{{ typeLabel(row.node.type) }}</span>
          <InterfaceTypeSelect
            v-else
            :model-value="row.node.type || 'string'"
            :options="fieldTypeOptions"
            @update:model-value="updateRow(row.path, { type: $event })"
          />
        </td>
        <td class="interface-param-table__center">
          <span v-if="readonly" class="interface-param-table__text">{{ requiredLabel(row.node.required) }}</span>
          <input
            v-else
            type="checkbox"
            class="interface-param-table__native-check"
            :checked="row.node.required"
            @change="updateRow(row.path, { required: nativeChecked($event) })"
          />
        </td>
        <td class="interface-param-table__col-desc">
          <InterfaceDocCopyCell v-if="readonly" :value="row.node.description" />
          <input
            v-else
            class="interface-param-table__native-input"
            :value="row.node.description"
            :placeholder="t('workspace.interfaceForm.paramDescPlaceholder')"
            @input="updateRow(row.path, { description: nativeValue($event) })"
          />
        </td>
        <td class="interface-param-table__col-example">
          <InterfaceDocCopyCell v-if="readonly" :value="row.node.example" />
          <input
            v-else
            class="interface-param-table__native-input"
            :value="row.node.example"
            :placeholder="t('workspace.interfaceForm.example')"
            @input="updateRow(row.path, { example: nativeValue($event) })"
          />
        </td>
        <td v-if="!readonly" class="interface-param-table__actions">
          <div v-if="hasFieldContent(row.node)" class="interface-param-table__field-actions">
            <button
              type="button"
              class="interface-param-table__icon-btn"
              :title="t('workspace.interfaceForm.addSiblingField')"
              :aria-label="t('workspace.interfaceForm.addSiblingField')"
              @click="addSibling(row.path)"
            >
              <el-icon :size="14"><Plus /></el-icon>
            </button>
            <button
              type="button"
              class="interface-param-table__icon-btn interface-param-table__icon-btn--boxed"
              :title="t('workspace.interfaceForm.addChildField')"
              :aria-label="t('workspace.interfaceForm.addChildField')"
              @click="addChild(row.path)"
            >
              <el-icon :size="12"><Plus /></el-icon>
            </button>
            <button
              type="button"
              class="interface-param-table__delete"
              :title="t('common.delete')"
              :aria-label="t('common.delete')"
              @click="removeRow(row.path)"
            >
              <el-icon :size="14"><Delete /></el-icon>
            </button>
          </div>
        </td>
      </tr>
    </tbody>
  </table>
  </div>
</template>
