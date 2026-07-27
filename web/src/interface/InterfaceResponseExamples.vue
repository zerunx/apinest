<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import InterfaceRequestBodyRaw from '@/interface/InterfaceRequestBodyRaw.vue'
import { useLocale } from '@/composables/useLocale'
import {
  contentTypeToExampleFormat,
  enrichResponseExample,
  exampleFormatToHighlightType,
  normalizeResponseExampleForSave,
} from '@/utils/response-example-format'
import {
  parseHttpStatusCodeInput,
  searchResponseStatusCodes,
} from '@/utils/response-status-code-options'
import type { InterfaceRequestBody, InterfaceResponseExample } from '@/types/workspace'

interface StatusCodeSuggestion {
  value: string
}

const props = defineProps<{
  modelValue: InterfaceResponseExample[]
  readonly?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: InterfaceResponseExample[]]
}>()

const { t } = useLocale()
const activeIndex = ref(0)

const formatOptions = ['JSON', 'XML', 'Raw']
const dataTypeOptions = ['Object', 'Array', 'String', 'Number', 'Boolean']

const activeExample = computed(() => props.modelValue[activeIndex.value] ?? null)

const statusCodeInput = ref('')

watch(
  [activeIndex, () => activeExample.value?.status_code],
  () => {
    statusCodeInput.value = activeExample.value ? String(activeExample.value.status_code) : ''
  },
  { immediate: true },
)

function searchStatusCodes(query: string, cb: (results: StatusCodeSuggestion[]) => void) {
  cb(searchResponseStatusCodes(query))
}

function syncExamples(examples: InterfaceResponseExample[]) {
  emit('update:modelValue', examples)
}

function updateActive(patch: Partial<InterfaceResponseExample>) {
  if (!activeExample.value) return
  const examples = props.modelValue.map((item, index) =>
    index === activeIndex.value ? normalizeResponseExampleForSave({ ...item, ...patch }) : item,
  )
  syncExamples(examples)
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
  statusCodeInput.value = activeExample.value ? String(activeExample.value.status_code) : ''
}

function createExample(name: string, statusCode: number): InterfaceResponseExample {
  const derived = contentTypeToExampleFormat('application/json')
  return normalizeResponseExampleForSave({
    name,
    status_code: statusCode,
    content_type: 'application/json',
    format: derived.format,
    data_type: derived.data_type,
    raw: '',
  })
}

function defaultAddedExample(): InterfaceResponseExample {
  return createExample(t('workspace.interfaceForm.failureExample'), 500)
}

function addExample() {
  const examples = [...props.modelValue, defaultAddedExample()]
  syncExamples(examples)
  activeIndex.value = examples.length - 1
}

function removeExample(index: number) {
  if (props.modelValue.length <= 1) return
  const examples = props.modelValue.filter((_, i) => i !== index)
  syncExamples(examples)
  if (activeIndex.value >= examples.length) {
    activeIndex.value = examples.length - 1
  }
}

function exampleTabLabel(example: InterfaceResponseExample) {
  const name = example.name.trim() || t('workspace.interfaceForm.responseExample')
  return `${name} (${example.status_code})`
}

const rawBody = computed({
  get(): InterfaceRequestBody {
    const example = activeExample.value
    if (!example) {
      return { format: 'raw', data_type: 'JSON', raw: '', fields: [] }
    }
    const enriched = enrichResponseExample(example)
    return {
      format: 'raw',
      data_type: exampleFormatToHighlightType(enriched.format ?? 'JSON', enriched.data_type ?? 'Object'),
      raw: example.raw,
      fields: [],
    }
  },
  set(body: InterfaceRequestBody) {
    updateActive({ raw: body.raw ?? '' })
  },
})

watch(
  () => props.modelValue.length,
  (length) => {
    if (length > 0 && activeIndex.value >= length) {
      activeIndex.value = length - 1
    }
  },
)

watch(
  () => activeExample.value?.format,
  (format) => {
    if (!format || formatOptions.includes(format)) return
    updateActive({
      format: format === 'HTML' || format === 'Text' ? 'Raw' : 'JSON',
    })
  },
)
</script>

<template>
  <div class="interface-response-examples">
    <div class="interface-result-tabs">
      <div class="interface-result-tabs__items">
        <button
          v-for="(example, index) in modelValue"
          :key="index"
          type="button"
          class="interface-submodule-tabs__tab"
          :class="{ 'interface-submodule-tabs__tab--active': activeIndex === index }"
          @click="activeIndex = index"
        >
          {{ exampleTabLabel(example) }}
        </button>
      </div>
      <button
        v-if="!readonly"
        type="button"
        class="interface-result-tabs__add"
        :aria-label="t('workspace.interfaceForm.addExample')"
        @click="addExample"
      >
        +
      </button>
    </div>

    <div v-if="activeExample" class="interface-response-examples__content">
      <div class="interface-response-results__toolbar">
        <el-input
          :model-value="activeExample.name"
          class="interface-response-results__name interface-response-results__boxed"
          :placeholder="t('workspace.interfaceForm.exampleName')"
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
          :model-value="activeExample.format ?? 'JSON'"
          class="interface-response-results__format interface-response-results__boxed"
          :disabled="readonly"
          popper-class="app-action-dropdown"
          @update:model-value="updateActive({ format: $event })"
        >
          <el-option v-for="item in formatOptions" :key="item" :label="item" :value="item" />
        </el-select>
        <el-select
          :model-value="activeExample.data_type ?? 'Object'"
          class="interface-response-results__data-type interface-response-results__boxed"
          :disabled="readonly"
          popper-class="app-action-dropdown"
          @update:model-value="updateActive({ data_type: $event })"
        >
          <el-option v-for="item in dataTypeOptions" :key="item" :label="item" :value="item" />
        </el-select>
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
            @click="removeExample(activeIndex)"
          >
            <el-icon :size="14"><Delete /></el-icon>
          </button>
        </el-tooltip>
      </div>

      <div class="interface-response-examples__raw">
        <InterfaceRequestBodyRaw v-model="rawBody" :readonly="readonly" hide-content-type />
      </div>
    </div>
  </div>
</template>
