<template>
  <div class="interface-editor">
    <div class="interface-create__section">
      <div class="interface-url-row">
        <div class="interface-url-bar">
          <div
            class="interface-url-bar__selects"
            :class="{ 'interface-url-bar__selects--single': isProtocolOnly }"
          >
            <div class="interface-url-bar__protocol-wrap">
              <MenuSelect
                :model-value="protocol"
                class="interface-url-bar__protocol"
                :options="protocolOptions"
                fit-menu
                :menu-width="isProtocolOnly ? 180 : 108"
                @update:model-value="onProtocolChange"
              />
            </div>
            <div v-if="!isProtocolOnly" class="interface-url-bar__method-wrap">
              <MethodPicker
                v-model="method"
                class="interface-url-bar__method"
                :options="methodOptions"
              />
            </div>
          </div>
          <div class="interface-url-bar__url-wrap">
            <div
              ref="urlHighlightRef"
              class="interface-url-bar__url-highlight"
              aria-hidden="true"
              v-html="highlightedUrlHtml"
            />
            <input
              ref="urlInputRef"
              v-model="displayUrl"
              class="interface-url-bar__url native-input"
              placeholder=""
              spellcheck="false"
              @scroll="syncUrlHighlightScroll"
              @paste="onUrlPaste"
              @input="onUrlInput"
              @keydown="onUrlKeydown"
              @mousemove="onUrlInputMouseMove"
              @mouseleave="onUrlInputMouseLeave"
            />
            <Teleport to="body">
              <div
                v-if="urlVarTip"
                class="url-var-tip"
                :class="{ 'url-var-tip--missing': urlVarTip.missing }"
                :style="urlVarTipStyle"
                @mouseenter="onUrlVarTipEnter"
                @mouseleave="onUrlVarTipLeave"
              >
                <div class="url-var-tip__key">{{ urlVarTip.key }}</div>
                <div class="url-var-tip__value">{{ urlVarTip.value }}</div>
              </div>
            </Teleport>
          </div>
        </div>
        <div class="interface-url-actions">
          <button
            class="btn btn-primary interface-url-actions__send"
            type="button"
            :class="{ 'is-busy': sending }"
            :disabled="sending"
            @click="onSend"
          >
            {{ sending ? t('request.sending') : t('request.send') }}
          </button>
          <button
            class="btn interface-url-actions__save"
            type="button"
            :class="{
              'is-busy': saveUi === 'saving',
              'is-saved': saveUi === 'saved',
            }"
            :disabled="saveUi === 'saving'"
            @click="onSave"
          >
            <span class="interface-url-actions__save-label">
              <template v-if="saveUi === 'saved'">
                <span class="interface-url-actions__save-check" aria-hidden="true">✓</span>
                {{ t('common.saved') }}
              </template>
              <template v-else>
                {{ t('common.save') }}
              </template>
            </span>
          </button>
          <button v-if="mode === 'create'" class="btn" type="button" @click="$emit('cancel')">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>

      <div class="interface-meta-row">
        <MenuSelect
          v-model="folderId"
          class="interface-meta-row__folder"
          :options="folderOptions"
          :placeholder="t('create.selectFolder')"
        />
        <input
          v-model="name"
          class="interface-meta-row__name native-input"
          :placeholder="t('create.apiNamePlaceholder')"
        />
      </div>
    </div>

    <div ref="splitContainerRef" class="interface-work-split">
      <section class="interface-request-pane">
        <div class="interface-request-tabs">
          <button
            v-for="tab in requestTabs"
            :key="tab.id"
            type="button"
            class="interface-request-tabs__tab"
            :class="{ 'interface-request-tabs__tab--active': requestTab === tab.id }"
            @click="requestTab = tab.id"
          >
            {{ t(tab.labelKey) }}
          </button>
        </div>

        <div class="interface-request-content">
          <div v-if="requestTab === 'params'" class="param-table param-table--resizable-wrap">
            <ParamTableColumnResizer
              :columns="paramGridColumns"
              :handles="[false, 'key', false, false, false]"
              @resize="startTableColResize"
            />
            <div
              v-for="(row, idx) in queryParams"
              :key="idx"
              class="param-table__row"
              :class="{ 'param-table__row--off': row.enabled === false }"
              :style="{ gridTemplateColumns: paramGridColumns }"
            >
              <label class="param-table__check">
                <input v-model="row.enabled" type="checkbox" />
              </label>
              <input
                v-model="row.name"
                class="param-table__cell"
                :placeholder="t('common.key')"
                @input="onQueryParamChange(idx)"
              />
              <input
                v-model="row.example"
                class="param-table__cell"
                :placeholder="t('common.value')"
                @input="onQueryParamChange(idx)"
              />
              <input
                v-model="row.description"
                class="param-table__cell"
                :placeholder="t('create.param.desc')"
                @input="onQueryParamChange(idx)"
              />
              <button
                type="button"
                class="param-table__del"
                :disabled="queryParams.length <= 1"
                @click="removeQueryRow(idx)"
              >
                ×
              </button>
            </div>
          </div>

          <div v-else-if="requestTab === 'authorization'" class="auth-pane">
            <div class="auth-pane__layout">
              <div class="auth-pane__left">
                <span class="auth-pane__label">{{ t('request.authType') }}</span>
                <MenuSelect
                  v-model="authType"
                  compact
                  class="auth-pane__type"
                  :menu-width="168"
                  :options="authTypeOptions"
                />
                <p v-if="authType !== 'none'" class="auth-pane__hint muted">{{ authTypeHint }}</p>
              </div>
              <div class="auth-pane__divider" />
              <div class="auth-pane__right">
                <template v-if="authType === 'none'">
                  <span class="auth-pane__label auth-pane__label--spacer" aria-hidden="true">&nbsp;</span>
                  <p class="auth-pane__hint auth-pane__hint--none muted">{{ t('request.authNoneHint') }}</p>
                </template>
                <template v-else-if="authType === 'bearer'">
                  <span class="auth-pane__label">{{ t('request.authToken') }}</span>
                  <input
                    v-model="authBearerToken"
                    class="native-input auth-pane__input"
                    :placeholder="t('request.authToken')"
                  />
                </template>
                <template v-else-if="authType === 'basic'">
                  <span class="auth-pane__label">{{ t('request.authUsername') }}</span>
                  <input
                    v-model="authBasicUser"
                    class="native-input auth-pane__input"
                    :placeholder="t('request.authUsername')"
                  />
                  <span class="auth-pane__label">{{ t('request.authPassword') }}</span>
                  <input
                    v-model="authBasicPassword"
                    class="native-input auth-pane__input"
                    type="password"
                    :placeholder="t('request.authPassword')"
                  />
                </template>
              </div>
            </div>
          </div>

          <div v-else-if="requestTab === 'headers'" class="param-table param-table--resizable-wrap">
            <ParamTableColumnResizer
              :columns="paramGridColumns"
              :handles="[false, 'key', false, false, false]"
              @resize="startTableColResize"
            />
            <div
              v-for="(row, idx) in requestHeaders"
              :key="idx"
              class="param-table__row"
              :class="{ 'param-table__row--off': row.enabled === false }"
              :style="{ gridTemplateColumns: paramGridColumns }"
            >
              <label class="param-table__check">
                <input v-model="row.enabled" type="checkbox" />
              </label>
              <ComboInput
                v-model="row.name"
                class="param-table__combo"
                :options="headerKeySuggestions"
                :placeholder="t('common.key')"
                @input="onHeaderChange(idx)"
              />
              <input
                v-model="row.example"
                class="param-table__cell"
                :placeholder="t('common.value')"
                @input="onHeaderChange(idx)"
              />
              <input
                v-model="row.description"
                class="param-table__cell"
                :placeholder="t('create.param.desc')"
                @input="onHeaderChange(idx)"
              />
              <button
                type="button"
                class="param-table__del"
                :disabled="requestHeaders.length <= 1"
                @click="removeHeaderRow(idx)"
              >
                ×
              </button>
            </div>
          </div>

          <div v-else-if="requestTab === 'body'" class="body-pane">
            <div class="body-format-row">
              <div class="body-format-switch">
                <button
                  v-for="fmt in bodyFormats"
                  :key="fmt.value"
                  type="button"
                  class="body-format-switch__btn"
                  :class="{ 'body-format-switch__btn--active': bodyFormat === fmt.value }"
                  @click="bodyFormat = fmt.value"
                >
                  {{ fmt.label }}
                </button>
              </div>
              <button
                v-if="bodyFormat === 'raw'"
                type="button"
                class="body-format-action"
                @click="formatBodyRaw"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
                  <path
                    fill="currentColor"
                    d="M7 5h10v2H7V5zm0 4h10v2H7V9zm0 4h7v2H7v-2zm-1 6l-4-4h3v-4h2v4h3l-4 4z"
                  />
                </svg>
                {{ t('request.rawFormat') }}
              </button>
            </div>

            <p v-if="bodyFormat === 'none'" class="body-none-hint muted">
              {{ t('request.bodyNoneHint') }}
            </p>

            <div v-else-if="bodyFormat === 'form-data'" class="param-table param-table--resizable-wrap">
              <ParamTableColumnResizer
                :columns="paramGridColumns"
                :handles="[false, 'key', false, false, false]"
                @resize="startTableColResize"
              />
              <div
                v-for="(row, idx) in formDataFields"
                :key="idx"
                class="param-table__row"
                :class="{ 'param-table__row--off': row.enabled === false }"
                :style="{ gridTemplateColumns: paramGridColumns }"
              >
                <label class="param-table__check">
                  <input v-model="row.enabled" type="checkbox" />
                </label>
                <div class="param-table__key-cell">
                  <input
                    v-model="row.name"
                    class="param-table__cell param-table__cell--inner param-table__cell--key"
                    :placeholder="t('common.key')"
                    @input="onFormDataChange(idx)"
                  />
                  <MenuSelect
                    v-model="row.type"
                    compact
                    stretch
                    fit-menu
                    dense
                    menu-align="right"
                    :menu-min-width="88"
                    class="param-table__key-type"
                    :options="formDataTypeOptions"
                    @update:model-value="onFormDataTypeChange(idx)"
                  />
                </div>
                <div class="param-table__value">
                  <input
                    v-if="row.type !== 'file'"
                    v-model="row.example"
                    class="param-table__cell param-table__cell--inner"
                    :placeholder="t('common.value')"
                    @input="onFormDataChange(idx)"
                  />
                  <label v-else class="param-table__file">
                    <input
                      type="file"
                      class="param-table__file-input"
                      @change="onFormDataFileChange(idx, $event)"
                    />
                    <span class="param-table__file-name" :class="{ filled: !!row.example }">
                      {{ row.example || t('request.formDataSelectFile') }}
                    </span>
                  </label>
                </div>
                <input
                  v-model="row.description"
                  class="param-table__cell"
                  :placeholder="t('create.param.desc')"
                  @input="onFormDataChange(idx)"
                />
                <button
                  type="button"
                  class="param-table__del"
                  :disabled="formDataFields.length <= 1"
                  @click="removeFormDataRow(idx)"
                >
                  ×
                </button>
              </div>
            </div>

            <RequestBodyRawEditor
              v-else-if="bodyFormat === 'raw'"
              ref="rawBodyEditorRef"
              v-model="bodyRaw"
              :placeholder="t('create.bodyPlaceholder')"
            />
          </div>
        </div>
      </section>

      <div
        class="interface-split-handle"
        :class="{ 'interface-split-handle--active': responseDragging }"
        @mousedown="startResponseDrag"
      />

      <section class="interface-response-pane" :style="{ height: `${responseHeight}px` }">
        <div class="interface-response-bar">
          <div class="interface-response-tabs">
            <button
              type="button"
              class="interface-response-tabs__tab"
              :class="{ 'interface-response-tabs__tab--active': responseTab === 'body' }"
              @click="responseTab = 'body'"
            >
              {{ t('request.body') }}
            </button>
            <button
              type="button"
              class="interface-response-tabs__tab"
              :class="{ 'interface-response-tabs__tab--active': responseTab === 'headers' }"
              @click="responseTab = 'headers'"
            >
              {{ t('request.headers') }}
              <span v-if="response?.headers?.length" class="interface-response-tabs__badge">
                {{ response.headers.length }}
              </span>
            </button>
          </div>
          <div v-if="response && !sending" class="interface-response-bar__right">
            <div
              v-if="responseTab === 'body'"
              class="body-format-switch interface-response-type-switch"
            >
              <button
                v-for="fmt in responseBodyTypes"
                :key="fmt.value"
                type="button"
                class="body-format-switch__btn"
                :class="{ 'body-format-switch__btn--active': responseBodyType === fmt.value }"
                @click="responseBodyType = fmt.value"
              >
                {{ fmt.label }}
              </button>
            </div>
            <div class="interface-response-meta">
              <span class="interface-response-meta__status" :class="responseStatusClass">
                {{ response.status }} {{ response.status_text }}
              </span>
              <span class="interface-response-meta__sep" aria-hidden="true" />
              <span class="interface-response-meta__item">{{ response.duration_ms }} ms</span>
              <span class="interface-response-meta__sep" aria-hidden="true" />
              <span class="interface-response-meta__item">{{ responseBodySize }}</span>
            </div>
          </div>
        </div>

        <div class="interface-response-content">
          <div v-if="sending" class="interface-response-loading" role="status" aria-live="polite">
            <div class="interface-response-loading__spinner" aria-hidden="true" />
            <p class="interface-response-loading__text">{{ t('request.sending') }}</p>
          </div>
          <template v-else-if="response">
            <RequestBodyRawEditor
              v-if="responseTab === 'body'"
              :model-value="responseDisplayBody"
              :content-type="responseBodyType"
              readonly
              class="interface-response-raw"
            />
            <div v-else class="interface-response-headers-table">
              <div class="interface-response-headers-table__head">
                <span>{{ t('common.key') }}</span>
                <span>{{ t('common.value') }}</span>
              </div>
              <div
                v-for="(h, i) in response.headers"
                :key="`${h.key}-${i}`"
                class="interface-response-headers-table__row"
              >
                <span class="interface-response-headers-table__key">{{ h.key }}</span>
                <span class="interface-response-headers-table__value">{{ h.value }}</span>
              </div>
            </div>
          </template>
          <p v-else class="interface-response-empty muted">{{ t('response.empty') }}</p>
        </div>
      </section>
    </div>
    <ConfirmDialog
      v-model:open="validationDialogOpen"
      :title="validationDialogTitle"
      :message="validationDialogMessage"
      variant="primary"
      :show-cancel="false"
      @confirm="validationDialogOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useContainerBoundedResize } from '../composables/useDragResize'
