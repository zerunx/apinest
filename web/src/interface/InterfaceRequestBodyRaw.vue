<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { CopyDocument, MagicStick } from '@element-plus/icons-vue'
import { useLocale } from '@/composables/useLocale'
import { CodeMirrorEditor, toEditorLanguage, type EditorLanguage } from '@/codemirror'
import type { InterfaceRequestBody } from '@/types/workspace'

const props = defineProps<{
  modelValue: InterfaceRequestBody
  readonly?: boolean
  hideContentType?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: InterfaceRequestBody]
}>()

const { t } = useLocale()
const editorRef = ref<{ formatContent: () => boolean } | null>(null)

const contentTypeOptions: EditorLanguage[] = ['JSON', 'Text', 'XML', 'HTML']

const rawContent = computed({
  get: () => props.modelValue.raw ?? '',
  set: (raw: string) => emit('update:modelValue', { ...props.modelValue, raw }),
})

const contentType = computed({
  get: () => toEditorLanguage(props.modelValue.data_type),
  set: (data_type: EditorLanguage) => emit('update:modelValue', { ...props.modelValue, data_type }),
})

function updateContentType(value: string) {
  contentType.value = toEditorLanguage(value)
}

function formatContent() {
  if (contentType.value !== 'JSON' || props.readonly) return
  editorRef.value?.formatContent()
}

async function copyContent() {
  if (!rawContent.value) return
  try {
    await navigator.clipboard.writeText(rawContent.value)
    ElMessage.success(t('workspace.interfaceForm.rawCopySuccess'))
  } catch {
    ElMessage.error(t('workspace.interfaceForm.rawCopyFailed'))
  }
}

watch(
  () => props.modelValue.format,
  (format) => {
    if (format !== 'raw') return
    if (props.modelValue.data_type === 'Object' || props.modelValue.data_type === 'Array') {
      emit('update:modelValue', { ...props.modelValue, data_type: 'JSON' })
    }
  },
  { immediate: true },
)

defineExpose({ formatContent })
</script>

<template>
  <div
    class="interface-body-raw"
    :class="{ 'interface-body-raw--readonly': readonly }"
  >
    <div class="interface-body-raw__toolbar">
      <label v-if="!hideContentType" class="interface-body-raw__toolbar-label">
        {{ t('workspace.interfaceForm.rawContentType') }}
        <el-select
          :model-value="contentType"
          class="interface-body-raw__type-select"
          :disabled="readonly"
          popper-class="app-action-dropdown"
          @update:model-value="updateContentType"
        >
          <el-option v-for="item in contentTypeOptions" :key="item" :label="item" :value="item" />
        </el-select>
      </label>
      <button
        v-if="contentType === 'JSON' && !readonly"
        type="button"
        class="interface-body-raw__tool-btn"
        @click="formatContent"
      >
        <el-icon :size="14"><MagicStick /></el-icon>
        {{ t('workspace.interfaceForm.rawFormat') }}
      </button>
      <button type="button" class="interface-body-raw__tool-btn" @click="copyContent">
        <el-icon :size="14"><CopyDocument /></el-icon>
        {{ t('workspace.interfaceForm.rawCopy') }}
      </button>
    </div>

    <div class="interface-body-raw__editor">
      <CodeMirrorEditor
        ref="editorRef"
        v-model="rawContent"
        class="interface-body-raw__host"
        :language="contentType"
        :readonly="readonly"
      />
    </div>
  </div>
</template>
