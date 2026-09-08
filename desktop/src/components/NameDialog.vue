<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="overlay"
      @mousedown="onOverlayMouseDown"
      @mouseup="onOverlayMouseUp"
    >
      <div ref="dialogRef" class="dialog" :style="dialogStyle" role="dialog" aria-modal="true">
        <h3>{{ title }}</h3>
        <p v-if="hint" class="muted hint">{{ hint }}</p>
        <label class="field">
          <span v-if="label">{{ label }}</span>
          <input
            ref="inputRef"
            v-model="name"
            class="input"
            type="text"
            maxlength="100"
            @keydown.enter.prevent="onConfirm"
          />
        </label>
        <p v-if="error" class="error">{{ error }}</p>
        <div class="actions">
          <button class="btn" type="button" @click="onCancel">{{ t('common.cancel') }}</button>
          <button class="btn btn-primary" type="button" :disabled="saving" @click="onConfirm">
            {{ saving ? '…' : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, toRef, watch } from 'vue'
import { useI18n } from '../i18n'
import type { DialogAnchor } from '../composables/useDialogAnchor'
import { useAnchoredDialog } from '../composables/useAnchoredDialog'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'

const props = defineProps<{
  open: boolean
  title: string
  label?: string
  hint?: string
  initial?: string
  anchor?: DialogAnchor | null
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  confirm: [name: string]
}>()

const { t } = useI18n()
const name = ref('')
const error = ref('')
const saving = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)
const openRef = toRef(props, 'open')
const anchorRef = computed(() => props.anchor)
const { dialogRef, dialogStyle, placeDialog, onReposition } = useAnchoredDialog(openRef, anchorRef)

function onCancel() {
  emit('update:open', false)
}

const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(openRef, onCancel, {
  escape: true,
})

watch(
  () => props.open,
  async (v) => {
    if (!v) return
    name.value = props.initial ?? ''
    error.value = ''
    saving.value = false
    await placeDialog()
    await nextTick()
    inputRef.value?.focus()
    inputRef.value?.select()
  },
)

watch(
  () => props.anchor,
  () => {
    if (props.open) void placeDialog()
  },
)

function onConfirm() {
  const trimmed = name.value.trim()
  if (!trimmed) {
    error.value = t('create.nameRequired')
    return
  }
  emit('confirm', trimmed)
}

onMounted(() => {
  window.addEventListener('resize', onReposition)
  window.addEventListener('scroll', onReposition, true)
})

onUnmounted(() => {
  window.removeEventListener('resize', onReposition)
  window.removeEventListener('scroll', onReposition, true)
})
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 1050;
  background: rgba(0, 0, 0, 0.45);
}

.dialog {
  position: fixed;
  width: min(420px, calc(100vw - 24px));
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: var(--shadow-dialog);
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.hint {
  margin: 0;
  font-size: 12px;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.error {
  margin: 0;
  color: var(--color-danger);
  font-size: 13px;
}
</style>
