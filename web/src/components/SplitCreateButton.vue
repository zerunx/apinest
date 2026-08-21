<script setup lang="ts">
import { nextTick, onBeforeUnmount, reactive, ref } from 'vue'

const emit = defineEmits<{
  create: [event: MouseEvent]
  import: []
  export: []
}>()

defineProps<{
  createTitle: string
  importLabel: string
  exportLabel: string
}>()

const open = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const menuStyle = reactive({ left: '0px', top: '0px' })
let closeTimer: ReturnType<typeof setTimeout> | null = null

function clearCloseTimer() {
  if (closeTimer) {
    clearTimeout(closeTimer)
    closeTimer = null
  }
}

function placeMenu() {
  const el = rootRef.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  const width = menuRef.value?.offsetWidth ?? 120
  const left = Math.min(Math.max(8, rect.right - width), window.innerWidth - width - 8)
  menuStyle.left = `${left}px`
  menuStyle.top = `${rect.bottom + 6}px`
}

function showMenu() {
  clearCloseTimer()
  open.value = true
  void nextTick(placeMenu)
}

function hideMenu() {
  clearCloseTimer()
  closeTimer = setTimeout(() => {
    open.value = false
    closeTimer = null
  }, 160)
}

function onCreate(event: MouseEvent) {
  open.value = false
  emit('create', event)
}

function onImport() {
  open.value = false
  emit('import')
}

function onExport() {
  open.value = false
  emit('export')
}

onBeforeUnmount(() => {
  clearCloseTimer()
})
</script>

<template>
  <div ref="rootRef" class="split-create">
    <button type="button" class="split-create__plus" :title="createTitle" @click="onCreate">
      <span class="split-create__plus-mark">+</span>
    </button>
    <span class="split-create__rule" aria-hidden="true" />
    <button
      type="button"
      class="split-create__caret"
      :class="{ 'is-open': open }"
      aria-label="more"
      @mouseenter="showMenu"
      @mouseleave="hideMenu"
      @click="showMenu"
    >
      <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.6">
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>
  </div>

  <Teleport to="body">
    <div
      v-if="open"
      ref="menuRef"
      class="split-create-menu"
      :style="menuStyle"
      @mouseenter="showMenu"
      @mouseleave="hideMenu"
    >
      <button type="button" @click="onImport">{{ importLabel }}</button>
      <button type="button" @click="onExport">{{ exportLabel }}</button>
    </div>
  </Teleport>
</template>

<style scoped>
.split-create {
  display: inline-flex;
  align-items: stretch;
  flex-shrink: 0;
  height: 34px;
  border-radius: 4px;
  overflow: hidden;
  background: var(--color-primary);
  color: #fff;
}

.split-create__plus,
.split-create__caret {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin: 0;
  border: 0;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
}

.split-create__plus {
  width: 36px;
}

.split-create__plus-mark {
  font-size: 20px;
  font-weight: 400;
  line-height: 1;
  color: #fff;
}

.split-create__caret {
  width: 20px;
}

.split-create__plus:hover,
.split-create__caret:hover,
.split-create__caret.is-open {
  background: var(--color-primary-hover);
}

.split-create__rule {
  width: 1px;
  height: 70%;
  align-self: center;
  transform: scaleX(0.5);
  background: rgba(255, 255, 255, 0.55);
}

.split-create-menu {
  position: fixed;
  z-index: 4000;
  min-width: 120px;
  padding: 6px;
  border-radius: 8px;
  background: var(--color-workspace-elevated, var(--color-surface));
  border: 1px solid var(--color-border);
  box-shadow: var(--color-menu-shadow, 0 8px 24px rgba(0, 0, 0, 0.12));
  display: flex;
  flex-direction: column;
}

.split-create-menu button {
  border: 0;
  background: transparent;
  color: var(--color-text);
  text-align: left;
  padding: 8px 12px;
  border-radius: 5px;
  font-size: 14px;
  cursor: pointer;
}

.split-create-menu button:hover {
  background: var(--color-workspace-menu-hover, var(--color-hover));
}
</style>
