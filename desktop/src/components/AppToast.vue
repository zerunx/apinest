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
        <span class="app-toast__mark" aria-hidden="true">
          <svg v-if="toast.type === 'success'" viewBox="0 0 16 16" width="14" height="14">
            <path
              fill="currentColor"
              d="M6.5 11.2 3.3 8l1.1-1.1 2.1 2.1 4.6-4.6L12.2 5.5 6.5 11.2z"
            />
          </svg>
          <svg v-else-if="toast.type === 'error'" viewBox="0 0 16 16" width="14" height="14">
            <path
              fill="currentColor"
              d="M8 1.5A6.5 6.5 0 1 0 14.5 8 6.51 6.51 0 0 0 8 1.5zm-.75 3h1.5v5h-1.5zm0 6h1.5v1.5h-1.5z"
            />
          </svg>
          <svg v-else-if="toast.type === 'warning'" viewBox="0 0 16 16" width="14" height="14">
            <path
              fill="currentColor"
              d="M8.85 2.2a1 1 0 0 0-1.7 0L1.2 12.1A1 1 0 0 0 2.05 13.5h11.9a1 1 0 0 0 .85-1.4L8.85 2.2zM7.25 6h1.5v3.5h-1.5zm0 4.5h1.5V12h-1.5z"
            />
          </svg>
          <svg v-else viewBox="0 0 16 16" width="14" height="14">
            <path
              fill="currentColor"
              d="M8 1.5A6.5 6.5 0 1 0 14.5 8 6.51 6.51 0 0 0 8 1.5zM7.25 4h1.5v5h-1.5zm0 6h1.5v1.5h-1.5z"
            />
          </svg>
        </span>
        <span class="app-toast__text">{{ toast.message }}</span>
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
  display: inline-flex;
  align-items: flex-start;
  gap: 8px;
  max-width: min(440px, calc(100vw - 32px));
  padding: 11px 16px 11px 14px;
  border-radius: 10px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  color: var(--color-text);
  box-shadow: var(--shadow-dialog, 0 10px 32px rgba(0, 0, 0, 0.18));
  font-size: 13px;
  line-height: 1.45;
  pointer-events: none;
  transform: translateX(-50%);
}

.app-toast__mark {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  margin-top: 1px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  color: var(--color-text-secondary);
  background: color-mix(in srgb, var(--color-text-secondary) 12%, transparent);
}

.app-toast__text {
  min-width: 0;
  text-align: left;
  word-break: break-word;
}

.app-toast--warning {
  border-color: color-mix(in srgb, #c27a00 40%, var(--color-border));
  background: color-mix(in srgb, #c27a00 10%, var(--color-surface));
}

.app-toast--warning .app-toast__mark {
  color: #c27a00;
  background: color-mix(in srgb, #c27a00 16%, transparent);
}

.app-toast--error {
  border-color: color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
  background: color-mix(in srgb, var(--color-danger) 10%, var(--color-surface));
  color: var(--color-danger);
}

.app-toast--error .app-toast__mark {
  color: var(--color-danger);
  background: color-mix(in srgb, var(--color-danger) 14%, transparent);
}

.app-toast--success {
  border-color: color-mix(in srgb, var(--method-get, #2ea043) 40%, var(--color-border));
  background: color-mix(in srgb, var(--method-get, #2ea043) 10%, var(--color-surface));
}

.app-toast--success .app-toast__mark {
  color: var(--method-get, #2ea043);
  background: color-mix(in srgb, var(--method-get, #2ea043) 14%, transparent);
}

.app-toast-enter-active,
.app-toast-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.app-toast-enter-from,
.app-toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-8px);
}
</style>
