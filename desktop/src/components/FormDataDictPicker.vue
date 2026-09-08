<template>
  <button
    ref="triggerRef"
    type="button"
    class="param-table__dict"
    :class="{ 'param-table__dict--matched': matched, 'param-table__dict--open': open }"
    :title="t('dict.pickForRow')"
    :aria-label="t('dict.pickForRow')"
    :aria-expanded="open"
    @click.stop="toggle"
  >
    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
      <path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H12v18H6.5A2.5 2.5 0 0 1 4 18.5v-13z" />
      <path d="M20 5.5A2.5 2.5 0 0 0 17.5 3H12v18h5.5A2.5 2.5 0 0 0 20 18.5v-13z" />
    </svg>
  </button>

  <Teleport to="body">
    <div
      v-if="open"
      ref="menuRef"
      class="dict-picker-menu"
      :style="menuStyle"
      role="listbox"
      @mousedown="onMenuMouseDown"
      @click.stop
    >
      <div class="dict-picker-menu__search">
        <input
          ref="searchRef"
          v-model="query"
          class="dict-picker-menu__search-input"
          type="text"
          :placeholder="t('dict.pickSearchPlaceholder')"
          @keydown.escape.stop.prevent="close"
        />
      </div>

      <div class="dict-picker-menu__cols" aria-hidden="true">
        <span>{{ t('dict.colName') }}</span>
        <span>{{ t('dict.colType') }}</span>
        <span>{{ t('dict.colDescription') }}</span>
      </div>

      <div class="dict-picker-menu__list">
        <button
          v-for="entry in filteredEntries"
          :key="entry.id"
          type="button"
          class="dict-picker-menu__item"
          :class="{ matched: dictMatchesParam(entry, paramName) }"
          role="option"
          @click="onPick(entry)"
        >
          <span class="dict-picker-menu__name" :title="entry.name">{{ entry.name }}</span>
          <span class="dict-picker-menu__type" :title="entry.field_type">{{ entry.field_type }}</span>
          <span class="dict-picker-menu__desc" :title="entry.description || ''">
            {{ entry.description || '—' }}
          </span>
        </button>
        <p v-if="!filteredEntries.length" class="dict-picker-menu__empty muted">
          {{ entries.length ? t('dict.pickSearchEmpty') : t('dict.pickEmpty') }}
        </p>
      </div>
      <div class="dict-picker-menu__footer">
        <button
          type="button"
          class="dict-picker-menu__apply-all"
          :disabled="!entries.length"
          @click="onApplyAll"
        >
          {{ t('dict.applyAllFormData') }}
        </button>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { useI18n } from '../i18n'
import { bindOutsidePointerDismiss } from '../composables/useOverlayDismiss'
import type { DictionaryEntryRow } from '../types'
import { dictMatchesParam, sortDictEntriesForPicker } from '../utils/dictionary-match'

const props = defineProps<{
  paramName: string
  entries: DictionaryEntryRow[]
  matched: boolean
}>()

const emit = defineEmits<{
  pick: [entry: DictionaryEntryRow]
  applyAll: []
  beforeOpen: []
}>()

const { t } = useI18n()
const open = ref(false)
const query = ref('')
const triggerRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const searchRef = ref<HTMLInputElement | null>(null)
const menuStyle = reactive({
  top: '0px',
  left: '0px',
  width: '420px',
})

const sortedEntries = computed(() => sortDictEntriesForPicker(props.entries, props.paramName))

const filteredEntries = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return sortedEntries.value
  return sortedEntries.value.filter((entry) => {
    const name = entry.name.toLowerCase()
    const desc = (entry.description ?? '').toLowerCase()
    return name.includes(q) || desc.includes(q)
  })
})

function placeMenu() {
  const el = triggerRef.value
  if (!el) return
  const width = 420
  const rect = el.getBoundingClientRect()
  let left = rect.right - width
  left = Math.max(8, Math.min(left, window.innerWidth - width - 8))
  let top = rect.bottom + 4
  const maxHeight = 400
  if (top + maxHeight > window.innerHeight - 8) {
    top = Math.max(8, rect.top - maxHeight - 4)
  }
  menuStyle.top = `${top}px`
  menuStyle.left = `${left}px`
  menuStyle.width = `${width}px`
}

async function openMenu() {
  emit('beforeOpen')
  query.value = ''
  open.value = true
  await nextTick()
  placeMenu()
  searchRef.value?.focus()
}

