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
}>()

const editorRef = ref<{ formatContent: () => boolean } | null>(null)

function formatContent() {
  return editorRef.value?.formatContent() ?? false
}

defineExpose({ formatContent })
</script>