import { useColumnResize } from '../composables/useColumnResize'
import type { ParamRow } from './ParamTable.vue'
import MenuSelect from './MenuSelect.vue'
import MethodPicker from './MethodPicker.vue'
import ComboInput from './ComboInput.vue'
import ParamTableColumnResizer from './ParamTableColumnResizer.vue'
import RequestBodyRawEditor from './RequestBodyRawEditor.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import { formatJsonLenient } from '../utils/raw-json-utils'
import { looksLikeCurl, parseCurl, type ParsedCurl } from '../utils/parse-curl'
import {
  isKnownProtocol,
  isProtocolOnlySelect,
  methodsForProtocol,
  normalizeProtocol,
  protocolFamily,
  protocolLabel,
  switchableProtocols,
  type Protocol,
} from '../utils/http-method'
import { useToast } from '../composables/useToast'
import { useWorkspaceContext } from '../composables/useWorkspaceContext'
import { createTextUndo, matchUndoRedoKey } from '../composables/useTextUndo'
import type { SendResponse } from '../types'

export type FolderOption = { id: number; name: string; parent_id: number; label: string }

type QueryParamRow = ParamRow & { enabled: boolean }
type HeaderRow = ParamRow & { enabled: boolean }
type BodyFormat = 'none' | 'form-data' | 'raw'
type FormDataRow = ParamRow & { enabled: boolean }

