<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref } from 'vue'

const props = defineProps<{
  modelValue: string
  options: readonly string[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const open = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const menuStyle = reactive({
  top: '0px',
  left: '0px',
  width: '0px',
  minWidth: '0px',
})

const currentLabel = computed(() => props.modelValue || 'string')

function placeMenu() {
  const el = rootRef.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  menuStyle.top = `${rect.bottom + 2}px`
  menuStyle.left = `${rect.left}px`
  menuStyle.minWidth = `${rect.width}px`
  menuStyle.width = `${rect.width}px`
}

function close() {
  open.value = false
}

function toggle() {
  if (open.value) {
    close()
    return
  }
  placeMenu()
  open.value = true
  void nextTick(placeMenu)
}

function pick(value: string) {
  emit('update:modelValue', value)
  close()
}

function onPointerDown(event: PointerEvent) {
  if (!open.value) return
  const target = event.target as Node | null
  if (rootRef.value?.contains(target) || menuRef.value?.contains(target)) return
  close()
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') close()
}

function onScroll() {
  if (open.value) placeMenu()
}

onMounted(() => {
  window.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('keydown', onKeydown)
  window.addEventListener('resize', onScroll)
  window.addEventListener('scroll', onScroll, true)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('resize', onScroll)
  window.removeEventListener('scroll', onScroll, true)
})
</script>

<template>
  <div ref="rootRef" class="interface-type-select" :class="{ open }">
    <button type="button" class="interface-type-select__trigger" @click.stop="toggle">
      <span class="interface-type-select__label">{{ currentLabel }}</span>
      <span class="interface-type-select__chevron" aria-hidden="true">
        <svg viewBox="0 0 12 12" width="12" height="12" fill="none">
          <path d="M3 4.5 6 8l3-3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </span>
    </button>
    <Teleport to="body">
      <div
        v-if="open"
        ref="menuRef"
        class="interface-type-select__menu"
        :style="menuStyle"
        @click.stop
      >
        <button
          v-for="type in options"
          :key="type"
          type="button"
          class="interface-type-select__item"
          :class="{ active: type === modelValue }"
          @click="pick(type)"
        >
          {{ type }}
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.interface-type-select {
  width: 100%;
  min-width: 0;
}

.interface-type-select__trigger {
  box-sizing: border-box;
  width: 100%;
  min-height: 32px;
  height: 32px;
  margin: 0;
  padding: 0 8px;
  border: none;
  border-radius: var(--color-interface-input-radius, 4px);
  background: transparent;
  color: var(--color-interface-field-text, var(--color-text));
  font: inherit;
  font-size: 14px;
  line-height: 32px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  outline: none;
}

.interface-type-select__trigger:hover,
.interface-type-select.open .interface-type-select__trigger {
  box-shadow: inset 0 0 0 1px var(--color-interface-focus-ring, var(--color-primary));
}

.interface-type-select__label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
}

.interface-type-select__chevron {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
  transition: transform 0.15s ease;
}

.interface-type-select.open .interface-type-select__chevron {
  transform: rotate(180deg);
}
</style>

<style>
.interface-type-select__menu {
  position: fixed;
  z-index: 4000;
  box-sizing: border-box;
  max-height: 240px;
  overflow: auto;
  padding: 6px 0;
  border-radius: 8px;
  background: var(--color-workspace-elevated, var(--color-surface));
  border: 1px solid var(--color-border);
  box-shadow: var(--color-menu-shadow, 0 8px 24px rgba(0, 0, 0, 0.18));
}

.interface-type-select__item {
  width: 100%;
  min-height: 36px;
  border: 0;
  background: transparent;
  color: var(--color-text);
  text-align: left;
  padding: 0 12px;
  border-radius: 0;
  font-size: 14px;
  line-height: 36px;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.interface-type-select__item:hover,
.interface-type-select__item.active {
  background: var(--color-workspace-menu-hover, var(--color-hover));
  color: var(--color-text);
}

.interface-type-select__item.active {
  color: var(--color-primary-light);
  font-weight: 500;
}
</style>
