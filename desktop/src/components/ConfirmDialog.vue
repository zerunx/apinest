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
        <p v-if="message" class="message">{{ message }}</p>
        <div class="actions">
          <button v-if="showCancel" class="btn" type="button" @click="onCancel">
            {{ cancelLabel || t('common.cancel') }}
          </button>
          <button
            class="btn"
            :class="variant === 'primary' ? 'btn-primary' : 'btn-danger-solid'"
            type="button"
            @click="onConfirm"
          >
            {{ confirmLabel || t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, toRef, watch } from 'vue'
import { useI18n } from '../i18n'
import type { DialogAnchor } from '../composables/useDialogAnchor'
import { useAnchoredDialog } from '../composables/useAnchoredDialog'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'

const props = withDefaults(
  defineProps<{
    open: boolean
    title: string
    message: string
    anchor?: DialogAnchor | null
    variant?: 'danger' | 'primary'
    showCancel?: boolean
    confirmLabel?: string
    cancelLabel?: string
    closeOnConfirm?: boolean
  }>(),
  {
    variant: 'danger',
    showCancel: true,
    closeOnConfirm: true,
  },
)

const emit = defineEmits<{
  'update:open': [boolean]
  confirm: []
}>()

const { t } = useI18n()
const openRef = toRef(props, 'open')
const anchorRef = computed(() => props.anchor)
const { dialogRef, dialogStyle, placeDialog, onReposition } = useAnchoredDialog(openRef, anchorRef)

function onCancel() {
  emit('update:open', false)
}

const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(openRef, onCancel)

watch(
  () => props.open,
  (visible) => {
    if (visible) void placeDialog()
  },
)

watch(
  () => props.anchor,
  () => {
    if (props.open) void placeDialog()
  },
)

function onConfirm() {
  emit('confirm')
  if (props.closeOnConfirm) {
    emit('update:open', false)
  }
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
  z-index: 1100;
  background: rgba(0, 0, 0, 0.45);
}

.dialog {
  position: fixed;
  width: min(380px, calc(100vw - 24px));
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: var(--shadow-dialog);
  padding: 22px 22px 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dialog h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 0.01em;
  color: var(--color-text);
}

.message {
  margin: 0;
  font-size: 13px;
  line-height: 1.55;
  color: var(--color-text-secondary);
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}

.btn-danger-solid {
  background: var(--color-danger);
  border-color: var(--color-danger);
  color: #fff;
}

.btn-danger-solid:hover {
  filter: brightness(1.08);
}
</style>
