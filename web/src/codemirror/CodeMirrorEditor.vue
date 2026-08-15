<template>
  <div ref="hostRef" class="cm-host" @mousedown="focusEditor" />
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
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

function focusEditor() {
  editor?.view.focus()
}

defineExpose({ formatContent })
</script>

<style>
.cm-host {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 100%;
  cursor: text;
}

.cm-host .cm-editor {
  flex: 1;
  height: 100%;
  min-height: 100%;
}

.cm-host .cm-scroller {
  min-height: 100%;
  height: 100%;
  align-items: stretch !important;
}

.cm-host .cm-content,
.cm-host .cm-gutters {
  min-height: 100%;
}
</style>
