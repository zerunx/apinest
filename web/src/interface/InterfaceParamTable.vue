<script setup lang="ts">
import { computed, watch } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import { reorderList, useTableRowDrag } from '@/composables/useTableRowDrag'
import { useLocale } from '@/composables/useLocale'
import InterfaceDocCopyCell from '@/interface/InterfaceDocCopyCell.vue'
import {
  compactParamRows,
  emptyParamRow,
  ensureTrailingEmptyRow,
  hasParamContent,
  setAllParamsRequired,
  type ParamRow,
} from '@/utils/interface-params'

interface HeaderSuggestion {
  value: string
}

const props = defineProps<{
  modelValue: ParamRow[]
  readonly?: boolean
  nameSuggestions?: readonly string[]
  nameLabel?: string
  namePlaceholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ParamRow[]]
}>()

const { t } = useLocale()

const fieldTypeOptions = ['string', 'number', 'boolean', 'object', 'array', 'file']
const useNameSuggest = computed(() => !!props.nameSuggestions?.length)
const nameColumnLabel = computed(() => props.nameLabel || t('workspace.interfaceForm.paramName'))
const nameColumnPlaceholder = computed(
  () => props.namePlaceholder || t('workspace.interfaceForm.paramName'),
)
const displayRows = computed(() =>
  props.readonly ? compactParamRows(props.modelValue) : props.modelValue,
)

const allFilledRequired = computed(
  () => props.modelValue.length > 0 && props.modelValue.every((row) => row.required),
)

const requiredIndeterminate = computed(() => {
  const requiredCount = props.modelValue.filter((row) => row.required).length
  return requiredCount > 0 && requiredCount < props.modelValue.length
})

const requiredHeaderTooltip = computed(() =>
  allFilledRequired.value
    ? t('workspace.interfaceForm.clearAllRequired')
    : t('workspace.interfaceForm.toggleAllRequired'),
)

function searchNameSuggestions(query: string, cb: (results: HeaderSuggestion[]) => void) {
  const keyword = query.trim().toLowerCase()
  const results = (props.nameSuggestions ?? [])
    .filter((header) => !keyword || header.toLowerCase().includes(keyword))
    .map((value) => ({ value }))
  cb([...results])
}

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

function syncRows(rows: ParamRow[]) {
  const normalized = props.readonly ? rows : ensureTrailingEmptyRow(rows)
  emit('update:modelValue', normalized)
}

function updateRow(index: number, patch: Partial<ParamRow>) {
  const rows = props.modelValue.map((row, i) => (i === index ? { ...row, ...patch } : row))
  syncRows(rows)
}

function removeRow(index: number) {
  const rows = props.modelValue.filter((_, i) => i !== index)
  syncRows(rows.length ? rows : [emptyParamRow()])
}

function toggleAllRequired() {
  syncRows(setAllParamsRequired(props.modelValue, !allFilledRequired.value))
}

function sortableEndIndex(rows: ParamRow[]) {
  if (!rows.length) return 0
  const last = rows[rows.length - 1]
  return hasParamContent(last) ? rows.length : rows.length - 1
}

function canDragRow(index: number) {
  if (props.readonly) return false
  return index < sortableEndIndex(props.modelValue)
}

function reorderRows(fromIndex: number, toIndex: number) {
  const end = sortableEndIndex(props.modelValue)
  if (fromIndex >= end || toIndex >= end) return
  syncRows(reorderList(props.modelValue, fromIndex, toIndex))
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
  (rows) => {
    if (props.readonly || !rows.length) return
    const last = rows[rows.length - 1]
    if (hasParamContent(last)) {
      syncRows(rows)
    }
  },
  { deep: true },
)

defineExpose({ compact: () => compactParamRows(props.modelValue) })
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
        <th class="interface-param-table__col-name">{{ nameColumnLabel }}</th>
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
        :key="index"
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
          <InterfaceDocCopyCell v-if="readonly" :value="row.name" />
          <el-autocomplete
            v-else-if="useNameSuggest"
            :model-value="row.name"
            :fetch-suggestions="searchNameSuggestions"
            :trigger-on-focus="true"
            clearable
            :placeholder="nameColumnPlaceholder"
            popper-class="app-action-dropdown"
            @update:model-value="updateRow(index, { name: $event ?? '' })"
          />
          <input
            v-else
            class="interface-param-table__native-input"
            :value="row.name"
            :placeholder="nameColumnPlaceholder"
            @input="updateRow(index, { name: nativeValue($event) })"
          />
        </td>
        <td class="interface-param-table__col-type">
          <span v-if="readonly" class="interface-param-table__text">{{ typeLabel(row.type) }}</span>
          <select
            v-else
            class="interface-param-table__native-select"
            :value="row.type"
            @change="updateRow(index, { type: nativeValue($event) })"
          >
            <option v-for="type in fieldTypeOptions" :key="type" :value="type">{{ type }}</option>
          </select>
        </td>
        <td class="interface-param-table__center">
          <span v-if="readonly" class="interface-param-table__text">{{ requiredLabel(row.required) }}</span>
          <input
            v-else
            type="checkbox"
            class="interface-param-table__native-check"
            :checked="row.required"
            @change="updateRow(index, { required: nativeChecked($event) })"
          />
        </td>
        <td class="interface-param-table__col-desc">
          <InterfaceDocCopyCell v-if="readonly" :value="row.description" />
          <input
            v-else
            class="interface-param-table__native-input"
            :value="row.description"
            :placeholder="t('workspace.interfaceForm.paramDescPlaceholder')"
            @input="updateRow(index, { description: nativeValue($event) })"
          />
        </td>
        <td class="interface-param-table__col-example">
          <InterfaceDocCopyCell v-if="readonly" :value="row.example" />
          <input
            v-else
            class="interface-param-table__native-input"
            :value="row.example"
            :placeholder="t('workspace.interfaceForm.example')"
            @input="updateRow(index, { example: nativeValue($event) })"
          />
        </td>
        <td v-if="!readonly" class="interface-param-table__actions">
          <button
            v-if="hasParamContent(row)"
            type="button"
            class="interface-param-table__delete"
            @click="removeRow(index)"
          >
            <el-icon :size="14"><Delete /></el-icon>
          </button>
        </td>
      </tr>
    </tbody>
  </table>
  </div>
</template>
