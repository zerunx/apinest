<template>
  <div class="menu-select" :class="{ open, compact, dense, stretch, plain }" ref="rootRef">
    <button type="button" class="menu-select__trigger" @click.stop="toggle">
      <span class="menu-select__label">{{ currentLabel }}</span>
      <span class="menu-select__chevron" :class="{ open }" aria-hidden="true">
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
        class="menu-select__menu"
        :class="{ 'menu-select__menu--fit-content': fitContent }"
        :style="menuStyle"
        @click.stop
      >
        <template v-for="(opt, idx) in options" :key="opt.separator ? `sep-${idx}` : String(opt.value)">
          <div v-if="opt.separator" class="menu-select__separator" role="separator" />
          <button
            v-else
            type="button"
            class="menu-select__item"
            :class="{ active: opt.value === modelValue }"
            :disabled="opt.disabled"
            @click="pick(opt.value)"
          >
            {{ opt.label }}
          </button>
        </template>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref } from 'vue'
import { bindOutsidePointerDismiss } from '../composables/useOverlayDismiss'

export type MenuOption = {
  value?: string | number | null
  label?: string
  separator?: boolean
  disabled?: boolean
}

const props = withDefaults(
  defineProps<{
    modelValue: string | number | null
    options: MenuOption[]
    placeholder?: string
    compact?: boolean
    fitMenu?: boolean
    fitContent?: boolean
    dense?: boolean
    stretch?: boolean
    plain?: boolean
    menuMinWidth?: number
    menuWidth?: number
    menuAlign?: 'left' | 'right'
  }>(),
  {
    placeholder: '—',
    compact: false,
    fitMenu: false,
    fitContent: false,
    dense: false,
    stretch: false,
    plain: false,
    menuMinWidth: 0,
    menuWidth: 0,
    menuAlign: 'left',
  },
)

const emit = defineEmits<{
  'update:modelValue': [string | number | null]
}>()

const open = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const menuStyle = reactive({
  top: '0px',
  left: '0px',
  minWidth: '0px',
  width: '',
})

const currentLabel = computed(() => {
  const hit = props.options.find((o) => !o.separator && o.value === props.modelValue)
  return hit?.label ?? props.placeholder
})

function placeMenu() {
  const el = rootRef.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  menuStyle.top = `${rect.bottom + 4}px`
  if (props.menuWidth > 0) {
    const width = props.menuWidth
    menuStyle.minWidth = `${width}px`
    menuStyle.width = `${width}px`
    menuStyle.left =
      props.menuAlign === 'right' ? `${rect.right - width}px` : `${rect.left}px`
    return
  }
  if (props.fitContent) {
    // Match trigger width as floor; CSS max-width caps growth.
    const minWidth = Math.max(rect.width, props.menuMinWidth)
    menuStyle.minWidth = `${minWidth}px`
    menuStyle.width = ''
    menuStyle.top = props.dense ? `${rect.bottom + 2}px` : `${rect.bottom + 4}px`
    if (props.menuAlign === 'right' && menuRef.value) {
      menuStyle.left = `${rect.right - menuRef.value.offsetWidth}px`
    } else {
      menuStyle.left = `${rect.left}px`
    }
    return
  }
  if (props.fitMenu) {
    const width = Math.max(rect.width, props.menuMinWidth)
    menuStyle.minWidth = `${width}px`
    menuStyle.width = `${width}px`
    menuStyle.left =
      props.menuAlign === 'right' ? `${rect.right - width}px` : `${rect.left}px`
    // Keep dense menus visually attached under the trigger.
    menuStyle.top = props.dense ? `${rect.bottom + 2}px` : `${rect.bottom + 4}px`
  } else {
    menuStyle.minWidth = `${Math.max(rect.width, 180)}px`
    menuStyle.width = ''
    menuStyle.left = `${rect.left}px`
  }
}

async function toggle() {
  open.value = !open.value
  if (open.value) {
    placeMenu()
    await nextTick()
    placeMenu()
  }
}

function pick(value: string | number | null | undefined) {
  if (value === undefined) return
  emit('update:modelValue', value)
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
.menu-select {
  position: relative;
  min-width: 0;
}

.menu-select:not(.compact) {
  height: 100%;
}

.menu-select.compact {
  height: 34px;
}

.menu-select.compact.stretch {
  height: 100%;
  min-height: 36px;
}

.menu-select__trigger {
  width: 100%;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  border: 0;
  background: transparent;
  color: var(--color-interface-field-text, var(--color-text));
  padding: 0 8px;
  text-align: left;
  cursor: pointer;
  box-sizing: border-box;
}

.menu-select:not(.compact) .menu-select__trigger {
  height: 100%;
  min-height: 34px;
}

.menu-select.compact .menu-select__trigger {
  height: 34px;
  min-height: 34px;
  max-height: 34px;
  border: 1px solid var(--color-interface-bar-border, var(--color-border));
  border-radius: var(--color-interface-input-radius, 4px);
}

.menu-select.compact.stretch .menu-select__trigger {
  height: 100%;
  min-height: 36px;
  max-height: none;
}

.menu-select.compact.plain {
  height: 100%;
  min-height: 0;
}

.menu-select.compact.plain .menu-select__trigger {
  height: 100%;
  min-height: 0;
  max-height: none;
  border: 0;
  border-radius: var(--raw-editor-selection-radius, 4px);
  background: transparent;
  box-shadow: none;
}

.menu-select__label {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
}

.menu-select__chevron {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 12px;
  color: var(--color-text-secondary);
  transition: transform 0.15s ease;
  flex-shrink: 0;
}

.menu-select__chevron.open {
  transform: rotate(180deg);
}

.menu-select__menu {
  position: fixed;
  z-index: 4000;
  max-height: min(360px, calc(100vh - 24px));
  overflow: auto;
  padding: 6px;
  border-radius: 8px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
}

.menu-select__menu--fit-content {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  width: max-content;
  max-width: min(280px, calc(100vw - 24px));
}

.menu-select__menu--fit-content .menu-select__item {
  flex: 0 0 auto;
  width: auto;
  min-width: 100%;
  box-sizing: border-box;
  overflow: hidden;
  text-overflow: ellipsis;
}

.menu-select__item {
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--color-text);
  text-align: left;
  padding: 7px 10px;
  border-radius: 5px;
  font-size: 13px;
  line-height: 20px;
  cursor: pointer;
  white-space: nowrap;
}

.menu-select__item:hover,
.menu-select__item.active {
  background: var(--color-menu-hover);
}

.menu-select__item:disabled {
  opacity: 0.45;
  cursor: default;
}

.menu-select__separator {
  height: 1px;
  margin: 6px 4px;
  background: var(--color-border);
}

.menu-select.dense .menu-select__menu {
  padding: 4px;
  border-radius: 6px;
  overflow: hidden;
}

.menu-select.dense .menu-select__item {
  padding: 6px 10px;
  font-size: 12px;
  border-radius: 4px;
}
</style>
