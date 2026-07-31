<script setup lang="ts">
import { reactive, watch } from 'vue'
import { ArrowDown } from '@element-plus/icons-vue'
import InterfaceRequestParams from '@/interface/InterfaceRequestParams.vue'
import InterfaceResponseHeaderTable from '@/interface/InterfaceResponseHeaderTable.vue'
import InterfaceResponseExamples from '@/interface/InterfaceResponseExamples.vue'
import InterfaceResponseResults from '@/interface/InterfaceResponseResults.vue'
import { useLocale } from '@/composables/useLocale'
import {
  hasRequestParamsContent,
  hasResponseExamplesContent,
  hasResponseHeadersContent,
  hasResponseResultsContent,
} from '@/utils/interface-editor-form'
import type { ParamRow } from '@/utils/interface-params'
import type {
  InterfaceRequestBody,
  InterfaceResponseExample,
  InterfaceResponseResult,
} from '@/types/workspace'

const props = withDefaults(
  defineProps<{
    requestHeaders: ParamRow[]
    queryParams: ParamRow[]
    requestBody: InterfaceRequestBody
    responseHeaders: ParamRow[]
    responseResults: InterfaceResponseResult[]
    responseExamples: InterfaceResponseExample[]
    readonly?: boolean
    /** Changes when switching API / create session so empty panels re-collapse. */
    panelStateKey?: string | number | null
    /** When true (create API), keep all four panels expanded by default. */
    defaultOpen?: boolean
  }>(),
  {
    defaultOpen: false,
  },
)

const emit = defineEmits<{
  'update:requestHeaders': [value: ParamRow[]]
  'update:queryParams': [value: ParamRow[]]
  'update:requestBody': [value: InterfaceRequestBody]
  'update:responseHeaders': [value: ParamRow[]]
  'update:responseResults': [value: InterfaceResponseResult[]]
  'update:responseExamples': [value: InterfaceResponseExample[]]
}>()

const { t } = useLocale()

const openPanels = reactive({
  requestParams: false,
  responseHeader: false,
  responseResult: false,
  responseExample: false,
})

function expandAllPanels() {
  openPanels.requestParams = true
  openPanels.responseHeader = true
  openPanels.responseResult = true
  openPanels.responseExample = true
}

function syncOpenFromContent() {
  openPanels.requestParams = hasRequestParamsContent(
    props.requestHeaders,
    props.queryParams,
    props.requestBody,
  )
  openPanels.responseHeader = hasResponseHeadersContent(props.responseHeaders)
  openPanels.responseResult = hasResponseResultsContent(props.responseResults)
  openPanels.responseExample = hasResponseExamplesContent(props.responseExamples)
}

// Create flow: open all panels once per session; user can still collapse manually.
watch(
  () => [props.panelStateKey, props.defaultOpen] as const,
  () => {
    if (props.defaultOpen) expandAllPanels()
    else syncOpenFromContent()
  },
  { immediate: true },
)

// Edit / doc: expand only sections that have content.
watch(
  () =>
    [
      hasRequestParamsContent(props.requestHeaders, props.queryParams, props.requestBody),
      hasResponseHeadersContent(props.responseHeaders),
      hasResponseResultsContent(props.responseResults),
      hasResponseExamplesContent(props.responseExamples),
    ] as const,
  () => {
    if (props.defaultOpen) return
    syncOpenFromContent()
  },
)
</script>

<template>
  <div class="interface-detail__panels">
    <section class="interface-panel" :class="{ 'interface-panel--open': openPanels.requestParams }">
      <button
        type="button"
        class="interface-panel__summary"
        :aria-expanded="openPanels.requestParams"
        @click="openPanels.requestParams = !openPanels.requestParams"
      >
        <span class="interface-panel__arrow"><el-icon :size="12"><ArrowDown /></el-icon></span>
        {{ t('workspace.interfaceForm.requestParams') }}
      </button>
      <div class="interface-panel__collapse">
        <div class="interface-panel__body">
          <InterfaceRequestParams
            :request-headers="requestHeaders"
            :query-params="queryParams"
            :request-body="requestBody"
            :readonly="readonly"
            @update:request-headers="emit('update:requestHeaders', $event)"
            @update:query-params="emit('update:queryParams', $event)"
            @update:request-body="emit('update:requestBody', $event)"
          />
        </div>
      </div>
    </section>

    <section class="interface-panel" :class="{ 'interface-panel--open': openPanels.responseHeader }">
      <button
        type="button"
        class="interface-panel__summary"
        :aria-expanded="openPanels.responseHeader"
        @click="openPanels.responseHeader = !openPanels.responseHeader"
      >
        <span class="interface-panel__arrow"><el-icon :size="12"><ArrowDown /></el-icon></span>
        {{ t('workspace.interfaceForm.responseHeader') }}
      </button>
      <div class="interface-panel__collapse">
        <div class="interface-panel__body">
          <InterfaceResponseHeaderTable
            :model-value="responseHeaders"
            :readonly="readonly"
            @update:model-value="emit('update:responseHeaders', $event)"
          />
        </div>
      </div>
    </section>

    <section class="interface-panel" :class="{ 'interface-panel--open': openPanels.responseResult }">
      <button
        type="button"
        class="interface-panel__summary"
        :aria-expanded="openPanels.responseResult"
        @click="openPanels.responseResult = !openPanels.responseResult"
      >
        <span class="interface-panel__arrow"><el-icon :size="12"><ArrowDown /></el-icon></span>
        {{ t('workspace.interfaceForm.responseResult') }}
      </button>
      <div class="interface-panel__collapse">
        <div class="interface-panel__body">
          <InterfaceResponseResults
            :model-value="responseResults"
            :readonly="readonly"
            @update:model-value="emit('update:responseResults', $event)"
          />
        </div>
      </div>
    </section>

    <section class="interface-panel" :class="{ 'interface-panel--open': openPanels.responseExample }">
      <button
        type="button"
        class="interface-panel__summary"
        :aria-expanded="openPanels.responseExample"
        @click="openPanels.responseExample = !openPanels.responseExample"
      >
        <span class="interface-panel__arrow"><el-icon :size="12"><ArrowDown /></el-icon></span>
        {{ t('workspace.interfaceForm.responseExample') }}
      </button>
      <div class="interface-panel__collapse">
        <div class="interface-panel__body">
          <InterfaceResponseExamples
            :model-value="responseExamples"
            :readonly="readonly"
            @update:model-value="emit('update:responseExamples', $event)"
          />
        </div>
      </div>
    </section>
  </div>
</template>
