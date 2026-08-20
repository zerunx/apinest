<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../i18n'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'
import { useToast } from '../composables/useToast'
import { IMPORT_SOURCES } from '../utils/import-sources'

const { t } = useI18n()
const { showToast } = useToast()

const open = defineModel<boolean>('open', { default: false })

const emit = defineEmits<{
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

const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(open, () => {
  open.value = false
})

watch(open, (next) => {
  if (!next) return
  step.value = 1
  sourceId.value = 'apinest'
  file.value = null
  parsed.value = null
  submitting.value = false
})

function pickSource(id: string, enabled: boolean) {
  if (!enabled) {
    showToast(t('importExport.comingSoon'), 1500, 'info')
    return
  }
  sourceId.value = id
}

function openFilePicker() {
  fileInputRef.value?.click()
}

async function readFile(next: File) {
  const text = await next.text()
  try {
    parsed.value = JSON.parse(text)
  } catch {
    showToast(t('importExport.invalidJson'), 1800, 'error')
    return
  }
  file.value = next
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
    showToast(t('importExport.needFile'), 1500, 'warning')
    return
  }
  step.value = 2
}

function startImport() {
  if (!parsed.value || !selectedSource.value?.enabled || submitting.value) return
  emit('submit', {
    source: selectedSource.value.id,
    document: parsed.value,
    filename: file.value?.name ?? 'import.json',
  })
}

function setSubmitting(value: boolean) {
  submitting.value = value
}

defineExpose({ setSubmitting })
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="overlay"
      @mousedown="onOverlayMouseDown"
      @mouseup="onOverlayMouseUp"
    >
      <div class="dialog" role="dialog" aria-modal="true" @click.stop>
        <div class="dialog-head">
          <h3>{{ t('importExport.title') }}</h3>
          <button type="button" class="close-btn" :title="t('common.close')" @click="open = false">
            ×
          </button>
        </div>

        <div class="import-steps">
          <span class="step" :class="{ active: step === 1 }">
            <i>1</i>
            {{ t('importExport.stepSource') }}
          </span>
          <span class="step-arrow">›</span>
          <span class="step" :class="{ active: step === 2 }">
            <i>2</i>
            {{ t('importExport.stepRules') }}
          </span>
        </div>

        <template v-if="step === 1">
          <div class="import-grid">
            <button
              v-for="item in IMPORT_SOURCES"
              :key="item.id"
              type="button"
              class="import-card"
              :class="{ active: sourceId === item.id, disabled: !item.enabled }"
              @click="pickSource(item.id, item.enabled)"
            >
              <img v-if="item.icon" class="icon" :src="item.icon" :alt="item.name" />
              <span v-else class="letter">{{ item.letter }}</span>
              <span>{{ item.name }}</span>
            </button>
          </div>

          <p class="hint">{{ t('importExport.hint') }}</p>

          <div
            class="drop"
            :class="{ dragging }"
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
              class="file-input"
              @change="onFileChange"
            />
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="1.7">
              <path d="M12 16V4M12 4l-4 4M12 4l4 4" />
              <path d="M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" />
            </svg>
            <p v-if="file">{{ t('importExport.selectedFile', { name: file.name }) }}</p>
            <p v-else>{{ t('importExport.drop') }}</p>
          </div>
        </template>

        <div v-else class="rules">
          <p>{{ t('importExport.ruleAppend') }}</p>
          <p v-if="file" class="muted">{{ file.name }}</p>
        </div>

        <div class="actions">
          <button type="button" class="btn" @click="open = false">{{ t('common.cancel') }}</button>
          <button v-if="step === 2" type="button" class="btn" @click="step = 1">
            {{ t('importExport.prev') }}
          </button>
          <button v-if="step === 1" type="button" class="btn btn-primary" :disabled="!canNext" @click="goNext">
            {{ t('importExport.next') }}
          </button>
          <button v-else type="button" class="btn btn-primary" :disabled="submitting" @click="startImport">
            {{ t('importExport.start') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 1200;
  background: rgba(0, 0, 0, 0.5);
  display: grid;
  place-items: center;
  padding: 24px;
}

.dialog {
  width: min(760px, 96vw);
  max-height: min(720px, 90vh);
  overflow: auto;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: var(--shadow-dialog);
  padding: 18px 20px 16px;
}

.dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.dialog-head h3 {
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
  font-size: 22px;
  line-height: 1;
}

.close-btn:hover {
  background: var(--color-hover);
}

.import-steps {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 18px;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.step {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.step i {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-style: normal;
  font-size: 12px;
  background: var(--color-hover);
}

.step.active {
  color: var(--color-primary);
}

.step.active i {
  background: var(--color-primary);
  color: #fff;
}

.step-arrow {
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
  background: var(--color-surface);
  text-align: left;
}

.import-card.active {
  border-color: var(--color-primary);
  box-shadow: inset 0 0 0 1px var(--color-primary);
}

.import-card.disabled {
  opacity: 0.55;
}

.icon {
  width: 22px;
  height: 22px;
  object-fit: contain;
  flex-shrink: 0;
}

.letter {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--color-hover);
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.hint {
  margin: 14px 0;
  padding: 10px 12px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

.drop {
  min-height: 150px;
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

.drop.dragging,
.drop:hover {
  border-color: var(--color-primary);
  color: var(--color-text);
}

.file-input {
  display: none;
}

.rules {
  min-height: 150px;
  padding: 8px 2px;
  line-height: 1.7;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}
</style>
