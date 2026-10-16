<template>
  <div class="method-picker" ref="rootRef">
    <button
      type="button"
      class="method-trigger"
      :class="{ open }"
      @click.stop="toggle"
    >
      <span class="method" :class="modelValue.toLowerCase()">{{ modelValue }}</span>
      <span class="chevron" :class="{ open }" aria-hidden="true">
        <svg
          viewBox="0 0 24 24"
          width="12"
          height="12"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M6 9l6 6 6-6" />
        </svg>
      </span>
    </button>

    <Teleport to="body">
      <div
        v-if="open"
        ref="menuRef"
        class="method-menu"
        :style="menuStyle"
        @click.stop
      >
        <button
          v-for="m in options"
          :key="m"
          type="button"
          class="method-item"
          :class="{ active: m === modelValue }"
          @click="pick(m)"
        >
          <span class="method" :class="m.toLowerCase()">{{ m }}</span>
        </button>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, reactive, ref } from 'vue'
import { bindOutsidePointerDismiss } from '../composables/useOverlayDismiss'
import { HTTP_METHODS } from '../utils/http-method'

withDefaults(
  defineProps<{
    modelValue: string
    options?: string[]
  }>(),
  {
    options: () => [...HTTP_METHODS],
  },
)

const emit = defineEmits<{
  'update:modelValue': [string]
}>()

const open = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const menuStyle = reactive({
  top: '0px',
  left: '0px',
})

function placeMenu() {
  const el = rootRef.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  menuStyle.top = `${rect.bottom + 4}px`
  menuStyle.left = `${rect.left}px`
}

async function toggle() {
  open.value = !open.value
  if (open.value) {
    placeMenu()
    await nextTick()
    placeMenu()
  }
}

function pick(m: string) {
  emit('update:modelValue', m)
  open.value = false
}

function onReposition() {
  if (open.value) placeMenu()
}

let unbindOutside: (() => void) | null = null

onMounted(() => {
  unbindOutside = bindOutsidePointerDismiss(
    () => open.value,
    () => [rootRef.value, menuRef.value],
    () => {
      open.value = false
    },
  )
  window.addEventListener('resize', onReposition)
  window.addEventListener('scroll', onReposition, true)
})
onUnmounted(() => {
  unbindOutside?.()
  window.removeEventListener('resize', onReposition)
  window.removeEventListener('scroll', onReposition, true)
})
</script>

<style scoped>
.method-picker {
  position: relative;
  height: 100%;
  width: 100%;
  display: flex;
  align-items: stretch;
}

.method-trigger {
  width: 100%;
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  padding: 0 8px 0 14px;
  box-shadow: none;
  outline: none;
  cursor: pointer;
}

.method-trigger:hover,
.method-trigger:focus,
.method-trigger:active,
.method-trigger.open {
  background: transparent;
}

.method-trigger > .method {
  flex: 1 1 auto;
  min-width: 0 !important;
  text-align: left;
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.02em;
  line-height: 1;
}

.method.get {
  color: var(--method-get);
}
.method.post {
  color: var(--method-post);
}
.method.put {
  color: var(--method-put);
}
.method.patch {
  color: var(--method-patch);
}
.method.delete,
.method.del {
  color: var(--method-delete);
}
.method.head {
  color: var(--method-head);
}
.method.options,
.method.opt {
  color: var(--method-options);
}
.method.hsf,
.method.dubbo,
.method.grpc,
.method.sofa,
.method.tr {
  color: var(--method-rpc);
}
.method.ws,
.method.wss {
  color: var(--method-ws);
}
.method.tcp,
.method.udp {
  color: var(--method-tcp);
}

.chevron {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 12px;
  color: var(--color-text-secondary);
  opacity: 0.7;
  transition: transform 0.15s ease;
  flex-shrink: 0;
}

.chevron.open {
  transform: rotate(180deg);
}

.method-menu {
  position: fixed;
  z-index: 4000;
  width: 108px;
  padding: 6px;
  border-radius: 8px;
  background: var(--color-method-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
}

.method-item {
  width: 100%;
  display: flex;
  align-items: center;
  border: 0;
  background: transparent;
  padding: 7px 10px;
  border-radius: 6px;
  text-align: left;
  cursor: pointer;
}

.method-item .method {
  flex: none;
  text-align: left;
}

.method-item:hover,
.method-item.active {
  background: var(--color-method-item-hover);
}
</style>
