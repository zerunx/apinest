<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useLocale } from '@/composables/useLocale'
import InterfaceFieldTreeTable from '@/interface/InterfaceFieldTreeTable.vue'
import InterfaceParamTable from '@/interface/InterfaceParamTable.vue'
import InterfaceRequestBodyImportDialog from '@/interface/InterfaceRequestBodyImportDialog.vue'
import InterfaceRequestBodyRaw from '@/interface/InterfaceRequestBodyRaw.vue'
import { compactFieldTree, emptyFieldNode, ensureTrailingEmptyRoot } from '@/utils/interface-field-tree'
import { compactParamRows, emptyParamRow, ensureTrailingEmptyRow, type ParamRow } from '@/utils/interface-params'
import {
  applyBodyFieldsImport,
  applyRawBodyImport,
  type BodyImportMode,
} from '@/utils/json-body-import'
import type { InterfaceBodyField, InterfaceRequestBody } from '@/types/workspace'

type RequestTab = 'header' | 'body' | 'query'

const props = defineProps<{
  requestHeaders: ParamRow[]
  queryParams: ParamRow[]
  requestBody: InterfaceRequestBody
  readonly?: boolean
}>()

const emit = defineEmits<{
  'update:requestHeaders': [value: ParamRow[]]
  'update:queryParams': [value: ParamRow[]]
  'update:requestBody': [value: InterfaceRequestBody]
}>()

const { t } = useLocale()
const activeTab = ref<RequestTab>('body')
const importVisible = ref(false)

const bodyFormatOptions = [
  { value: 'form-data', label: 'Form-data' },
  { value: 'json', label: 'JSON' },
  { value: 'raw', label: 'Raw' },
]

const tabs = computed(() => [
  { key: 'header' as const, label: t('workspace.interfaceForm.reqHeader') },
  { key: 'body' as const, label: t('workspace.interfaceForm.reqBody') },
  { key: 'query' as const, label: t('workspace.interfaceForm.queryParams') },
])

const isRawBody = computed(() => props.requestBody.format === 'raw')

const bodyFields = computed({
  get: () => props.requestBody.fields,
  set: (fields: InterfaceBodyField[]) => {
    emit('update:requestBody', { ...props.requestBody, fields })
  },
})

function initRows(rows: ParamRow[]) {
  return ensureTrailingEmptyRow(rows.length ? rows : [emptyParamRow()])
}

function initBodyFields(fields: InterfaceBodyField[]) {
  return ensureTrailingEmptyRoot(fields.length ? fields : [emptyFieldNode()])
}

watch(
  () => props.requestHeaders,
  (rows) => {
    if (!rows.length) emit('update:requestHeaders', initRows([]))
  },
  { immediate: true },
)

watch(
  () => props.queryParams,
  (rows) => {
    if (!rows.length) emit('update:queryParams', initRows([]))
  },
  { immediate: true },
)

watch(
  () => props.requestBody.fields,
  (fields) => {
    if (props.requestBody.format === 'raw') return
    if (!fields.length) {
      emit('update:requestBody', { ...props.requestBody, fields: initBodyFields([]) })
    }
  },
  { immediate: true },
)

const rawContentTypes = new Set(['JSON', 'Text', 'XML', 'HTML'])
const schemaDataTypes = new Set(['Object', 'Array', 'String', 'Number', 'Boolean'])

function updateBodyFormat(format: string) {
  const next: InterfaceRequestBody = { ...props.requestBody, format }
  if (format === 'raw') {
    if (!rawContentTypes.has(next.data_type)) {
      next.data_type = 'JSON'
    }
    if (next.raw === undefined) {
      next.raw = ''
    }
  } else if (rawContentTypes.has(next.data_type) || !schemaDataTypes.has(next.data_type)) {
    next.data_type = 'Object'
  }
  emit('update:requestBody', next)
}

function handleImportApply(payload: { mode: BodyImportMode; jsonText: string }) {
  try {
    if (isRawBody.value) {
      const raw = applyRawBodyImport(props.requestBody.raw ?? '', payload.jsonText, payload.mode)
      emit('update:requestBody', { ...props.requestBody, raw })
    } else {
      const fields = ensureTrailingEmptyRoot(
        applyBodyFieldsImport(props.requestBody.fields, payload.jsonText, payload.mode),
      )
      emit('update:requestBody', { ...props.requestBody, fields })
    }
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

defineExpose({
  compactHeaders: () => compactParamRows(props.requestHeaders),
  compactQuery: () => compactParamRows(props.queryParams),
  compactBodyFields: () => compactFieldTree(props.requestBody.fields),
})
</script>

<template>
  <div class="interface-request-params">
    <div class="interface-submodule-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        type="button"
        class="interface-submodule-tabs__tab"
        :class="{ 'interface-submodule-tabs__tab--active': activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <div v-if="activeTab === 'header'" class="interface-request-params__content">
      <InterfaceParamTable
        :model-value="requestHeaders"
        :readonly="readonly"
        @update:model-value="emit('update:requestHeaders', $event)"
      />
    </div>

    <div v-else-if="activeTab === 'query'" class="interface-request-params__content">
      <InterfaceParamTable
        :model-value="queryParams"
        :readonly="readonly"
        @update:model-value="emit('update:queryParams', $event)"
      />
    </div>

    <div v-else class="interface-request-params__content">
      <div class="interface-request-params__body-toolbar">
        <el-radio-group
          :model-value="requestBody.format"
          class="interface-body-format-radio"
          :class="{ 'interface-body-format-radio--readonly': readonly }"
          :disabled="readonly"
          @update:model-value="updateBodyFormat"
        >
          <el-radio
            v-for="item in bodyFormatOptions"
            :key="item.value"
            :value="item.value"
            class="interface-body-format-radio__item"
          >
            <span class="interface-body-format-radio__pill">
              <span class="interface-body-format-radio__label">{{ item.label }}</span>
            </span>
          </el-radio>
        </el-radio-group>
        <button
          v-if="!readonly"
          type="button"
          class="interface-body-import-btn"
          @click="importVisible = true"
        >
          {{ t('workspace.interfaceForm.importJson') }}
        </button>
      </div>
      <InterfaceRequestBodyImportDialog
        v-model:visible="importVisible"
        @apply="handleImportApply"
      />
      <InterfaceRequestBodyRaw
        v-if="isRawBody"
        :model-value="requestBody"
        :readonly="readonly"
        @update:model-value="emit('update:requestBody', $event)"
      />
      <InterfaceFieldTreeTable v-else v-model="bodyFields" :readonly="readonly" />
    </div>
  </div>
</template>
