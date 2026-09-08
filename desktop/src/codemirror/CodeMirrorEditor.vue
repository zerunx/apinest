<template>
  <div ref="hostRef" class="cm-host" @keydown.capture="onFindKeys" />
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { findNext, findPrevious, openSearchPanel } from '@codemirror/search'
import { useTheme } from '../composables/useTheme'
import { createCodeMirror, type CodeMirrorHandle } from './create-editor'
import type { EditorLanguage } from './languages'

const props = withDefaults(
  defineProps<{
    modelValue: string
    language?: EditorLanguage
    readonly?: boolean
    lineNumbers?: boolean
    placeholder?: string
  }>(),
  {
    language: 'JSON',
    readonly: false,
    lineNumbers: true,
    placeholder: '',
  },
)

const emit = defineEmits<{
  'update:modelValue': [string]
}>()

const { theme } = useTheme()
const hostRef = ref<HTMLElement | null>(null)
let editor: CodeMirrorHandle | null = null

onMounted(() => {
  const parent = hostRef.value
  if (!parent) return

  editor = createCodeMirror({
    parent,
    doc: props.modelValue,
    language: props.language,
    readonly: props.readonly,
    lineNumbers: props.lineNumbers,
    placeholder: props.placeholder,
    dark: theme.value === 'dark',
    onChange(value) {
      emit('update:modelValue', value)
    },
  })
})

watch(
  () => props.modelValue,
  (value) => {
    editor?.setDoc(value)
  },
)

watch(
  () => props.language,
  (language) => {
    editor?.setLanguage(language)
  },
)

watch(
  () => props.readonly,
  (readonly) => {
    editor?.setReadonly(readonly)
  },
)

watch(
  () => props.lineNumbers,
  (show) => {
    editor?.setLineNumbers(show)
  },
)

watch(theme, (mode) => {
  editor?.setDark(mode === 'dark')
})

onBeforeUnmount(() => {
  editor?.destroy()
  editor = null
})

function formatContent() {
  return editor?.formatJson() ?? false
}

function onFindKeys(event: KeyboardEvent) {
  const view = editor?.view
  if (!view) return
  const key = event.key.toLowerCase()
  const mod = event.ctrlKey || event.metaKey
  if (mod && key === 'f') {
    event.preventDefault()
    event.stopPropagation()
    openSearchPanel(view)
    return
  }
  if (key === 'f3' || (mod && key === 'g')) {
    event.preventDefault()
    event.stopPropagation()
    if (event.shiftKey) findPrevious(view)
    else findNext(view)
  }
}

function focus() {
  editor?.view.focus()
}

defineExpose({ formatContent, focus })
</script>

<style>
.cm-host {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  cursor: text;
}

.cm-host .cm-editor {
  flex: 1;
  height: 100%;
  min-height: 0;
  position: relative;
  overflow: hidden;
  cursor: text;
}

.cm-host .cm-scroller,
.cm-host .cm-content {
  cursor: text;
}

.cm-host .cm-content,
.cm-host .cm-gutters {
  min-height: 100%;
}

.cm-host .cm-panels.cm-panels-top {
  position: absolute;
  top: 8px;
  right: 8px;
  left: auto;
  z-index: 50;
  width: max-content;
  max-width: calc(100% - 16px);
  background: transparent;
  border: none;
}

.cm-host .cm-panels {
  pointer-events: none;
}

.cm-host .cm-panel {
  background: transparent;
}

.cm-find-widget {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 2px;
  height: 32px;
  padding: 3px 4px 3px 8px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: var(--shadow-menu);
  color: var(--color-text);
}

.cm-find-widget__input {
  width: 148px;
  min-width: 0;
  height: 24px;
  margin: 0;
  padding: 0 6px;
  border: 0;
  outline: none;
  background: transparent;
  color: var(--color-text);
  font-size: 12px;
  line-height: 24px;
}

.cm-find-widget__input::placeholder {
  color: var(--color-interface-placeholder);
}

.cm-find-widget__count {
  min-width: 52px;
  padding: 0 6px 0 4px;
  font-size: 11px;
  line-height: 1;
  color: var(--color-text-secondary);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  text-align: right;
  flex-shrink: 0;
}

.cm-find-widget__count--empty {
  color: var(--color-danger);
}

.cm-find-widget__btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  margin: 0;
  padding: 0;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--color-text-secondary);
}

.cm-find-widget__btn:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.cm-find-widget__btn:focus-visible {
  outline: 1px solid var(--color-interface-accent);
  outline-offset: 0;
}

.cm-find-widget__btn--close {
  margin-left: 2px;
}
</style>
