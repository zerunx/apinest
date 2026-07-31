<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Delete } from '@element-plus/icons-vue'
import InterfaceFieldTreeTable from '@/interface/InterfaceFieldTreeTable.vue'
import InterfaceRequestBodyImportDialog from '@/interface/InterfaceRequestBodyImportDialog.vue'
import { useLocale } from '@/composables/useLocale'
import { ensureTrailingEmptyRoot, responseFieldTreeFromApi } from '@/utils/interface-field-tree'
import { applyBodyFieldsImport, type BodyImportMode } from '@/utils/json-body-import'
import {
  parseHttpStatusCodeInput,
  searchResponseStatusCodes,
} from '@/utils/response-status-code-options'
import type { InterfaceResponseField, InterfaceResponseResult } from '@/types/workspace'

interface StatusCodeSuggestion {
  value: string
}

const props = defineProps<{
  modelValue: InterfaceResponseResult[]
  readonly?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: InterfaceResponseResult[]]
}>()

const { t } = useLocale()
const activeIndex = ref(0)
const importVisible = ref(false)

const formatOptions = ['JSON', 'XML', 'Raw']
const dataTypeOptions = ['Object', 'Array', 'String', 'Number', 'Boolean']

const activeResult = computed(() => props.modelValue[activeIndex.value] ?? null)

const statusCodeInput = ref('')

watch(
  [activeIndex, () => activeResult.value?.status_code],
  () => {
    statusCodeInput.value = activeResult.value ? String(activeResult.value.status_code) : ''
  },
  { immediate: true },
)

function searchStatusCodes(query: string, cb: (results: StatusCodeSuggestion[]) => void) {
  cb(searchResponseStatusCodes(query))
}

function updateStatusCode(value: string) {
  statusCodeInput.value = value
  const parsed = parseHttpStatusCodeInput(value)
  if (parsed !== null) {
    updateActive({ status_code: parsed })
  }
}

function commitStatusCode() {
  const parsed = parseHttpStatusCodeInput(statusCodeInput.value)
  if (parsed !== null) {
    updateActive({ status_code: parsed })
    statusCodeInput.value = String(parsed)
    return
  }
  statusCodeInput.value = activeResult.value ? String(activeResult.value.status_code) : ''
}

function createResult(name: string, statusCode: number): InterfaceResponseResult {
  return {
    name,
    status_code: statusCode,
    format: 'JSON',
    data_type: 'Object',
    fields: responseFieldTreeFromApi([]),
  }
}

function defaultAddedResult(): InterfaceResponseResult {
  return createResult(t('workspace.interfaceForm.defaultFailure'), 500)
}

function syncResults(results: InterfaceResponseResult[]) {
  emit('update:modelValue', results)
}

function updateActive(patch: Partial<InterfaceResponseResult>) {
  if (!activeResult.value) return
  const results = props.modelValue.map((item, index) =>
    index === activeIndex.value ? { ...item, ...patch } : item,
  )
  syncResults(results)
}

function addResult() {
  const results = [...props.modelValue, defaultAddedResult()]
  syncResults(results)
  activeIndex.value = results.length - 1
}

function removeResult(index: number) {
  if (props.modelValue.length <= 1) return
  const results = props.modelValue.filter((_, i) => i !== index)
  syncResults(results)
  if (activeIndex.value >= results.length) {
    activeIndex.value = results.length - 1
  }
}

const activeFields = computed({
  get: () => activeResult.value?.fields ?? [],
  set: (fields: InterfaceResponseField[]) => {
    updateActive({ fields })
  },
})

function resultTabLabel(result: InterfaceResponseResult) {
  const name = result.name.trim() || t('workspace.interfaceForm.responseResult')
  return `${name} (${result.status_code})`
}

