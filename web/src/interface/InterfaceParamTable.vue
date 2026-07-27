<script setup lang="ts">
import { computed, watch } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import { reorderList, useTableRowDrag } from '@/composables/useTableRowDrag'
import { useLocale } from '@/composables/useLocale'
import {
  compactParamRows,
  emptyParamRow,
  ensureTrailingEmptyRow,
  hasParamContent,
  setAllParamsRequired,
  type ParamRow,
} from '@/utils/interface-params'

const props = defineProps<{
  modelValue: ParamRow[]
  readonly?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ParamRow[]]
}>()

const { t } = useLocale()

const fieldTypeOptions = ['string', 'number', 'boolean', 'object', 'array', 'file']

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
        v-for="(row, index) in modelValue"
        :key="index"
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
          <el-input
            :model-value="row.name"
            :placeholder="t('workspace.interfaceForm.paramName')"
            :readonly="readonly"
            @update:model-value="updateRow(index, { name: $event })"
          />
        </td>
        <td class="interface-param-table__col-type">
          <el-select
            :model-value="row.type"
            :disabled="readonly"
            popper-class="app-action-dropdown"
            @update:model-value="updateRow(index, { type: $event })"
          >
            <el-option v-for="type in fieldTypeOptions" :key="type" :label="type" :value="type" />
          </el-select>
        </td>
        <td class="interface-param-table__center">
          <el-checkbox
            :model-value="row.required"
            :disabled="readonly"
            @update:model-value="updateRow(index, { required: $event })"
          />
        </td>
        <td class="interface-param-table__col-desc">
          <el-input
            :model-value="row.description"
            :placeholder="t('workspace.interfaceForm.paramDescPlaceholder')"
            :readonly="readonly"
            @update:model-value="updateRow(index, { description: $event })"
          />
        </td>
        <td class="interface-param-table__col-example">
          <el-input
            :model-value="row.example"
            :placeholder="t('workspace.interfaceForm.example')"
            :readonly="readonly"
            @update:model-value="updateRow(index, { example: $event })"
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
