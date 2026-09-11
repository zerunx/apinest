<template>
  <div
    class="interface-body-raw"
    :class="{
      'interface-body-raw--no-gutter': hideGutter,
      'interface-body-raw--readonly': readonly,
    }"
  >
    <CodeMirrorEditor
      ref="editorRef"
      class="interface-body-raw__host"
      :model-value="modelValue"
      :language="contentType"
      :readonly="readonly"
      :line-numbers="!hideGutter"
      :placeholder="placeholder"
      @update:model-value="$emit('update:modelValue', $event)"
      @scroll="$emit('scroll', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { CodeMirrorEditor, type EditorLanguage } from '../codemirror'

withDefaults(
  defineProps<{
    modelValue: string
    readonly?: boolean
    placeholder?: string
    contentType?: EditorLanguage
    hideGutter?: boolean
  }>(),
  {
    readonly: false,
    placeholder: '',
    contentType: 'JSON',
    hideGutter: false,
  },
)

defineEmits<{
  'update:modelValue': [string]
  scroll: [number]
}>()

const editorRef = ref<{
  formatContent: () => boolean
  focus: () => void
  getScrollTop: () => number
  setScrollTop: (top: number) => void
} | null>(null)

function formatContent() {
  return editorRef.value?.formatContent() ?? false
}

function focus() {
  editorRef.value?.focus()
}

function getScrollTop() {
  return editorRef.value?.getScrollTop() ?? 0
}

function setScrollTop(top: number) {
  editorRef.value?.setScrollTop(top)
}

defineExpose({ formatContent, focus, getScrollTop, setScrollTop })
</script>