const props = defineProps<{
  mode: 'create' | 'edit'
  workspaceId: number
  projectId: number
  interfaceId?: number | null
  initial?: {
    name?: string
    method?: string
    url?: string
    folder_id?: number
    status?: number
    detail?: Record<string, unknown> | null
  } | null
}>()

const emit = defineEmits<{
  saved: [id: number]
  cancel: []
  'state-change': []
  'editor-ready': []
}>()

const { t } = useI18n()
const { showToast } = useToast()
const { environmentId, envVarsRevision } = useWorkspaceContext()
const folderOptions = computed(() => [
  { value: 0, label: t('create.noFolder') },
  ...folders.value.map((f) => ({ value: f.id, label: f.label })),
])
const envVarKeys = ref<Set<string>>(new Set())
const envVarMap = ref<Record<string, string>>({})
const urlHighlightRef = ref<HTMLElement | null>(null)
const urlInputRef = ref<HTMLInputElement | null>(null)
const urlUndo = createTextUndo()
let applyingUrlUndo = false

type UrlVarTip = {
  key: string
  value: string
  missing: boolean
  left: number
  top: number
}
const urlVarTip = ref<UrlVarTip | null>(null)
const urlVarTipStyle = computed(() => {
  const tip = urlVarTip.value
  if (!tip) return {}
  return {
    left: `${tip.left}px`,
    top: `${tip.top}px`,
  }
})
let urlVarTipHideTimer: ReturnType<typeof setTimeout> | null = null
let urlVarTipShowTimer: ReturnType<typeof setTimeout> | null = null
const URL_VAR_TIP_SHOW_DELAY_MS = 420
const URL_VAR_TIP_HIDE_DELAY_MS = 200

const ENV_VAR_RE = /\{\{\s*([^{}]+?)\s*\}\}/g

type UrlVarRange = { start: number; end: number; key: string; ok: boolean }

type UrlVarHitBox = UrlVarRange & { left: number; right: number; bottom: number }

let urlInputMeasureCtx: CanvasRenderingContext2D | null = null
let urlInputMeasureFont = ''
let urlVarHitBoxes: UrlVarHitBox[] = []
let urlVarHitBoxesFor = ''
let urlVarHoverRaf = 0
let urlVarHoverKey: string | null = null
let urlVarTipPinned = false

function findUrlVarRanges(text: string): UrlVarRange[] {
  const ranges: UrlVarRange[] = []
  ENV_VAR_RE.lastIndex = 0
  let match: RegExpExecArray | null
  while ((match = ENV_VAR_RE.exec(text))) {
    const key = match[1].trim()
    ranges.push({
      start: match.index,
      end: match.index + match[0].length,
      key,
      ok: envVarKeys.value.has(key),
    })
  }
  return ranges
}

function urlInputTextCtx(input: HTMLInputElement): CanvasRenderingContext2D {
  if (!urlInputMeasureCtx) {
    const canvas = document.createElement('canvas')
    urlInputMeasureCtx = canvas.getContext('2d')!
  }
  const style = getComputedStyle(input)
  const font = `${style.fontStyle} ${style.fontVariant} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`
  if (font !== urlInputMeasureFont) {
    urlInputMeasureCtx.font = font
    urlInputMeasureFont = font
  }
  return urlInputMeasureCtx
}

function urlInputTextWidth(input: HTMLInputElement, text: string): number {
  return urlInputTextCtx(input).measureText(text).width
}

function rebuildUrlVarHitBoxes() {
  const input = urlInputRef.value
  const text = displayUrl.value
  urlVarHitBoxesFor = text
  if (!input || !text) {
    urlVarHitBoxes = []
    return
  }
  const rect = input.getBoundingClientRect()
  const padL = parseFloat(getComputedStyle(input).paddingLeft) || 0
  const bottom = rect.bottom + 6
  const scrollLeft = input.scrollLeft
  urlVarHitBoxes = findUrlVarRanges(text).map((r) => {
    const left =
      rect.left + padL + urlInputTextWidth(input, text.slice(0, r.start)) - scrollLeft
    const right =
      rect.left + padL + urlInputTextWidth(input, text.slice(0, r.end)) - scrollLeft
    return { ...r, left, right, bottom }
  })
}

function clearUrlVarTipHideTimer() {
  if (urlVarTipHideTimer == null) return
  clearTimeout(urlVarTipHideTimer)
  urlVarTipHideTimer = null
}

function clearUrlVarTipShowTimer() {
  if (urlVarTipShowTimer == null) return
  clearTimeout(urlVarTipShowTimer)
  urlVarTipShowTimer = null
}

function hideUrlVarTip() {
  clearUrlVarTipHideTimer()
  clearUrlVarTipShowTimer()
  if (urlVarHoverRaf) {
    cancelAnimationFrame(urlVarHoverRaf)
    urlVarHoverRaf = 0
  }
  urlVarTipPinned = false
  urlVarTip.value = null
  urlVarHoverKey = null
}

function scheduleHideUrlVarTip() {
  if (urlVarTipPinned) return
  clearUrlVarTipShowTimer()
  if (!urlVarTip.value) return
  clearUrlVarTipHideTimer()
  urlVarTipHideTimer = setTimeout(() => {
    urlVarTipHideTimer = null
    if (urlVarTipPinned) return
    urlVarTip.value = null
    urlVarHoverKey = null
  }, URL_VAR_TIP_HIDE_DELAY_MS)
}

function openUrlVarTip(hit: UrlVarHitBox) {
  clearUrlVarTipHideTimer()
  const rawValue = envVarMap.value[hit.key]
  let value: string
  if (!hit.ok) value = t('request.envVarUndefined')
  else if (rawValue == null || rawValue === '') value = t('request.envVarEmpty')
  else value = rawValue
  urlVarTip.value = {
    key: hit.key,
    value,
    missing: !hit.ok,
    left: (hit.left + hit.right) / 2,
    top: hit.bottom,
  }
}

function scheduleShowUrlVarTip(hit: UrlVarHitBox) {
  clearUrlVarTipHideTimer()
  // Already showing this key — keep it.
  if (urlVarTip.value?.key === hit.key) return
  clearUrlVarTipShowTimer()
  urlVarTipShowTimer = setTimeout(() => {
    urlVarTipShowTimer = null
    openUrlVarTip(hit)
  }, URL_VAR_TIP_SHOW_DELAY_MS)
}

function onUrlVarTipEnter() {
  urlVarTipPinned = true
  clearUrlVarTipHideTimer()
  clearUrlVarTipShowTimer()
}

function onUrlVarTipLeave() {
  urlVarTipPinned = false
  scheduleHideUrlVarTip()
}

function onUrlInputMouseMove(event: MouseEvent) {
  // Dragging to select text — skip heavy work (was freezing the UI).
  if (event.buttons !== 0) {
    if (urlVarHoverKey !== null || urlVarTip.value || urlVarTipShowTimer != null) {
      const input = urlInputRef.value
      if (input) input.style.cursor = ''
      hideUrlVarTip()
    }
    return
  }

  const clientX = event.clientX
  if (urlVarHoverRaf) cancelAnimationFrame(urlVarHoverRaf)
  urlVarHoverRaf = requestAnimationFrame(() => {
    urlVarHoverRaf = 0
    const input = urlInputRef.value
    if (!input) return

    const text = displayUrl.value
    if (urlVarHitBoxesFor !== text) rebuildUrlVarHitBoxes()

    const hit = urlVarHitBoxes.find((r) => clientX >= r.left && clientX < r.right)
    if (!hit) {
      if (urlVarHoverKey !== null || urlVarTipShowTimer != null) {
        urlVarHoverKey = null
        input.style.cursor = ''
        clearUrlVarTipShowTimer()
        scheduleHideUrlVarTip()
      }
      return
    }

    input.style.cursor = 'pointer'
    if (urlVarHoverKey === hit.key) {
      // Still over same var — cancel pending hide if any.
      clearUrlVarTipHideTimer()
      return
    }
    urlVarHoverKey = hit.key
    scheduleShowUrlVarTip(hit)
  })
}

function onUrlInputMouseLeave() {
  const input = urlInputRef.value
  if (input) input.style.cursor = ''
  urlVarHoverKey = null
  clearUrlVarTipShowTimer()
  // Delay hide so pointer can move onto the tip bubble.
  scheduleHideUrlVarTip()
}

