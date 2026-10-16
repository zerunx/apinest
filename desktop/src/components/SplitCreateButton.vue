<script setup lang="ts">
import { nextTick, onMounted, onBeforeUnmount, reactive, ref } from 'vue'
import { bindOutsidePointerDismiss } from '../composables/useOverlayDismiss'

const emit = defineEmits<{
  'add-api': [event: MouseEvent]
  'add-folder': [event: MouseEvent]
  import: []
  export: []
}>()

defineProps<{
  addApiLabel: string
  addFolderLabel: string
  importLabel: string
  exportLabel: string
}>()

const createOpen = ref(false)
const moreOpen = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const createMenuRef = ref<HTMLElement | null>(null)
const moreMenuRef = ref<HTMLElement | null>(null)
const createMenuStyle = reactive({ left: '0px', top: '0px' })
const moreMenuStyle = reactive({ left: '0px', top: '0px' })
let closeTimer: ReturnType<typeof setTimeout> | null = null
let unbindOutside: (() => void) | null = null

function clearCloseTimer() {
  if (closeTimer) {
    clearTimeout(closeTimer)
    closeTimer = null
  }
}

function placeMenu(
  style: { left: string; top: string },
  menuEl: HTMLElement | null,
) {
  const el = rootRef.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  const width = menuEl?.offsetWidth ?? 140
  const left = Math.min(Math.max(8, rect.right - width), window.innerWidth - width - 8)
  style.left = `${left}px`
  style.top = `${rect.bottom + 6}px`
}

async function showCreateMenu() {
  clearCloseTimer()
  moreOpen.value = false
  createOpen.value = !createOpen.value
  if (!createOpen.value) return
  await nextTick()
  placeMenu(createMenuStyle, createMenuRef.value)
  await nextTick()
  placeMenu(createMenuStyle, createMenuRef.value)
}

function showMoreMenu() {
  clearCloseTimer()
  createOpen.value = false
  moreOpen.value = true
  void nextTick(() => placeMenu(moreMenuStyle, moreMenuRef.value))
}

function hideMoreMenu() {
  clearCloseTimer()
  closeTimer = setTimeout(() => {
    moreOpen.value = false
    closeTimer = null
  }, 160)
}

function onAddApi(event: MouseEvent) {
  createOpen.value = false
  emit('add-api', event)
}

function onAddFolder(event: MouseEvent) {
  createOpen.value = false
  emit('add-folder', event)
}

function onImport() {
  moreOpen.value = false
  emit('import')
}

function onExport() {
  moreOpen.value = false
  emit('export')
}

onMounted(() => {
  unbindOutside = bindOutsidePointerDismiss(
    () => createOpen.value,
    () => [rootRef.value, createMenuRef.value],
    () => {
      createOpen.value = false
    },
  )
})

onBeforeUnmount(() => {
  clearCloseTimer()
  unbindOutside?.()
})
</script>

<template>
  <div ref="rootRef" class="split-create">
    <button
      type="button"
      class="split-create__plus"
      :class="{ 'is-open': createOpen }"
      :title="addApiLabel"
      @click.stop="showCreateMenu"
    >
      <svg
        class="split-create__plus-icon"
        viewBox="0 0 24 24"
        width="16"
        height="16"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        aria-hidden="true"
      >
        <path d="M12 5v14M5 12h14" />
      </svg>
    </button>
    <span class="split-create__rule" aria-hidden="true" />
    <button
      type="button"
      class="split-create__caret"
      :class="{ 'is-open': moreOpen }"
      aria-label="more"
      @mouseenter="showMoreMenu"
      @mouseleave="hideMoreMenu"
      @click.stop="showMoreMenu"
    >
      <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.6">
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>
  </div>

  <Teleport to="body">
    <div
      v-if="createOpen"
      ref="createMenuRef"
      class="split-create-menu"
      :style="createMenuStyle"
      @click.stop
    >
      <button type="button" @click="onAddApi($event)">{{ addApiLabel }}</button>
      <button type="button" @click="onAddFolder($event)">{{ addFolderLabel }}</button>
    </div>
  </Teleport>

  <Teleport to="body">
    <div
      v-if="moreOpen"
      ref="moreMenuRef"
      class="split-create-menu"
      :style="moreMenuStyle"
      @mouseenter="showMoreMenu"
      @mouseleave="hideMoreMenu"
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
  height: 28px;
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
  width: 34px;
}

.split-create__plus-icon {
  display: block;
}

.split-create__caret {
  width: 18px;
}

.split-create__plus:hover,
.split-create__plus.is-open,
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
  min-width: 140px;
  padding: 6px;
  border-radius: 8px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
  display: flex;
  flex-direction: column;
}

.split-create-menu button {
  border: 0;
  background: transparent;
  color: inherit;
  text-align: left;
  padding: 7px 10px;
  border-radius: 5px;
  font-size: 13px;
  cursor: pointer;
}

.split-create-menu button:hover {
  background: var(--color-menu-hover);
}
</style>
