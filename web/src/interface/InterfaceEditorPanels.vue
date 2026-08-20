<script setup lang="ts">
import { computed, reactive, watch } from 'vue'
import { ArrowDown } from '@element-plus/icons-vue'
import InterfaceRequestParams from '@/interface/InterfaceRequestParams.vue'
import InterfaceParamTable from '@/interface/InterfaceParamTable.vue'
import InterfaceFieldTreeTable from '@/interface/InterfaceFieldTreeTable.vue'
import InterfaceRequestBodyRaw from '@/interface/InterfaceRequestBodyRaw.vue'
import InterfaceResponseExamples from '@/interface/InterfaceResponseExamples.vue'
import InterfaceResponseResults from '@/interface/InterfaceResponseResults.vue'
import { useLocale } from '@/composables/useLocale'
import {
  hasQueryParamsContent,
  hasRequestBodyContent,
  hasRequestHeadersContent,
  hasRequestParamsContent,
  hasResponseExamplesContent,
  hasResponseHeadersContent,
  hasResponseResultsContent,
} from '@/utils/interface-editor-form'
import { COMMON_REQUEST_HEADERS } from '@/utils/request-header-options'
import { COMMON_RESPONSE_HEADERS } from '@/utils/response-header-options'
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
    /** When false, wait before applying the initial open/closed state. */
    ready?: boolean
  }>(),
  {
    defaultOpen: false,
    ready: true,
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

type PanelKey =
  | 'requestParams'
  | 'requestHeader'
  | 'queryParams'
  | 'requestBody'
  | 'responseHeader'
  | 'responseResult'
  | 'responseExample'

const openPanels = reactive<Record<PanelKey, boolean>>({
  requestParams: false,
  requestHeader: false,
  queryParams: false,
  requestBody: false,
  responseHeader: false,
  responseResult: false,
  responseExample: false,
})

const hasHeaders = computed(() => hasRequestHeadersContent(props.requestHeaders))
const hasQuery = computed(() => hasQueryParamsContent(props.queryParams))
const hasBody = computed(() => hasRequestBodyContent(props.requestBody))
const hasRespHeaders = computed(() => hasResponseHeadersContent(props.responseHeaders))
const hasRespResults = computed(() => hasResponseResultsContent(props.responseResults))
const hasRespExamples = computed(() => hasResponseExamplesContent(props.responseExamples))
const isRawBody = computed(() => props.requestBody.format === 'raw')

function expandAllPanels() {
  openPanels.requestParams = true
  openPanels.responseHeader = true
  openPanels.responseResult = true
  openPanels.responseExample = true
}

function syncOpenFromContent() {
  if (props.readonly) {
    openPanels.requestHeader = hasRequestHeadersContent(props.requestHeaders)
    openPanels.queryParams = hasQueryParamsContent(props.queryParams)
    openPanels.requestBody = hasRequestBodyContent(props.requestBody)
  } else {
    openPanels.requestParams = hasRequestParamsContent(
      props.requestHeaders,
      props.queryParams,
      props.requestBody,
    )
  }
  openPanels.responseHeader = hasResponseHeadersContent(props.responseHeaders)
  openPanels.responseResult = hasResponseResultsContent(props.responseResults)
  openPanels.responseExample = hasResponseExamplesContent(props.responseExamples)
}

function togglePanel(key: PanelKey) {
  openPanels[key] = !openPanels[key]
}

// First open of an API / create session / doc|edit view: empty panels stay closed.
// After that, only the summary click changes open state.
watch(
  () => [props.panelStateKey, props.defaultOpen, props.readonly, props.ready] as const,
  () => {
    if (!props.ready) return
    if (props.defaultOpen) expandAllPanels()
    else syncOpenFromContent()
  },
  { immediate: true },
)
</script>

<template>
  <div class="interface-detail__panels">
    <template v-if="readonly">
      <section
        v-if="hasHeaders"
        class="interface-panel"
        :class="{ 'interface-panel--open': openPanels.requestHeader }"
      >
        <button
          type="button"
          class="interface-panel__summary"
          :aria-expanded="openPanels.requestHeader"
          @click="togglePanel('requestHeader')"
        >
          <span class="interface-panel__arrow"><el-icon :size="12"><ArrowDown /></el-icon></span>
          {{ t('workspace.interfaceForm.reqHeader') }}
        </button>
        <div class="interface-panel__collapse">
          <div class="interface-panel__body">
            <InterfaceParamTable
              :model-value="requestHeaders"
              readonly
              :name-suggestions="COMMON_REQUEST_HEADERS"
              :name-label="t('workspace.interfaceForm.headerLabel')"
              :name-placeholder="t('workspace.interfaceForm.reqHeaderPlaceholder')"
            />
          </div>
        </div>
      </section>

      <section
        v-if="hasQuery"
        class="interface-panel"
        :class="{ 'interface-panel--open': openPanels.queryParams }"
      >
        <button
          type="button"
          class="interface-panel__summary"
          :aria-expanded="openPanels.queryParams"
          @click="togglePanel('queryParams')"
        >
          <span class="interface-panel__arrow"><el-icon :size="12"><ArrowDown /></el-icon></span>
          {{ t('workspace.interfaceForm.queryParams') }}
        </button>
        <div class="interface-panel__collapse">
          <div class="interface-panel__body">
            <InterfaceParamTable :model-value="queryParams" readonly />
          </div>
        </div>
      </section>

      <section
        v-if="hasBody"
        class="interface-panel"
        :class="{ 'interface-panel--open': openPanels.requestBody }"
      >
        <button
          type="button"
          class="interface-panel__summary"
          :aria-expanded="openPanels.requestBody"
          @click="togglePanel('requestBody')"
        >
          <span class="interface-panel__arrow"><el-icon :size="12"><ArrowDown /></el-icon></span>
          {{ t('workspace.interfaceForm.reqBodyParams') }}
        </button>
        <div class="interface-panel__collapse">
          <div class="interface-panel__body">
            <InterfaceRequestBodyRaw
              v-if="isRawBody"
              :model-value="requestBody"
              readonly
              hide-content-type
            />
            <InterfaceFieldTreeTable v-else :model-value="requestBody.fields" readonly />
          </div>
        </div>
      </section>
    </template>

    <section
      v-else
      class="interface-panel"
      :class="{ 'interface-panel--open': openPanels.requestParams }"
    >
      <button
        type="button"
        class="interface-panel__summary"
        :aria-expanded="openPanels.requestParams"
        @click="togglePanel('requestParams')"
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
            @update:request-headers="emit('update:requestHeaders', $event)"
            @update:query-params="emit('update:queryParams', $event)"
            @update:request-body="emit('update:requestBody', $event)"
          />
        </div>
      </div>
    </section>

    <section
      v-if="!readonly || hasRespHeaders"
      class="interface-panel"
      :class="{ 'interface-panel--open': openPanels.responseHeader }"
    >
      <button
        type="button"
        class="interface-panel__summary"
        :aria-expanded="openPanels.responseHeader"
        @click="togglePanel('responseHeader')"
      >
        <span class="interface-panel__arrow"><el-icon :size="12"><ArrowDown /></el-icon></span>
        {{ t('workspace.interfaceForm.responseHeader') }}
      </button>
      <div class="interface-panel__collapse">
        <div class="interface-panel__body">
          <InterfaceParamTable
            :model-value="responseHeaders"
            :readonly="readonly"
            :name-suggestions="COMMON_RESPONSE_HEADERS"
            :name-label="t('workspace.interfaceForm.headerLabel')"
            :name-placeholder="t('workspace.interfaceForm.reqHeaderPlaceholder')"
            @update:model-value="emit('update:responseHeaders', $event)"
          />
        </div>
      </div>
    </section>

    <section
      v-if="!readonly || hasRespResults"
      class="interface-panel"
      :class="{ 'interface-panel--open': openPanels.responseResult }"
    >
      <button
        type="button"
        class="interface-panel__summary"
        :aria-expanded="openPanels.responseResult"
        @click="togglePanel('responseResult')"
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

    <section
      v-if="!readonly || hasRespExamples"
      class="interface-panel"
      :class="{ 'interface-panel--open': openPanels.responseExample }"
    >
      <button
        type="button"
        class="interface-panel__summary"
        :aria-expanded="openPanels.responseExample"
        @click="togglePanel('responseExample')"
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
