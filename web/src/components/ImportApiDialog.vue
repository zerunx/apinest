<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useLocale } from '@/composables/useLocale'
import { IMPORT_SOURCES } from '@/utils/import-sources'

const { t } = useLocale()

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  submit: [payload: { source: string; document: unknown; filename: string }]
}>()

const step = ref<1 | 2>(1)
const sourceId = ref('apinest')
const file = ref<File | null>(null)
const parsed = ref<unknown>(null)
const dragging = ref(false)
const submitting = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)

const selectedSource = computed(() => IMPORT_SOURCES.find((item) => item.id === sourceId.value))
const canNext = computed(() => !!selectedSource.value?.enabled && !!parsed.value)

watch(
  () => props.open,
  (open) => {
    if (!open) return
    step.value = 1
    sourceId.value = 'apinest'
    file.value = null
    parsed.value = null
    submitting.value = false
  },
)

function close() {
  emit('update:open', false)
}

function pickSource(id: string, enabled: boolean) {
  if (!enabled) {
    ElMessage.info(t('workspace.importExport.comingSoon'))
    return
  }
  sourceId.value = id
}

function openFilePicker() {
  fileInputRef.value?.click()
}

async function readFile(next: File) {
  const text = await next.text()
  let data: unknown
  try {
    data = JSON.parse(text)
  } catch {
    ElMessage.error(t('workspace.importExport.invalidJson'))
    return
  }
  file.value = next
  parsed.value = data
}

async function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  const next = input.files?.[0]
  input.value = ''
  if (next) await readFile(next)
}

async function onDrop(event: DragEvent) {
  dragging.value = false
  const next = event.dataTransfer?.files?.[0]
  if (next) await readFile(next)
}

function goNext() {
  if (!canNext.value) {
    ElMessage.warning(t('workspace.importExport.needFile'))
    return
  }
  step.value = 2
}

async function startImport() {
  if (!parsed.value || !selectedSource.value?.enabled) return
  submitting.value = true
  try {
    emit('submit', {
      source: selectedSource.value.id,
      document: parsed.value,
      filename: file.value?.name ?? 'import.json',
    })
  } finally {
    submitting.value = false
  }
}

defineExpose({
  close,
  setSubmitting(value: boolean) {
    submitting.value = value
  },
})
</script>

<template>
  <el-dialog
    :model-value="open"
    :title="t('workspace.importExport.title')"
    width="760px"
    align-center
    class="import-api-dialog"
    destroy-on-close
    @update:model-value="emit('update:open', $event)"
  >
    <div class="import-steps">
      <span class="import-steps__item" :class="{ 'is-active': step === 1 }">
        <i>1</i>
        {{ t('workspace.importExport.stepSource') }}
      </span>
      <span class="import-steps__arrow">›</span>
      <span class="import-steps__item" :class="{ 'is-active': step === 2 }">
        <i>2</i>
        {{ t('workspace.importExport.stepRules') }}
      </span>
    </div>

    <template v-if="step === 1">
      <div class="import-grid">
        <button
          v-for="item in IMPORT_SOURCES"
          :key="item.id"
          type="button"
          class="import-card"
          :class="{
            'is-active': sourceId === item.id,
            'is-disabled': !item.enabled,
          }"
          @click="pickSource(item.id, item.enabled)"
        >
          <img v-if="item.icon" class="import-card__icon" :src="item.icon" :alt="item.name" />
          <span v-else class="import-card__letter">{{ item.letter }}</span>
          <span>{{ item.name }}</span>
        </button>
      </div>

      <p class="import-hint">{{ t('workspace.importExport.hint') }}</p>

      <div
        class="import-drop"
        :class="{ 'is-dragging': dragging }"
        @click="openFilePicker"
        @dragenter.prevent="dragging = true"
        @dragover.prevent="dragging = true"
        @dragleave.prevent="dragging = false"
        @drop.prevent="onDrop"
      >
        <input
          ref="fileInputRef"
          type="file"
          accept=".json,application/json"
          class="import-drop__input"
          @change="onFileChange"
        />
        <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="1.7">
          <path d="M12 16V4M12 4l-4 4M12 4l4 4" />
          <path d="M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" />
        </svg>
        <p v-if="file">{{ t('workspace.importExport.selectedFile', { name: file.name }) }}</p>
        <p v-else>{{ t('workspace.importExport.drop') }}</p>
      </div>
    </template>

    <div v-else class="import-rules">
      <p>{{ t('workspace.importExport.ruleAppend') }}</p>
      <p v-if="file" class="import-rules__file">{{ file.name }}</p>
    </div>

    <template #footer>
      <el-button @click="close">{{ t('common.cancel') }}</el-button>
      <el-button v-if="step === 2" @click="step = 1">{{ t('workspace.importExport.prev') }}</el-button>
      <el-button v-if="step === 1" type="primary" :disabled="!canNext" @click="goNext">
        {{ t('workspace.importExport.next') }}
      </el-button>
      <el-button v-else type="primary" :loading="submitting" @click="startImport">
        {{ t('workspace.importExport.start') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.import-steps {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 20px;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.import-steps__item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.import-steps__item i {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-style: normal;
  font-size: 12px;
  background: var(--color-workspace-inset);
}

.import-steps__item.is-active {
  color: var(--color-primary);
}

.import-steps__item.is-active i {
  background: var(--color-primary);
  color: #fff;
}

.import-steps__arrow {
  opacity: 0.45;
}

.import-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}

.import-card {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 44px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-workspace-elevated);
  color: var(--color-text);
  cursor: pointer;
  text-align: left;
}

.import-card.is-active {
  border-color: var(--color-primary);
  box-shadow: inset 0 0 0 1px var(--color-primary);
}

.import-card.is-disabled {
  opacity: 0.55;
}

.import-card__icon {
  width: 22px;
  height: 22px;
  object-fit: contain;
  flex-shrink: 0;
}

.import-card__letter {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--color-workspace-inset);
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.import-hint {
  margin: 14px 0;
  padding: 10px 12px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--color-primary) 10%, transparent);
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

.import-drop {
  min-height: 160px;
  border: 1px dashed var(--color-border);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.import-drop.is-dragging,
.import-drop:hover {
  border-color: var(--color-primary);
  color: var(--color-text);
}

.import-drop__input {
  display: none;
}

.import-rules {
  min-height: 160px;
  padding: 8px 4px;
  color: var(--color-text);
  line-height: 1.7;
}

.import-rules__file {
  color: var(--color-text-secondary);
}
</style>
