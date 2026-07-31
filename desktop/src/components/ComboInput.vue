<template>
  <div
    class="combo-input"
    ref="rootRef"
    :class="{ 'combo-input--open': open }"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  >
    <input
      class="combo-input__field"
      :value="modelValue"
      :placeholder="placeholder"
      @input="onInput"
      @focus="onFocus"
      @keydown.escape="close"
    />
    <button
      type="button"
      class="combo-input__chevron"
      :class="{ 'combo-input__chevron--visible': hovered || open, open }"
      tabindex="-1"
      aria-label="Toggle options"
      @mousedown.prevent
      @click="toggle"
    >
      <svg
        viewBox="0 0 24 24"
        width="14"
        height="14"
        fill="none"
        stroke="currentColor"
        stroke-width="2.4"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>

    <Teleport to="body">
      <div
        v-if="open"
        ref="menuRef"
        class="combo-input__menu"
        :style="menuStyle"
        @mousedown.prevent
        @click.stop
      >
        <button
          v-for="opt in filteredOptions"
          :key="opt"
          type="button"
          class="combo-input__item"
          :class="{ active: opt === modelValue }"
          @click="pick(opt)"
        >
          {{ opt }}
        </button>
        <p v-if="!filteredOptions.length" class="combo-input__empty muted">{{ emptyText }}</p>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref } from 'vue'
import { bindOutsidePointerDismiss } from '../composables/useOverlayDismiss'

const props = withDefaults(
  defineProps<{
    modelValue: string
    options: string[]
    placeholder?: string
    emptyText?: string
  }>(),
  {
    placeholder: '',
    emptyText: '—',
  },
)

const emit = defineEmits<{
  'update:modelValue': [string]
  input: []
}>()

const open = ref(false)
const hovered = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const menuStyle = reactive({
  top: '0px',
  left: '0px',
  width: '0px',
})

const filteredOptions = computed(() => {
  const q = props.modelValue.trim().toLowerCase()
  if (!q) return props.options
  return props.options.filter((opt) => opt.toLowerCase().includes(q))
})

function measureOptionsMinWidth() {
  const sample = filteredOptions.value.length ? filteredOptions.value : props.options
  if (!sample.length) return 0
  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    const longest = sample.reduce((a, b) => (a.length >= b.length ? a : b), '')
    return Math.ceil(longest.length * 8.2 + 36)
  }
  const style = rootRef.value ? getComputedStyle(rootRef.value) : null
  const fontSize = style?.fontSize || '13px'
  const fontFamily = style?.fontFamily || 'ui-sans-serif, system-ui, sans-serif'
  ctx.font = `${fontSize} ${fontFamily}`
  let max = 0
  for (const opt of sample) {
    max = Math.max(max, ctx.measureText(opt).width)
  }
  // item horizontal padding (10+10) + scrollbar cushion
  return Math.ceil(max + 36)
}

function placeMenu() {
  const el = rootRef.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  const minWidth = measureOptionsMinWidth()
  const width = Math.max(rect.width, minWidth)
  menuStyle.top = `${rect.bottom + 4}px`
  menuStyle.left = `${rect.left}px`
  menuStyle.width = `${width}px`

  // Keep menu inside the viewport on the right edge.
  const overflow = rect.left + width - window.innerWidth + 8
  if (overflow > 0) {
    menuStyle.left = `${Math.max(8, rect.left - overflow)}px`
  }
}

async function openMenu() {
  open.value = true
  await nextTick()
  placeMenu()
}

function close() {
  open.value = false
}

function toggle() {
  if (open.value) close()
  else void openMenu()
}

function onFocus() {
  void openMenu()
}

function onInput(e: Event) {
  emit('update:modelValue', (e.target as HTMLInputElement).value)
  emit('input')
  if (!open.value) void openMenu()
  else placeMenu()
}

function pick(value: string) {
  emit('update:modelValue', value)
  emit('input')
  close()
}

function onReposition() {
  if (open.value) placeMenu()
}

let unbindOutside: (() => void) | null = null

onMounted(() => {
  unbindOutside = bindOutsidePointerDismiss(
    () => open.value,
    () => [rootRef.value, menuRef.value],
    close,
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
.combo-input {
  position: relative;
  display: flex;
  align-items: stretch;
  min-width: 0;
  height: 100%;
}

.combo-input__field {
  flex: 1;
  min-width: 0;
  width: 100%;
  height: 100%;
  border: 0;
  border-radius: 0;
  background: transparent;
  color: var(--color-interface-field-text, var(--color-text));
  padding: 0 28px 0 10px;
  font: inherit;
  font-size: 13px;
  outline: none;
}

.combo-input__field::placeholder {
  color: var(--color-interface-placeholder, var(--color-text-secondary));
  opacity: 1;
}

.combo-input__field:focus {
  background: color-mix(in srgb, var(--color-interface-accent, var(--color-primary)) 6%, transparent);
}

.combo-input__chevron {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  opacity: 0;
  cursor: pointer;
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.combo-input__chevron--visible {
  opacity: 1;
}

.combo-input__chevron.open {
  transform: rotate(180deg);
}

.combo-input__chevron:hover {
  color: var(--color-text);
}

.combo-input__menu {
  position: fixed;
  z-index: 4000;
  max-height: 320px;
  overflow: auto;
  padding: 6px;
  border-radius: 8px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
}

.combo-input__item {
  display: block;
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--color-text);
  text-align: left;
  padding: 9px 10px;
  border-radius: 5px;
  font-size: 13px;
  white-space: nowrap;
  cursor: pointer;
}

.combo-input__item:hover,
.combo-input__item.active {
  background: var(--color-menu-hover);
}

.combo-input__empty {
  margin: 8px 6px;
  font-size: 13px;
}
</style>
