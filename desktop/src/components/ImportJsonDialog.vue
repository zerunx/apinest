<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="overlay"
      @mousedown="onOverlayMouseDown"
      @mouseup="onOverlayMouseUp"
    >
      <div ref="dialogRef" class="dialog" :style="dialogStyle" role="dialog" aria-modal="true">
        <div class="dialog__header">
          <h3>{{ t('request.importJsonTitle') }}</h3>
          <button type="button" class="close-btn" :title="t('common.close')" @click="onCancel">
            <span>×</span>
          </button>
        </div>

        <div class="interface-body-raw interface-body-raw--dialog import-raw-box">
          <div class="interface-body-raw__toolbar">
            <label class="interface-body-raw__toolbar-label">
              {{ t('request.rawContentType') }}
              <span class="interface-body-import__type">JSON</span>
            </label>
            <button type="button" class="interface-body-raw__tool-btn" @click="formatContent">
              <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
                <path
                  fill="currentColor"
                  d="M7 5h10v2H7V5zm0 4h10v2H7V9zm0 4h7v2H7v-2zm-1 6l-4-4h3v-4h2v4h3l-4 4z"
                />
              </svg>
              {{ t('request.rawFormat') }}
            </button>
            <button type="button" class="interface-body-raw__tool-btn" @click="copyContent">
              <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
                <path
                  fill="currentColor"
                  d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"
                />
              </svg>
              {{ t('request.rawCopy') }}
            </button>
          </div>
          <div class="interface-body-raw__editor" @mousedown="onEditorSurfaceMouseDown">
            <CodeMirrorEditor
              ref="editorRef"
              v-model="jsonText"
              class="interface-body-raw__host"
              language="JSON"
            />
          </div>
        </div>

        <div class="actions">
          <button class="btn" type="button" @click="onCancel">{{ t('common.cancel') }}</button>
          <button class="btn" type="button" @click="submit('replace')">
            {{ t('request.importReplace') }}
          </button>
          <button class="btn" type="button" @click="submit('append')">
            {{ t('request.importAppend') }}
          </button>
          <button class="btn btn-primary" type="button" @click="submit('merge')">
            {{ t('request.importMerge') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, toRef, watch } from 'vue'
import { useI18n } from '../i18n'
import type { DialogAnchor } from '../composables/useDialogAnchor'
import { useAnchoredDialog } from '../composables/useAnchoredDialog'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'
import { useToast } from '../composables/useToast'
import { CodeMirrorEditor } from '../codemirror'
import type { FormDataImportMode } from '../utils/form-data-json-import'

const props = defineProps<{
  open: boolean
  anchor?: DialogAnchor | null
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  apply: [payload: { mode: FormDataImportMode; jsonText: string }]
}>()

const { t } = useI18n()
const { showToast } = useToast()
const jsonText = ref('')
const editorRef = ref<{ formatContent: () => boolean; focus: () => void } | null>(null)
const openRef = toRef(props, 'open')
const anchorRef = computed(() => props.anchor)
const { dialogRef, dialogStyle, placeDialog, onReposition } = useAnchoredDialog(openRef, anchorRef)

function onCancel() {
  emit('update:open', false)
}

const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(openRef, onCancel, {
  escape: true,
})

watch(
  () => props.open,
  async (visible) => {
    if (!visible) return
    jsonText.value = ''
    await placeDialog()
    await nextTick()
    await nextTick()
    requestAnimationFrame(() => {
      editorRef.value?.focus()
    })
  },
)

watch(
  () => props.anchor,
  () => {
    if (props.open) void placeDialog()
  },
)

function onEditorSurfaceMouseDown(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  if (!target) return
  // Clicks on empty editor chrome (not already starting a content selection) focus CM.
  if (target.closest('.cm-content') || target.closest('.cm-gutterElement')) return
  editorRef.value?.focus()
}

function formatContent() {
  const ok = editorRef.value?.formatContent() ?? false
  if (!ok && jsonText.value.trim()) {
    showToast(t('request.importJsonInvalid'), 1600, 'warning')
  }
}

async function copyContent() {
  if (!jsonText.value) return
  try {
    await navigator.clipboard.writeText(jsonText.value)
    showToast(t('request.rawCopySuccess'), 1400, 'success')
  } catch {
    showToast(t('request.rawCopyFailed'), 1600, 'error')
  }
}

function submit(mode: FormDataImportMode) {
  if (!jsonText.value.trim()) {
    showToast(t('request.importJsonEmpty'), 1600, 'warning')
    return
  }
  emit('apply', { mode, jsonText: jsonText.value })
}

onMounted(() => {
  window.addEventListener('resize', onReposition)
  window.addEventListener('scroll', onReposition, true)
})

onUnmounted(() => {
  window.removeEventListener('resize', onReposition)
  window.removeEventListener('scroll', onReposition, true)
})
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 1050;
  background: rgba(0, 0, 0, 0.45);
}

.dialog {
  position: fixed;
  width: min(720px, calc(100vw - 24px));
  max-height: min(80vh, 720px);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: var(--shadow-dialog);
  padding: 18px 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
}

.dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-shrink: 0;
}

.dialog h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.close-btn {
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 19px;
  line-height: 0;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  flex-shrink: 0;
}

.close-btn:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.close-btn span {
  display: block;
  transform: translateY(-1px);
}

.import-raw-box {
  flex: 1;
  min-height: 0;
  border: 1px solid var(--color-interface-bar-border, var(--color-border));
  border-radius: 8px;
  overflow: hidden;
  background: var(--color-workspace-content, var(--color-surface));
}

.import-raw-box .interface-body-raw__editor {
  --raw-editor-box-height: min(48vh, 420px);
  min-height: 280px;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  flex-shrink: 0;
  margin-top: 2px;
}
</style>
