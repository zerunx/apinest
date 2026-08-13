<template>
  <aside class="code-panel" :style="{ width: `${width}px` }">
    <div class="code-panel__header">
      <button
        type="button"
        class="code-panel__tab"
        :title="t('codeSnippet.title')"
        @click="$emit('close')"
      >
        <svg
          class="code-icon"
          viewBox="0 0 1024 1024"
          width="16"
          height="16"
          aria-hidden="true"
        >
          <path
            fill="currentColor"
            d="M345.7 904c-3.4 0-6.6-0.7-9.7-2.1-11.6-5.3-16.6-19.1-11.3-30.7l340.8-739.1c3.8-8.2 12-13.5 21-13.5 3.4 0 6.6 0.7 9.7 2.1 11.6 5.4 16.6 19.1 11.3 30.7L366.7 890.6c-3.7 8.2-12 13.4-21 13.4z m410.9-140.9c-13.2 0-23.9-10.7-23.9-23.9 0-6.3 2.6-12.5 7.1-16.9l210.9-211-211-211.1c-4.4-4.4-7-10.5-7-16.9 0-13.2 10.7-23.9 23.9-23.9 6.3 0 12.2 2.5 16.8 7l244.7 244.8-244.8 244.9c-4.5 4.6-10.4 7-16.7 7z m-493 0c-6.3 0-12.2-2.5-16.8-7L2.1 511.4l244.7-244.8c4.5-4.5 10.5-7 16.8-7 13.2 0 23.9 10.7 23.9 23.9 0 6.3-2.5 12.4-7 16.9l-211 211.1 211 211.1c4.4 4.3 7 10.5 7 16.8 0 13-10.7 23.7-23.9 23.7z"
          />
        </svg>
      </button>
    </div>

    <div class="code-panel__body">
      <div class="code-panel__code-wrap">
        <button
          type="button"
          class="code-panel__copy"
          :title="t('codeSnippet.copy')"
          :disabled="!curlText"
          @click="onCopy"
        >
          <svg
            viewBox="0 0 24 24"
            width="15"
            height="15"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <rect x="9" y="9" width="11" height="11" rx="2" />
            <path d="M5 15V5a2 2 0 0 1 2-2h10" />
          </svg>
        </button>

        <RequestBodyRawEditor
          v-if="curlText"
          class="code-panel__editor"
          :model-value="curlText"
          content-type="CURL"
          readonly
          hide-gutter
        />
        <p v-else class="code-panel__empty muted">{{ t('codeSnippet.empty') }}</p>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { useI18n } from '../i18n'
import { useToast } from '../composables/useToast'
import RequestBodyRawEditor from './RequestBodyRawEditor.vue'

const props = defineProps<{
  width: number
  curlText: string
}>()

defineEmits<{
  close: []
}>()

const { t } = useI18n()
const { showToast } = useToast()

async function writeClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    try {
      const ta = document.createElement('textarea')
      ta.value = text
      ta.style.position = 'fixed'
      ta.style.left = '-9999px'
      document.body.appendChild(ta)
      ta.select()
      const ok = document.execCommand('copy')
      document.body.removeChild(ta)
      return ok
    } catch {
      return false
    }
  }
}

async function onCopy() {
  if (!props.curlText) return
  const ok = await writeClipboard(props.curlText)
  showToast(
    ok ? t('codeSnippet.copied') : t('codeSnippet.copyFailed'),
    1500,
    ok ? 'success' : 'error',
  )
}
</script>

<style scoped>
.code-panel {
  flex-shrink: 0;
  min-width: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-content);
  overflow: hidden;
}

.code-panel__header {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 8px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg);
}

.code-panel__tab {
  width: 30px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 6px;
  background: var(--color-active);
  color: var(--color-primary);
  padding: 0;
  cursor: pointer;
}

.code-panel__tab:hover {
  filter: brightness(1.05);
}

.code-icon {
  display: block;
}

.code-panel__body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 10px 12px 12px;
}

.code-panel__code-wrap {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.code-panel__copy {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 5;
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 6px;
  background: color-mix(in srgb, var(--color-surface) 88%, transparent);
  color: var(--color-text-secondary);
  padding: 0;
  cursor: pointer;
}

.code-panel__copy:hover:not(:disabled) {
  background: var(--color-hover);
  color: var(--color-text);
}

.code-panel__copy:disabled {
  opacity: 0.4;
  cursor: default;
}

.code-panel__editor {
  flex: 1;
  min-height: 0;
  height: 100%;
}

.code-panel__editor :deep(.interface-body-raw) {
  height: 100%;
  min-height: 0;
}

.code-panel__editor :deep(.interface-body-raw__host) {
  height: 100%;
  min-height: 0;
  margin: 0;
  border-radius: 8px;
  background: var(--color-surface);
}

.code-panel__editor :deep(.cm-scroller) {
  padding-right: 28px;
}

.code-panel__empty {
  margin: 0;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface);
  font-size: 13px;
  text-align: center;
  padding: 24px 8px;
}
</style>
