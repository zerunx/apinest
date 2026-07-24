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
          width="15"
          height="15"
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
          v-for="m in methods"
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

defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [string]
}>()

const methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']
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

function onDocClick(e: MouseEvent) {
  const target = e.target as Node
  if (rootRef.value?.contains(target) || menuRef.value?.contains(target)) return
  open.value = false
}

function onReposition() {
  if (open.value) placeMenu()
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  window.addEventListener('resize', onReposition)
  window.addEventListener('scroll', onReposition, true)
})
onUnmounted(() => {
  document.removeEventListener('click', onDocClick)
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
  justify-content: space-between;
  gap: 4px;
  border: 0;
  border-radius: 0;
  background: transparent;
  padding: 0 8px 0 10px;
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

.method {
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
.method.delete {
  color: var(--method-delete);
}
.method.head {
  color: var(--method-head);
}
.method.options {
  color: var(--method-options);
}

.chevron {
  display: inline-flex;
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

.method-item:hover,
.method-item.active {
  background: var(--color-method-item-hover);
}
</style>