function close() {
  open.value = false
  query.value = ''
}

function toggle() {
  if (open.value) close()
  else void openMenu()
}

function onMenuMouseDown(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  if (target?.closest('input, textarea')) return
  event.preventDefault()
}

function onPick(entry: DictionaryEntryRow) {
  emit('pick', entry)
  close()
}

function onApplyAll() {
  emit('applyAll')
  close()
}

function onReposition() {
  if (open.value) placeMenu()
}

let unbindOutside: (() => void) | null = null

watch(
  () => props.paramName,
  () => {
    if (open.value) placeMenu()
  },
)

onMounted(() => {
  unbindOutside = bindOutsidePointerDismiss(
    () => open.value,
    () => [triggerRef.value, menuRef.value],
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
.param-table__dict {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  width: 28px;
  height: 28px;
  margin: 0;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  border-radius: var(--raw-editor-selection-radius, 4px);
  opacity: 0;
  pointer-events: none;
  position: relative;
  z-index: 2;
  transition:
    opacity 0.12s ease,
    color 0.12s ease,
    background-color 0.12s ease;
}

.param-table__dict--open {
  opacity: 1;
  pointer-events: auto;
  color: var(--color-interface-accent, var(--color-primary));
  background: color-mix(in srgb, var(--color-interface-accent, var(--color-primary)) 16%, transparent);
}

.param-table__dict:hover {
  color: var(--color-interface-accent, var(--color-primary));
  background: color-mix(in srgb, var(--color-interface-accent, var(--color-primary)) 16%, transparent);
}

.dict-picker-menu {
  --dict-col-name: minmax(96px, 1.2fr);
  --dict-col-type: 72px;
  --dict-col-desc: minmax(0, 1.6fr);
  position: fixed;
  z-index: 4000;
  display: flex;
  flex-direction: column;
  max-height: 400px;
  padding: 8px;
  border-radius: 8px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
  box-sizing: border-box;
}

.dict-picker-menu__search {
  flex-shrink: 0;
  margin-bottom: 6px;
}

.dict-picker-menu__search-input {
  width: 100%;
  height: 30px;
  box-sizing: border-box;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface, var(--color-menu));
  color: var(--color-text);
  padding: 0 10px;
  font-size: 13px;
  outline: none;
}

.dict-picker-menu__search-input:focus {
  border-color: var(--color-interface-accent, var(--color-primary));
}

.dict-picker-menu__search-input::placeholder {
  color: var(--color-text-secondary);
}

.dict-picker-menu__cols,
.dict-picker-menu__item {
  display: grid;
  grid-template-columns: var(--dict-col-name) var(--dict-col-type) var(--dict-col-desc);
  column-gap: 10px;
  align-items: center;
  width: 100%;
  box-sizing: border-box;
  padding: 6px 10px;
}

.dict-picker-menu__cols {
  flex-shrink: 0;
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: 600;
  line-height: 1.2;
}

.dict-picker-menu__list {
  flex: 1;
  min-height: 0;
  overflow: auto;
  scrollbar-gutter: stable;
}

.dict-picker-menu__item {
  border: 0;
  background: transparent;
  color: var(--color-text);
  text-align: left;
  border-radius: 5px;
  cursor: pointer;
  min-height: 32px;
}

.dict-picker-menu__item:hover,
.dict-picker-menu__item.matched {
  background: var(--color-menu-hover);
}

.dict-picker-menu__name,
.dict-picker-menu__type,
.dict-picker-menu__desc {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dict-picker-menu__name {
  font-size: 13px;
  font-weight: 600;
}

.dict-picker-menu__type {
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  color: var(--color-text-secondary);
}

.dict-picker-menu__desc {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.dict-picker-menu__empty {
  margin: 12px 8px;
  font-size: 13px;
  text-align: center;
}

.dict-picker-menu__footer {
  flex-shrink: 0;
  padding-top: 6px;
  margin-top: 4px;
  border-top: 1px solid var(--color-border);
}

.dict-picker-menu__apply-all {
  width: 100%;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--color-interface-accent, var(--color-primary));
  font-size: 12px;
  line-height: 1.3;
  padding: 8px 10px;
  cursor: pointer;
  text-align: center;
}

.dict-picker-menu__apply-all:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-interface-accent, var(--color-primary)) 10%, transparent);
}

.dict-picker-menu__apply-all:disabled {
  opacity: 0.45;
  cursor: default;
}
</style>
