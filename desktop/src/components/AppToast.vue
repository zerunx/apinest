<template>
  <Teleport to="body">
    <Transition name="app-toast">
      <div
        v-if="toast.visible"
        class="app-toast"
        :class="`app-toast--${toast.type}`"
        role="status"
        aria-live="polite"
      >
        {{ toast.message }}
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { useToast } from '../composables/useToast'

const { toast } = useToast()
</script>

<style scoped>
.app-toast {
  position: fixed;
  top: 52px;
  left: 50%;
  z-index: 5000;
  transform: translateX(-50%);
  max-width: min(420px, calc(100vw - 32px));
  padding: 10px 16px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  color: var(--color-text);
  box-shadow: var(--shadow-dialog, 0 8px 28px rgba(0, 0, 0, 0.18));
  font-size: 13px;
  line-height: 1.4;
  text-align: center;
  pointer-events: none;
}

.app-toast--warning {
  border-color: color-mix(in srgb, var(--color-danger) 35%, var(--color-border));
  background: color-mix(in srgb, var(--color-danger) 8%, var(--color-surface));
}

.app-toast--error {
  border-color: color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
  background: color-mix(in srgb, var(--color-danger) 10%, var(--color-surface));
  color: var(--color-danger);
}

.app-toast--success {
  border-color: color-mix(in srgb, var(--method-get, #2ea043) 35%, var(--color-border));
  background: color-mix(in srgb, var(--method-get, #2ea043) 10%, var(--color-surface));
}

.app-toast-enter-active,
.app-toast-leave-active {
  transition:
    opacity 0.18s ease,
    transform 0.18s ease;
}

.app-toast-enter-from,
.app-toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-6px);
}
</style>