function handleImportApply(payload: { mode: BodyImportMode; jsonText: string }) {
  if (!activeResult.value) return
  try {
    const fields = ensureTrailingEmptyRoot(
      applyBodyFieldsImport(activeResult.value.fields, payload.jsonText, payload.mode),
    )
    updateActive({ fields: fields as InterfaceResponseField[] })
    ElMessage.success(t('workspace.interfaceForm.importJsonSuccess'))
  } catch (error) {
    if (error instanceof Error) {
      if (error.message === 'empty') {
        ElMessage.warning(t('workspace.interfaceForm.importJsonEmpty'))
        return
      }
      if (error.message === 'root-object') {
        ElMessage.warning(t('workspace.interfaceForm.importJsonRootInvalid'))
        return
      }
    }
    ElMessage.warning(t('workspace.interfaceForm.importJsonInvalid'))
  }
}

watch(
  () => props.modelValue.length,
  (length) => {
    if (length > 0 && activeIndex.value >= length) {
      activeIndex.value = length - 1
    }
  },
)

watch(
  () => activeResult.value?.format,
  (format) => {
    if (!format || formatOptions.includes(format)) return
    updateActive({
      format: format === 'HTML' || format === 'Text' ? 'Raw' : 'JSON',
    })
  },
)
</script>

<template>
  <div class="interface-response-results">
    <div class="interface-result-tabs">
      <div class="interface-result-tabs__items">
        <button
          v-for="(result, index) in modelValue"
          :key="index"
          type="button"
          class="interface-submodule-tabs__tab"
          :class="{ 'interface-submodule-tabs__tab--active': activeIndex === index }"
          @click="activeIndex = index"
        >
          {{ resultTabLabel(result) }}
        </button>
      </div>
      <button
        v-if="!readonly"
        type="button"
        class="interface-result-tabs__add"
        :aria-label="t('workspace.interfaceForm.addResult')"
        @click="addResult"
      >
        +
      </button>
    </div>

    <div v-if="activeResult" class="interface-response-results__content">
      <div class="interface-response-results__toolbar">
        <el-input
          :model-value="activeResult.name"
          class="interface-response-results__name interface-response-results__boxed"
          :placeholder="t('workspace.interfaceForm.resultName')"
          :readonly="readonly"
          @update:model-value="updateActive({ name: $event })"
        />
        <el-autocomplete
          :model-value="statusCodeInput"
          class="interface-response-results__status interface-response-results__boxed"
          :fetch-suggestions="searchStatusCodes"
          :trigger-on-focus="true"
          clearable
          :disabled="readonly"
          :placeholder="t('workspace.interfaceForm.statusCodePlaceholder')"
          popper-class="app-action-dropdown"
          @update:model-value="updateStatusCode($event ?? '')"
          @blur="commitStatusCode"
        />
        <el-select
          :model-value="activeResult.format"
          class="interface-response-results__format interface-response-results__boxed"
          :disabled="readonly"
          popper-class="app-action-dropdown"
          @update:model-value="updateActive({ format: $event })"
        >
          <el-option v-for="item in formatOptions" :key="item" :label="item" :value="item" />
        </el-select>
        <el-select
          :model-value="activeResult.data_type"
          class="interface-response-results__data-type interface-response-results__boxed"
          :disabled="readonly"
          popper-class="app-action-dropdown"
          @update:model-value="updateActive({ data_type: $event })"
        >
          <el-option v-for="item in dataTypeOptions" :key="item" :label="item" :value="item" />
        </el-select>
        <button
          v-if="!readonly"
          type="button"
          class="interface-body-import-btn"
          @click="importVisible = true"
        >
          {{ t('workspace.interfaceForm.importJson') }}
        </button>
        <el-tooltip
          v-if="!readonly && modelValue.length > 1"
          :content="t('common.delete')"
          placement="top"
          :show-after="0"
          :hide-after="0"
        >
          <button
            type="button"
            class="interface-response-results__delete"
            :aria-label="t('common.delete')"
            @click="removeResult(activeIndex)"
          >
            <el-icon :size="14"><Delete /></el-icon>
          </button>
        </el-tooltip>
      </div>

      <InterfaceRequestBodyImportDialog
        v-model:visible="importVisible"
        @apply="handleImportApply"
      />

      <InterfaceFieldTreeTable v-model="activeFields" :readonly="readonly" />
    </div>
  </div>
</template>