function urlSnapshot() {
  const el = urlInputRef.value
  const text = displayUrl.value
  return {
    text,
    start: el?.selectionStart ?? text.length,
    end: el?.selectionEnd ?? text.length,
  }
}

function resetUrlUndo() {
  void nextTick(() => {
    urlUndo.reset(urlSnapshot())
  })
}

function onUrlInput() {
  if (applyingUrlUndo) return
  void nextTick(() => urlUndo.noteInput(urlSnapshot()))
}

function onUrlKeydown(event: KeyboardEvent) {
  const kind = matchUndoRedoKey(event)
  if (!kind) return
  event.preventDefault()
  const snap =
    kind === 'undo' ? urlUndo.undo(urlSnapshot()) : urlUndo.redo(urlSnapshot())
  if (!snap) return
  applyingUrlUndo = true
  parseUrlInput(snap.text)
  void nextTick(() => {
    const el = urlInputRef.value
    if (el) {
      const start = Math.max(0, Math.min(snap.start, el.value.length))
      const end = Math.max(0, Math.min(snap.end, el.value.length))
      try {
        el.setSelectionRange(start, end)
      } catch {
        /* ignore */
      }
    }
    applyingUrlUndo = false
  })
}

function escapeHtml(text: string) {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function applyEnvVars(input: string, vars: Record<string, string>) {
  if (!input || !Object.keys(vars).length) return input
  return input.replace(/\{\{\s*([^{}]+?)\s*\}\}/g, (match, key: string) => {
    const k = key.trim()
    return Object.prototype.hasOwnProperty.call(vars, k) ? vars[k] : match
  })
}

function syncUrlHighlightScroll(e: Event) {
  const el = e.target as HTMLInputElement
  if (urlHighlightRef.value) {
    urlHighlightRef.value.scrollLeft = el.scrollLeft
  }
  hideUrlVarTip()
  urlVarHoverKey = null
  urlVarHitBoxesFor = ''
}

async function refreshEnvVars() {
  if (!environmentId.value) {
    envVarKeys.value = new Set()
    envVarMap.value = {}
    return
  }
  try {
    const rows = await invoke<[string, string][]>('get_env_vars', {
      environmentId: environmentId.value,
    })
    const map: Record<string, string> = {}
    const keys = new Set<string>()
    for (const [k, v] of rows) {
      keys.add(k)
      map[k] = v
    }
    envVarKeys.value = keys
    envVarMap.value = map
  } catch {
    envVarKeys.value = new Set()
    envVarMap.value = {}
  }
}

watch([environmentId, envVarsRevision], () => {
  void refreshEnvVars()
})

type RequestTabId = 'params' | 'authorization' | 'headers' | 'body'

const allRequestTabs: { id: RequestTabId; labelKey: string }[] = [
  { id: 'params', labelKey: 'request.params' },
  { id: 'authorization', labelKey: 'request.authorization' },
  { id: 'headers', labelKey: 'request.headers' },
  { id: 'body', labelKey: 'request.body' },
]

const requestTabs = computed(() => {
  if (protocolFamily(protocol.value) === 'ws') {
    return allRequestTabs.filter((tab) => tab.id !== 'authorization')
  }
  return allRequestTabs
})

const headerKeySuggestions = [
  'Accept',
  'Accept-Language',
  'Authorization',
  'Cache-Control',
  'Connection',
  'Content-Type',
  'Cookie',
  'Host',
  'Origin',
  'Referer',
  'User-Agent',
  'X-Requested-With',
]

const authTypeOptions = computed(() => [
  { value: 'none', label: t('request.authNone') },
  { value: 'bearer', label: t('request.authBearer') },
  { value: 'basic', label: t('request.authBasic') },
])

const bodyFormats = computed(() => [
  { value: 'none' as const, label: t('request.bodyNone') },
  { value: 'form-data' as const, label: t('request.bodyFormData') },
  { value: 'raw' as const, label: t('request.bodyRaw') },
])

const formDataTypeOptions = computed(() => [
  { value: 'string', label: t('request.formDataText') },
  { value: 'file', label: t('request.formDataFile') },
])

const splitContainerRef = ref<HTMLElement | null>(null)
const {
  size: responseHeight,
  dragging: responseDragging,
  startDrag: startResponseDrag,
  clampToContainer: clampResponseHeight,
} = useContainerBoundedResize(splitContainerRef, {
  axis: 'y',
  min: 120,
  initial: 280,
  maxRatio: 0.75,
  storageKey: 'interface_response_height',
})

const folders = ref<FolderOption[]>([])
const saveUi = ref<'idle' | 'saving' | 'saved'>('idle')
const sending = ref(false)
let saveUiTimer: ReturnType<typeof setTimeout> | null = null
const protocol = ref<Protocol>('HTTP')
const method = ref('POST')
const isCreateMode = computed(() => props.mode === 'create')
const isProtocolOnly = computed(() => isProtocolOnlySelect(protocol.value))
const protocolOptions = computed(() =>
  switchableProtocols(protocol.value, isCreateMode.value).map((p) => ({
    value: p,
    label: protocolLabel(p),
  })),
)
const methodOptions = computed(() => methodsForProtocol(protocol.value))

function onProtocolChange(value: string | number | null) {
  const next = normalizeProtocol(value)
  if (next === protocol.value) return
  if (!isCreateMode.value) {
    const allowed = switchableProtocols(protocol.value, false)
    if (!allowed.includes(next)) return
  }
  protocol.value = next
}

// Keep method valid when protocol changes (picker or URL scheme).
watch(protocol, (next) => {
  const allowed = methodsForProtocol(next)
  if (!allowed.includes(method.value)) {
    method.value = allowed[0] ?? 'GET'
  }
  // WS has no Authorization tab — leave it if currently open.
  if (protocolFamily(next) === 'ws' && requestTab.value === 'authorization') {
    requestTab.value = 'body'
  }
})

const urlBase = ref('')
const name = ref('')
const folderId = ref<string | number | null>(null)
const requestTab = ref<RequestTabId>('body')
const bodyFormat = ref<BodyFormat>('none')
const bodyRaw = ref('')
const rawBodyEditorRef = ref<{ formatContent: () => boolean } | null>(null)
const validationDialogOpen = ref(false)
const validationDialogTitle = ref('')
const validationDialogMessage = ref('')

function showValidation(message: string, title?: string) {
  validationDialogTitle.value = title ?? t('create.validationTitle')
  validationDialogMessage.value = message
  validationDialogOpen.value = true
}
const formDataFields = ref<FormDataRow[]>([emptyParamRow()])
const {
  widths: tableColWidths,
  start: startTableColResize,
} = useColumnResize(
  'request_table_col_widths_v4',
  { key: 280 },
  { key: 160 },
)

const paramGridColumns = computed(
  () => `40px ${tableColWidths.value.key}px minmax(160px, 1fr) minmax(160px, 1fr) 36px`,
)

const requestHeaders = ref<HeaderRow[]>([defaultHeaderRow()])
const queryParams = ref<QueryParamRow[]>([defaultQueryRow()])
const responseHeaders = ref<ParamRow[]>([emptyRow()])
const resultName = ref('')
const resultStatus = ref('200')
const resultSchema = ref('')
const exampleName = ref('')
const exampleStatus = ref('200')
const exampleBody = ref('')

const authType = ref<'none' | 'bearer' | 'basic'>('none')
const authBearerToken = ref('')
const authBasicUser = ref('')
const authBasicPassword = ref('')

const authTypeHint = computed(() => {
  if (authType.value === 'bearer') return t('request.authBearerHint')
  if (authType.value === 'basic') return t('request.authBasicHint')
  return t('request.authNoneHint')
})

type ResponseBodyType = 'JSON' | 'Text'

const response = ref<SendResponse | null>(null)
const responseTab = ref<'body' | 'headers'>('body')
const responseBodyType = ref<ResponseBodyType>('JSON')
const responseBodyTypes = computed(() => [
  { value: 'JSON' as const, label: t('response.bodyJson') },
  { value: 'Text' as const, label: t('response.bodyText') },
])

const responseDisplayBody = computed(() => {
  const raw = response.value?.body ?? ''
  if (responseBodyType.value !== 'JSON') return raw
  return formatJsonLenient(raw.trim()) ?? raw
})

const responseBodySize = computed(() => {
  const bytes = new TextEncoder().encode(response.value?.body ?? '').length
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
})

function detectResponseBodyType(body: string): ResponseBodyType {
  const trimmed = body.trim()
  if (!trimmed) return 'Text'
  if (
    (trimmed.startsWith('{') && trimmed.endsWith('}')) ||
    (trimmed.startsWith('[') && trimmed.endsWith(']'))
  ) {
    if (formatJsonLenient(trimmed)) return 'JSON'
  }
  return 'Text'
}

const responseStatusClass = computed(() => {
  const s = response.value?.status ?? 0
  if (s >= 200 && s < 300) return 'ok'
  if (s >= 400 || s === 0) return 'err'
  return 'warn'
})

function emptyRow(): ParamRow {
  return { name: '', type: 'string', required: false, description: '', example: '' }
}

function defaultHeaderRow(): HeaderRow {
  return { ...emptyRow(), name: 'Content-Type', example: 'application/json', enabled: true }
}

function emptyParamRow(): QueryParamRow {
  return { ...emptyRow(), enabled: true }
}

function defaultQueryRow(): QueryParamRow {
  return emptyParamRow()
}

function isParamRowEmpty(row: Pick<QueryParamRow, 'name' | 'example' | 'description'>) {
  return !row.name.trim() && !row.example.trim() && !row.description.trim()
}

function normalizeQueryParams() {
  const rows = queryParams.value
  while (
    rows.length > 1 &&
    isParamRowEmpty(rows[rows.length - 1]) &&
    isParamRowEmpty(rows[rows.length - 2])
  ) {
    rows.pop()
  }
  if (!rows.length || !isParamRowEmpty(rows[rows.length - 1])) {
    rows.push(emptyParamRow())
  }
}

function normalizeRequestHeaders() {
  const rows = requestHeaders.value
  while (
    rows.length > 1 &&
    isParamRowEmpty(rows[rows.length - 1]) &&
    isParamRowEmpty(rows[rows.length - 2])
  ) {
    rows.pop()
  }
  if (!rows.length || !isParamRowEmpty(rows[rows.length - 1])) {
    rows.push(emptyParamRow())
  }
}

function normalizeFormDataFields() {
  const rows = formDataFields.value
  while (
    rows.length > 1 &&
    isParamRowEmpty(rows[rows.length - 1]) &&
    isParamRowEmpty(rows[rows.length - 2])
  ) {
    rows.pop()
  }
  if (!rows.length || !isParamRowEmpty(rows[rows.length - 1])) {
    rows.push(emptyParamRow())
  }
}

function onFormDataChange(idx: number) {
  if (idx !== formDataFields.value.length - 1) return
  if (isParamRowEmpty(formDataFields.value[idx])) return
  normalizeFormDataFields()
}

function onFormDataTypeChange(idx: number) {
  const row = formDataFields.value[idx]
  if (row.type === 'file') row.example = ''
  onFormDataChange(idx)
}

function onFormDataFileChange(idx: number, event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  formDataFields.value[idx].example = file?.name ?? ''
  onFormDataChange(idx)
}

function removeFormDataRow(idx: number) {
  if (formDataFields.value.length <= 1) return
  formDataFields.value.splice(idx, 1)
  normalizeFormDataFields()
}

function hasFormDataContent(fields: FormDataRow[]) {
  return fields.some(
    (r) =>
      r.enabled !== false &&
      (r.name.trim() || r.example.trim() || r.description.trim()),
  )
}

function resolveBodyFormat(raw: string, fields: FormDataRow[]): BodyFormat {
  if (hasFormDataContent(fields)) return 'form-data'
  if (raw.trim()) return 'raw'
  return 'none'
}

function formatBodyRaw() {
  rawBodyEditorRef.value?.formatContent()
}

function buildRequestBodyForSend(): string | null {
  if (bodyFormat.value === 'none') return null
  if (bodyFormat.value === 'raw') return bodyRaw.value || null
  const fields = formDataFields.value.filter((r) => r.enabled !== false && r.name.trim())
  if (!fields.length) return null
  const params = new URLSearchParams()
  for (const row of fields) {
    params.append(row.name.trim(), row.example)
  }
  return params.toString()
}

function onQueryParamChange(idx: number) {
  if (idx !== queryParams.value.length - 1) return
  if (isParamRowEmpty(queryParams.value[idx])) return
  normalizeQueryParams()
}

function onHeaderChange(idx: number) {
  if (idx !== requestHeaders.value.length - 1) return
  if (isParamRowEmpty(requestHeaders.value[idx])) return
  normalizeRequestHeaders()
}

function removeQueryRow(idx: number) {
  if (queryParams.value.length <= 1) return
  queryParams.value.splice(idx, 1)
  normalizeQueryParams()
}

function removeHeaderRow(idx: number) {
  if (requestHeaders.value.length <= 1) return
  requestHeaders.value.splice(idx, 1)
  normalizeRequestHeaders()
}

/** Encode for URL bar display; keep `/` readable (do not show as %2F). */
function encodeQueryComponent(value: string) {
  return encodeURIComponent(value).replace(/%2F/gi, '/')
}

function decodeQueryComponent(value: string) {
  try {
    return decodeURIComponent(value.replace(/\+/g, ' '))
  } catch {
    return value.replace(/\+/g, ' ')
  }
}

function buildQuerySuffix() {
  const params = queryParams.value.filter((r) => r.enabled !== false && r.name.trim())
  if (!params.length) return ''
  const qs = params
    .map((r) => `${encodeQueryComponent(r.name.trim())}=${encodeQueryComponent(r.example)}`)
    .join('&')
  return `?${qs}`
}

const displayUrl = computed({
  get() {
    return `${urlBase.value}${buildQuerySuffix()}`
  },
  set(val: string) {
    parseUrlInput(val)
  },
})

const highlightedUrlHtml = computed(() => {
  const raw = displayUrl.value
  if (!raw) return ''
  let html = ''
  let last = 0
  ENV_VAR_RE.lastIndex = 0
  let match: RegExpExecArray | null
  while ((match = ENV_VAR_RE.exec(raw))) {
    html += escapeHtml(raw.slice(last, match.index))
    const full = match[0]
    const key = match[1].trim()
    const ok = envVarKeys.value.has(key)
    const end = match.index + full.length
    html += `<span class="url-var ${ok ? 'url-var--ok' : 'url-var--missing'}">${escapeHtml(full)}</span>`
    last = end
  }
  html += escapeHtml(raw.slice(last))
  return html || '&nbsp;'
})

watch(displayUrl, () => {
  hideUrlVarTip()
  urlVarHoverKey = null
  urlVarHitBoxesFor = ''
})

function clearQueryParams() {
  queryParams.value = [emptyParamRow()]
  normalizeQueryParams()
}

function applyParsedCurl(parsed: ParsedCurl) {
  method.value = (parsed.method || 'GET').toUpperCase()
  parseUrlInput(parsed.url || '')

  if (parsed.headers.length) {
    requestHeaders.value = parsed.headers.map((h) => ({
      ...emptyRow(),
      name: h.key,
      example: h.value,
      enabled: true,
    }))
  } else {
    requestHeaders.value = [emptyParamRow()]
  }
  normalizeRequestHeaders()

  // Prefer explicit Authorization header from curl over auth pane to avoid duplicates.
  authType.value = 'none'
  authBearerToken.value = ''
  authBasicUser.value = ''
  authBasicPassword.value = ''

  if (parsed.formData?.length) {
    bodyFormat.value = 'form-data'
    bodyRaw.value = ''
    formDataFields.value = parsed.formData.map((f) => ({
      ...emptyRow(),
      name: f.name,
      type: f.type,
      example: f.value,
      enabled: true,
    }))
    normalizeFormDataFields()
    requestTab.value = 'body'
  } else if (parsed.body != null && parsed.body !== '') {
    bodyFormat.value = 'raw'
    bodyRaw.value = parsed.body
    formDataFields.value = [emptyParamRow()]
    normalizeFormDataFields()
    requestTab.value = 'body'
  } else {
    bodyFormat.value = 'none'
    bodyRaw.value = ''
    formDataFields.value = [emptyParamRow()]
    normalizeFormDataFields()
    requestTab.value = parsed.headers.length ? 'headers' : 'params'
  }

  showToast(t('request.curlImported'), 1800, 'success')
}

function onUrlPaste(e: ClipboardEvent) {
  const text = e.clipboardData?.getData('text') ?? ''
  if (!looksLikeCurl(text)) return
  const parsed = parseCurl(text)
  if (!parsed) return
  e.preventDefault()
  urlUndo.checkpoint(urlSnapshot())
  applyParsedCurl(parsed)
  void nextTick(() => urlUndo.commit(urlSnapshot()))
}

function parseUrlInput(raw: string) {
  if (!raw.trim()) {
    urlBase.value = ''
    clearQueryParams()
    return
  }

  // Keep scheme in the URL bar; only update protocol metadata.
  // After create, only allow same-family protocol changes.
  const setProtocolFromUrl = (next: Protocol) => {
    if (isCreateMode.value || switchableProtocols(protocol.value, false).includes(next)) {
      protocol.value = next
    }
  }
  if (/^https:\/\//i.test(raw)) {
    setProtocolFromUrl('HTTPS')
  } else if (/^http:\/\//i.test(raw)) {
    setProtocolFromUrl('HTTP')
  } else if (/^wss:\/\//i.test(raw)) {
    setProtocolFromUrl('WS')
  } else if (/^ws:\/\//i.test(raw)) {
    setProtocolFromUrl('WS')
  } else if (/^tcp:\/\//i.test(raw)) {
    setProtocolFromUrl('TCP')
  } else if (/^udp:\/\//i.test(raw)) {
    setProtocolFromUrl('UDP')
  }

  const hashIndex = raw.indexOf('#')
  const withoutHash = hashIndex >= 0 ? raw.slice(0, hashIndex) : raw
  const hash = hashIndex >= 0 ? raw.slice(hashIndex) : ''
  const qIndex = withoutHash.indexOf('?')

  if (qIndex === -1) {
    // Keep exactly what was typed (e.g. "http://"); never force "/".
    urlBase.value = `${withoutHash}${hash}`
    clearQueryParams()
    return
  }

  urlBase.value = `${withoutHash.slice(0, qIndex)}${hash}`
  const search = withoutHash.slice(qIndex + 1)
  if (!search) {
    clearQueryParams()
    return
  }

  const rows: QueryParamRow[] = []
  for (const part of search.split('&')) {
    if (!part) continue
    const eq = part.indexOf('=')
    const key = eq >= 0 ? part.slice(0, eq) : part
    const value = eq >= 0 ? part.slice(eq + 1) : ''
    rows.push({
      ...emptyRow(),
      name: decodeQueryComponent(key),
      example: decodeQueryComponent(value),
      enabled: true,
    })
  }
  queryParams.value = rows
  normalizeQueryParams()
}

/** Persist URL as typed. Never force a leading "/". */
function urlForSave(): string {
  const raw = urlBase.value.trim()
  if (!raw) return ''
  // Keep path-only for http(s) absolute URLs; leave ws/tcp/udp/etc. untouched.
  if (/^https?:\/\//i.test(raw)) {
    return raw.replace(/^https?:\/\/[^/]*/i, '') || ''
  }
  return raw
}

function resolveRequestUrl(rawInput?: string): string | null {
  const raw = (rawInput ?? displayUrl.value).trim()
  if (!raw) return null
  // Absolute URLs keep their own scheme (http(s) / ws(s) / …).
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(raw)) {
    try {
      return new URL(raw).href
    } catch {
      return null
    }
  }
  if (raw.startsWith('/')) {
    return null
  }
  const hostPath = raw.replace(/^\/*/, '')
  if (!hostPath) return null
  const schemeMap: Record<string, string> = {
    HTTP: 'http',
    HTTPS: 'https',
    WS: 'ws',
    TCP: 'tcp',
    UDP: 'udp',
    GRPC: 'grpc',
  }
  const scheme = schemeMap[protocol.value] ?? 'http'
  try {
    return new URL(`${scheme}://${hostPath}`).href
  } catch {
    return null
  }
}

function compactParams(rows: ParamRow[]) {
  return rows
    .filter((r) => r.name.trim())
    .map((r) => ({
      name: r.name.trim(),
      type: r.type,
      required: r.required,
      description: r.description,
      example: r.example,
    }))
}

function buildDetail() {
  return {
    protocol: protocol.value,
    request_headers: compactParams(requestHeaders.value),
    query_params: compactParams(queryParams.value),
    request_body: {
      format: bodyFormat.value,
      raw: bodyRaw.value,
      data_type: 'Object',
      fields: compactParams(formDataFields.value),
    },
    authorization: {
      type: authType.value,
      bearer_token: authBearerToken.value,
      basic_username: authBasicUser.value,
      basic_password: authBasicPassword.value,
    },
    response_headers: compactParams(responseHeaders.value),
    response_results: [
      {
        name: resultName.value || t('create.defaultSuccess'),
        status_code: Number(resultStatus.value) || 200,
        format: 'json',
        data_type: 'Object',
        schema_raw: resultSchema.value,
        fields: [],
      },
    ],
    response_examples: [
      {
        name: exampleName.value || t('create.successExample'),
        status_code: Number(exampleStatus.value) || 200,
        content_type: 'application/json',
        body: exampleBody.value,
      },
    ],
  }
}

function buildSendHeaders() {
  const headers = requestHeaders.value
    .filter((h) => h.enabled !== false && h.name.trim())
    .map((h) => ({
      key: h.name.trim(),
      value: h.example,
      enabled: true,
    }))

  if (authType.value === 'bearer' && authBearerToken.value.trim()) {
    headers.push({
      key: 'Authorization',
      value: `Bearer ${authBearerToken.value.trim()}`,
      enabled: true,
    })
  } else if (authType.value === 'basic' && (authBasicUser.value || authBasicPassword.value)) {
    const token = btoa(`${authBasicUser.value}:${authBasicPassword.value}`)
    headers.push({
      key: 'Authorization',
      value: `Basic ${token}`,
      enabled: true,
    })
  }

  return headers
}

function applyInitial(resetUi = false) {
  const init = props.initial
  name.value = init?.name ?? ''
  const initUrl = init?.url ?? ''
  parseUrlInput(initUrl)
  resetUrlUndo()

  const detail = init?.detail
  const d = detail && typeof detail === 'object' ? (detail as Record<string, unknown>) : null
  if (d && isKnownProtocol(d.protocol)) {
    protocol.value = normalizeProtocol(d.protocol)
  }

  const allowed = methodsForProtocol(protocol.value)
  const rawMethod = (init?.method ?? allowed[0] ?? 'POST').toUpperCase()
  method.value = allowed.includes(rawMethod) ? rawMethod : (allowed[0] ?? 'POST')

  const fromInitFolder =
    init?.folder_id != null && Number.isFinite(Number(init.folder_id))
      ? Number(init.folder_id)
      : null
  const fromDetailFolder = d ? Number(d.folder_id) : NaN
  folderId.value =
    fromInitFolder ??
    (Number.isFinite(fromDetailFolder) ? fromDetailFolder : null) ??
    (props.mode === 'create' ? 0 : null)
  response.value = null
  if (resetUi) {
    responseTab.value = 'body'
    requestTab.value = 'body'
  }

  if (d) {
    if (Array.isArray(d.request_headers) && d.request_headers.length) {
      requestHeaders.value = d.request_headers.map((x: any) => ({
        name: String(x.name ?? ''),
        type: String(x.type ?? 'string'),
        required: !!x.required,
        description: String(x.description ?? ''),
        example: String(x.example ?? ''),
        enabled: x.enabled !== false,
      }))
    } else {
      requestHeaders.value = [defaultHeaderRow()]
    }
    normalizeRequestHeaders()
    if (Array.isArray(d.query_params) && d.query_params.length) {
      queryParams.value = d.query_params.map((x: any) => ({
        name: String(x.name ?? ''),
        type: String(x.type ?? 'string'),
        required: !!x.required,
        description: String(x.description ?? ''),
        example: String(x.example ?? ''),
        enabled: x.enabled !== false,
      }))
    } else if (!queryParams.value.length) {
      queryParams.value = [emptyParamRow()]
    }
    normalizeQueryParams()
    if (Array.isArray(d.response_headers) && d.response_headers.length) {
      responseHeaders.value = d.response_headers.map((x: any) => ({
        name: String(x.name ?? ''),
        type: String(x.type ?? 'string'),
        required: !!x.required,
        description: String(x.description ?? ''),
        example: String(x.example ?? ''),
      }))
    }
    const body = d.request_body as Record<string, unknown> | undefined
    if (body) {
      bodyRaw.value = String(body.raw ?? '')
      if (Array.isArray(body.fields) && body.fields.length) {
        formDataFields.value = (body.fields as any[]).map((x) => ({
          name: String(x.name ?? ''),
          type: x.type === 'file' ? 'file' : 'string',
          required: !!x.required,
          description: String(x.description ?? ''),
          example: String(x.example ?? ''),
          enabled: x.enabled !== false,
        }))
      } else {
        formDataFields.value = [emptyParamRow()]
      }
      normalizeFormDataFields()
      bodyFormat.value = resolveBodyFormat(bodyRaw.value, formDataFields.value)
    } else {
      bodyRaw.value = ''
      formDataFields.value = [emptyParamRow()]
      normalizeFormDataFields()
      bodyFormat.value = 'none'
    }
    const auth = d.authorization as Record<string, unknown> | undefined
    if (auth) {
      const type = String(auth.type ?? 'none')
      authType.value = type === 'bearer' || type === 'basic' ? type : 'none'
      authBearerToken.value = String(auth.bearer_token ?? '')
      authBasicUser.value = String(auth.basic_username ?? '')
      authBasicPassword.value = String(auth.basic_password ?? '')
    } else {
      authType.value = 'none'
      authBearerToken.value = ''
      authBasicUser.value = ''
      authBasicPassword.value = ''
    }
    const results = d.response_results as any[] | undefined
    if (results?.[0]) {
      resultName.value = String(results[0].name ?? '')
      resultStatus.value = String(results[0].status_code ?? '200')
      resultSchema.value = String(results[0].schema_raw ?? '')
    }
    const examples = d.response_examples as any[] | undefined
    if (examples?.[0]) {
      exampleName.value = String(examples[0].name ?? '')
      exampleStatus.value = String(examples[0].status_code ?? '200')
      exampleBody.value = String(examples[0].body ?? '')
    }
  } else {
    requestHeaders.value = [defaultHeaderRow()]
    normalizeRequestHeaders()
    queryParams.value = [emptyParamRow()]
    normalizeQueryParams()
    bodyFormat.value = 'none'
    bodyRaw.value = ''
    formDataFields.value = [emptyParamRow()]
    normalizeFormDataFields()
    authType.value = 'none'
    authBearerToken.value = ''
    authBasicUser.value = ''
    authBasicPassword.value = ''
    resultName.value = t('create.defaultSuccess')
    exampleName.value = t('create.successExample')
  }
}

async function loadFolders() {
  try {
    folders.value = await invoke('list_local_folders', {
      workspaceId: props.workspaceId,
      projectId: props.projectId,
    })
    if (folderId.value == null && props.mode === 'create') {
      folderId.value = 0
    }
  } catch {
    folders.value = []
  }
}

async function onSend() {
  await refreshEnvVars()
  const substituted = applyEnvVars(displayUrl.value.trim(), envVarMap.value)
  const url = resolveRequestUrl(substituted)
  if (!url) {
    showToast(t('request.invalidUrl'), 1500, 'warning')
    return
  }
  sending.value = true
  response.value = null
  responseTab.value = 'body'
  try {
    const result = await invoke<SendResponse>('send_http_request', {
      args: {
        method: method.value,
        url,
        headers: buildSendHeaders(),
        body: buildRequestBodyForSend(),
        interface_id: props.interfaceId ?? null,
        variables: Object.keys(envVarMap.value).length ? envVarMap.value : null,
      },
    })
    response.value = result
    responseBodyType.value = detectResponseBodyType(result.body ?? '')
    responseTab.value = 'body'
  } catch (e) {
    alert(String(e))
  } finally {
    sending.value = false
  }
}

function exportTabState() {
  return {
    name: name.value,
    method: method.value,
    url: displayUrl.value,
    folderId: folderId.value,
    detail: {
      ...buildDetail(),
      folder_id: folderId.value,
      protocol: protocol.value,
    },
  }
}

function snapshotFromState(state: ReturnType<typeof exportTabState>) {
  return JSON.stringify({
    name: state.name,
    method: state.method,
    url: state.url,
    detail: state.detail ?? null,
  })
}

const baselineSnapshot = ref('')

function getSnapshot() {
  return snapshotFromState(exportTabState())
}

function syncBaseline() {
  baselineSnapshot.value = getSnapshot()
}

function setBaseline(snapshot: string) {
  baselineSnapshot.value = snapshot
}

function isDirty() {
  if (!baselineSnapshot.value) return false
  return getSnapshot() !== baselineSnapshot.value
}

async function onSave(): Promise<boolean> {
  // Local folder ids are negative (next_local_id); 0 means project root.
  const folderIdNum =
    folderId.value == null || folderId.value === '' ? 0 : Number(folderId.value)
  if (!Number.isFinite(folderIdNum)) {
    showValidation(t('create.folderRequired'))
    return false
  }
  if (!name.value.trim()) {
    showValidation(t('create.nameRequired'))
    return false
  }
  if (saveUiTimer) {
    clearTimeout(saveUiTimer)
    saveUiTimer = null
  }
  saveUi.value = 'saving'
  try {
    // Keep exactly what was typed; never force a leading "/".
    // WS/UDP typically store host:port — prefixing "/" would break reopen/send.
    const saveUrl = urlForSave()

    let ok = false
    if (props.mode === 'create') {
      const row = await invoke<{ id: number }>('create_local_interface', {
        args: {
          workspace_id: props.workspaceId,
          project_id: props.projectId,
          folder_id: folderIdNum,
          name: name.value.trim(),
          method: method.value,
          url: saveUrl,
          status: 1,
          detail: buildDetail(),
        },
      })
      emit('saved', row.id)
      ok = true
    } else if (props.interfaceId != null) {
      await invoke('update_local_interface', {
        args: {
          id: props.interfaceId,
          workspace_id: props.workspaceId,
          project_id: props.projectId,
          folder_id: folderIdNum,
          name: name.value.trim(),
          method: method.value,
          url: saveUrl,
          status: 1,
          detail: buildDetail(),
        },
      })
      emit('saved', props.interfaceId)
      ok = true
    }
    if (ok) {
      saveUi.value = 'saved'
      saveUiTimer = setTimeout(() => {
        saveUi.value = 'idle'
        saveUiTimer = null
      }, 1400)
      return true
    }
    saveUi.value = 'idle'
    return false
  } catch (e) {
    saveUi.value = 'idle'
    alert(String(e))
    return false
  }
}

defineExpose({ save: onSave, exportTabState, getSnapshot, syncBaseline, setBaseline, isDirty })

const editorReady = ref(false)
const trackDirty = ref(false)
let editorAlive = false

const contentSnapshot = computed(() => getSnapshot())

onMounted(async () => {
  editorAlive = true
  baselineSnapshot.value = ''
  await Promise.all([loadFolders(), refreshEnvVars()])
  if (!editorAlive) return
  applyInitial(true)
  clampResponseHeight()
  await nextTick()
  if (!editorAlive) return
  emit('editor-ready')
  await nextTick()
  if (!editorAlive) return
  trackDirty.value = true
  editorReady.value = true
})

onUnmounted(() => {
  editorAlive = false
  trackDirty.value = false
  editorReady.value = false
  baselineSnapshot.value = ''
  hideUrlVarTip()
  urlUndo.dispose()
  if (saveUiTimer) {
    clearTimeout(saveUiTimer)
    saveUiTimer = null
  }
})

watch(
  () => [props.workspaceId, props.projectId, props.initial],
  async () => {
    editorReady.value = false
    trackDirty.value = false
    baselineSnapshot.value = ''
    await loadFolders()
    if (!editorAlive) return
    applyInitial(false)
    await nextTick()
    if (!editorAlive) return
    emit('editor-ready')
    await nextTick()
    if (!editorAlive) return
    trackDirty.value = true
    editorReady.value = true
  },
)

watch(
  contentSnapshot,
  () => {
    if (editorReady.value && trackDirty.value) emit('state-change')
  },
  { flush: 'post' },
)
</script>

<style scoped>
.body-pane {
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow: hidden;
}

.body-format-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  position: relative;
  z-index: 2;
}

.body-format-action {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
  margin-right: 10px;
  border: 0;
  background: transparent;
  appearance: none;
  color: var(--color-text-secondary);
  padding: 3px 0;
  font-size: 13px;
  line-height: 1.2;
  cursor: pointer;
  transition: color 0.15s ease;
}

.body-format-action:hover {
  color: var(--color-interface-field-text);
}

.body-format-switch {
  display: inline-flex;
  align-self: flex-start;
  flex-shrink: 0;
  padding: 2px;
  gap: 2px;
  border: 1px solid var(--color-interface-bar-border);
  border-radius: 7px;
  background: color-mix(in srgb, var(--color-interface-bar-border) 28%, transparent);
}

.body-format-switch__btn {
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  padding: 4px 14px;
  border-radius: 5px;
  font-size: 13px;
  line-height: 1.2;
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease,
    box-shadow 0.15s ease;
}

.body-format-switch__btn:hover:not(.body-format-switch__btn--active) {
  color: var(--color-interface-field-text);
  background: color-mix(in srgb, var(--color-interface-accent) 8%, transparent);
}

.body-format-switch__btn--active {
  background: var(--color-workspace-content);
  color: var(--color-interface-field-text);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
}

.interface-response-type-switch {
  flex-shrink: 0;
}

.interface-response-type-switch .body-format-switch__btn {
  padding: 3px 10px;
  font-size: 12px;
}

.body-none-hint {
  margin: 14px 0 0 18px;
  font-size: 13px;
  line-height: 1.55;
}

.param-table {
  border: 1px solid var(--color-interface-bar-border);
  border-radius: 4px;
  overflow: hidden;
  min-height: 0;
  max-width: 100%;
  min-width: 0;
}

.param-table__row {
  display: grid;
  grid-template-columns: 40px 1fr 1fr 1.2fr 36px;
  align-items: stretch;
  min-height: 36px;
  border-bottom: 1px solid var(--color-interface-bar-border);
  background: var(--color-workspace-content);
}

.param-table--resizable-wrap {
  position: relative;
  max-width: 100%;
  min-width: 0;
}

.param-table--resizable-wrap .param-table__row {
  position: relative;
  z-index: 1;
}

.param-table__row:last-child {
  border-bottom: 0;
}

.param-table__row--off {
  opacity: 0.55;
}

.param-table__check {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0;
  border-right: 1px solid var(--color-interface-bar-border);
  background: transparent;
  cursor: pointer;
}

.param-table__check input {
  width: 14px;
  height: 14px;
  margin: 0;
  cursor: pointer;
  accent-color: var(--color-interface-accent);
}

.param-table__combo {
  min-width: 0;
  border-right: 1px solid var(--color-interface-bar-border);
}

.param-table__cell {
  min-width: 0;
  height: 100%;
  min-height: 36px;
  border: 0;
  border-right: 1px solid var(--color-interface-bar-border);
  border-radius: 0;
  background: transparent;
  color: var(--color-interface-field-text);
  padding: 0 10px;
  font: inherit;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
}

.param-table__value {
  min-width: 0;
  height: 100%;
  min-height: 36px;
  border-right: 1px solid var(--color-interface-bar-border);
  display: flex;
  align-items: stretch;
}

.param-table__cell--inner {
  flex: 1;
  min-width: 0;
  border-right: 0;
}

.param-table__key-cell {
  display: flex;
  align-items: stretch;
  min-width: 0;
  height: 100%;
  min-height: 36px;
  border-right: 1px solid var(--color-interface-bar-border);
  background: transparent;
  transition: background-color 0.12s ease;
}

.param-table__key-cell:focus-within {
  background: color-mix(in srgb, var(--color-interface-accent) 6%, transparent);
}

.param-table__key-cell .param-table__cell--key {
  flex: 1;
  min-width: 0;
  border-right: 0;
  padding-right: 6px;
}

.param-table__key-cell .param-table__cell--key:focus {
  background: transparent;
}

.param-table__key-type {
  flex-shrink: 0;
  display: flex;
  align-self: stretch;
  width: 70px;
  min-width: 70px;
  height: 100%;
}

.param-table__key-type :deep(.menu-select.compact) {
  flex: 1;
  min-width: 70px;
}

.param-table__key-type :deep(.menu-select.compact .menu-select__trigger) {
  border: 0;
  border-left: 1px solid var(--color-interface-bar-border);
  border-radius: 0;
  background: transparent;
  padding: 0 6px 0 8px;
  gap: 3px;
  color: var(--color-text-secondary);
  transition:
    color 0.12s ease,
    background-color 0.12s ease;
}

.param-table__key-type :deep(.menu-select.compact .menu-select__trigger:hover),
.param-table__key-type :deep(.menu-select.open .menu-select__trigger) {
  color: var(--color-interface-field-text);
  background: color-mix(in srgb, var(--color-interface-accent) 8%, transparent);
}

.param-table__key-type :deep(.menu-select__label) {
  flex: 1 1 auto;
  min-width: 0;
  overflow: visible;
  text-overflow: clip;
  font-size: 12px;
  font-weight: 500;
}

.param-table__key-type :deep(.menu-select__chevron) {
  flex-shrink: 0;
  margin-left: 0;
}

.param-table__key-type :deep(.menu-select__chevron svg) {
  width: 12px;
  height: 12px;
}

.param-table__file {
  flex: 1;
  display: flex;
  align-items: center;
  min-width: 0;
  padding: 0 10px;
  cursor: pointer;
  overflow: hidden;
}

.param-table__file-input {
  display: none;
}

.param-table__file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  color: var(--color-interface-placeholder);
}

.param-table__file-name.filled {
  color: var(--color-interface-field-text);
}

.param-table__cell::placeholder {
  color: var(--color-interface-placeholder);
  opacity: 1;
}

.param-table__cell:focus {
  background: color-mix(in srgb, var(--color-interface-accent) 6%, transparent);
}

.param-table__del {
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
}

.param-table__del:hover:not(:disabled) {
  color: var(--color-text);
  background: var(--color-hover);
}

.param-table__del:disabled {
  opacity: 0.25;
  cursor: not-allowed;
}

.auth-pane {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.auth-pane__layout {
  flex: 1;
  display: flex;
  align-items: stretch;
  min-height: 0;
}

.auth-pane__left {
  width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 20px;
}

.auth-pane__type {
  width: 200px;
  max-width: 100%;
  flex-shrink: 0;
}

.auth-pane__type :deep(.menu-select) {
  width: 100%;
}

.auth-pane__divider {
  width: 1px;
  flex-shrink: 0;
  align-self: stretch;
  margin: 10px 0;
  background: var(--color-interface-bar-border);
}

.auth-pane__right {
  flex: 1;
  min-width: 0;
  max-width: 520px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-left: 20px;
}

.auth-pane__label {
  font-size: 13px;
  font-weight: 600;
  line-height: 20px;
  color: var(--color-interface-field-text);
}

.auth-pane__label--spacer {
  visibility: hidden;
  user-select: none;
}

.auth-pane__hint--none {
  margin: 0;
  line-height: 34px;
}

.auth-pane__input {
  width: 100%;
  max-width: 520px;
}

.auth-pane__input::placeholder {
  color: var(--color-interface-placeholder);
  opacity: 1;
}

.auth-pane__hint {
  margin: 2px 0 0;
  font-size: 12px;
  line-height: 1.55;
}
</style>
