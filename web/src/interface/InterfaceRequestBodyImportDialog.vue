<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { CopyDocument, InfoFilled, MagicStick } from '@element-plus/icons-vue'
import { useLocale } from '@/composables/useLocale'
import { highlightRawContent } from '@/utils/raw-editor-highlight'
import type { BodyImportMode } from '@/utils/json-body-import'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  apply: [payload: { mode: BodyImportMode; jsonText: string }]
}>()

const { t } = useLocale()
const jsonText = ref('')
const gutterRef = ref<HTMLElement | null>(null)
const highlightRef = ref<HTMLElement | null>(null)

const lineNumbers = computed(() => {
  const count = Math.max(1, jsonText.value.split('\n').length)
  return Array.from({ length: count }, (_, index) => index + 1)
})

const highlightedContent = computed(() => highlightRawContent(jsonText.value, 'JSON', 'raw'))

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      jsonText.value = ''
    }
  },
)

function closeDialog() {
  emit('update:visible', false)
}

function syncScroll(event: Event) {
  const target = event.target as HTMLTextAreaElement
  const scrollTop = target.scrollTop
  if (gutterRef.value) gutterRef.value.scrollTop = scrollTop
  if (highlightRef.value) highlightRef.value.scrollTop = scrollTop
}

function formatContent() {
  const text = jsonText.value.trim()
  if (!text) return
  try {
    jsonText.value = `${JSON.stringify(JSON.parse(text), null, 2)}\n`
  } catch {
    ElMessage.warning(t('workspace.interfaceForm.rawFormatInvalid'))
  }
}

async function copyContent() {
  if (!jsonText.value) return
  try {
    await navigator.clipboard.writeText(jsonText.value)
    ElMessage.success(t('workspace.interfaceForm.rawCopySuccess'))
  } catch {
    ElMessage.error(t('workspace.interfaceForm.rawCopyFailed'))
  }
}

function submit(mode: BodyImportMode) {
  if (!jsonText.value.trim()) {
    ElMessage.warning(t('workspace.interfaceForm.importJsonEmpty'))
    return
  }
  emit('apply', { mode, jsonText: jsonText.value })
  closeDialog()
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    class="interface-body-import-dialog"
    :title="t('workspace.interfaceForm.importJsonTitle')"
    width="720px"
    destroy-on-close
    @update:model-value="emit('update:visible', $event)"
  >
    <div class="interface-body-import__tip">
      <el-icon :size="14"><InfoFilled /></el-icon>
      <span>{{ t('workspace.interfaceForm.importJsonTip') }}</span>
    </div>

    <div class="interface-body-raw interface-body-raw--dialog">
      <div class="interface-body-raw__toolbar">
        <label class="interface-body-raw__toolbar-label">
          {{ t('workspace.interfaceForm.rawContentType') }}
          <span class="interface-body-import__type">JSON</span>
        </label>
        <button type="button" class="interface-body-raw__tool-btn" @click="formatContent">
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
        <div class="interface-body-raw__code-wrap interface-body-raw__code-wrap--json">
          <pre ref="highlightRef" class="interface-body-raw__highlight" aria-hidden="true"><code v-html="highlightedContent" /></pre>
          <textarea
            v-model="jsonText"
            class="interface-body-raw__textarea"
            spellcheck="false"
            @scroll="syncScroll"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <div class="interface-body-import__footer">
        <el-button @click="closeDialog">{{ t('common.cancel') }}</el-button>
        <el-button @click="submit('replace')">{{ t('workspace.interfaceForm.importReplace') }}</el-button>
        <el-button @click="submit('append')">{{ t('workspace.interfaceForm.importAppend') }}</el-button>
        <el-button type="primary" @click="submit('merge')">{{ t('workspace.interfaceForm.importMerge') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>
