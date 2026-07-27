<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { CopyDocument, MagicStick } from '@element-plus/icons-vue'
import { useLocale } from '@/composables/useLocale'
import { highlightRawContent } from '@/utils/raw-editor-highlight'
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
const editorRef = ref<HTMLTextAreaElement | null>(null)
const gutterRef = ref<HTMLElement | null>(null)
const highlightRef = ref<HTMLElement | null>(null)

const contentTypeOptions = ['JSON', 'Text', 'XML', 'HTML']

const rawContent = computed({
  get: () => props.modelValue.raw ?? '',
  set: (raw: string) => emit('update:modelValue', { ...props.modelValue, raw }),
})

const contentType = computed({
  get: () => props.modelValue.data_type || 'JSON',
  set: (data_type: string) => emit('update:modelValue', { ...props.modelValue, data_type }),
})

const lineNumbers = computed(() => {
  const count = Math.max(1, rawContent.value.split('\n').length)
  return Array.from({ length: count }, (_, index) => index + 1)
})

const highlightedContent = computed(() =>
  highlightRawContent(rawContent.value, contentType.value, props.modelValue.format),
)

const editorLanguageClass = computed(() => {
  const type = contentType.value.toLowerCase()
  return type === 'text' ? 'interface-body-raw__code-wrap--text' : `interface-body-raw__code-wrap--${type}`
})

function syncScroll(event: Event) {
  const target = event.target as HTMLTextAreaElement
  const scrollTop = target.scrollTop
  if (gutterRef.value) gutterRef.value.scrollTop = scrollTop
  if (highlightRef.value) highlightRef.value.scrollTop = scrollTop
}

function updateContentType(value: string) {
  contentType.value = value
}

function formatContent() {
  if (contentType.value !== 'JSON') return
  const text = rawContent.value.trim()
  if (!text) return
  try {
    rawContent.value = `${JSON.stringify(JSON.parse(text), null, 2)}\n`
  } catch {
    ElMessage.warning(t('workspace.interfaceForm.rawFormatInvalid'))
  }
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
</script>

<template>
  <div class="interface-body-raw">
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
      <div ref="gutterRef" class="interface-body-raw__gutter" aria-hidden="true">
        <div v-for="line in lineNumbers" :key="line" class="interface-body-raw__line-no">{{ line }}</div>
      </div>
      <div class="interface-body-raw__code-wrap" :class="editorLanguageClass">
        <pre ref="highlightRef" class="interface-body-raw__highlight" aria-hidden="true"><code v-html="highlightedContent" /></pre>
        <textarea
          ref="editorRef"
          v-model="rawContent"
          class="interface-body-raw__textarea"
          :readonly="readonly"
          spellcheck="false"
          @scroll="syncScroll"
        />
      </div>
    </div>
  </div>
</template>
