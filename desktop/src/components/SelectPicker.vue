<template>
  <div class="picker" ref="rootRef">
    <button type="button" class="picker-trigger" @click.stop="toggle">
      <span class="picker-icon" aria-hidden="true" v-html="iconSvg" />
      <span class="picker-label">{{ currentLabel }}</span>
      <span class="picker-chevron" :class="{ open }" aria-hidden="true">
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
      </span>
    </button>

    <div v-if="open" class="picker-menu" :class="{ 'align-right': align === 'right' }" @click.stop>
      <input
        v-model="query"
        class="picker-search"
        type="search"
        :placeholder="searchPlaceholder"
        autofocus
      />
      <button
        v-if="allowEmpty"
        type="button"
        class="picker-item"
        :class="{ active: modelValue == null }"
        @click="pick(null)"
      >
        <span class="picker-icon" v-html="iconSvg" />
        <span>{{ emptyLabel }}</span>
      </button>
      <div
        v-for="item in filtered"
        :key="item.id"
        class="picker-row"
        :class="{ active: item.id === modelValue }"
      >
        <button type="button" class="picker-item picker-item-main" @click="pick(item.id)">
          <span class="picker-icon" v-html="iconSvg" />
          <span class="picker-item-name">{{ item.name }}</span>
        </button>
        <div v-if="itemActions" class="picker-item-actions">
          <button
            type="button"
            class="picker-action"
            :title="t('common.edit')"
            @click.stop="onEdit(item, $event)"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
              <path d="M12 20h9" />
              <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z" />
            </svg>
          </button>
          <button
            type="button"
            class="picker-action danger"
            :title="t('common.delete')"
            @click.stop="onDelete(item, $event)"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
              <path d="M3 6h18" />
              <path d="M8 6V4h8v2" />
              <path d="M19 6l-1 14H6L5 6" />
              <path d="M10 11v6M14 11v6" />
            </svg>
          </button>
        </div>
      </div>
      <p v-if="!filtered.length" class="picker-empty muted">{{ emptyResult }}</p>
      <div v-if="createLabel" class="picker-footer">
        <button type="button" class="picker-create" @click="onCreate($event)">
          <span>{{ createLabel }}</span>
        </button>
      </div>
      <button
        v-if="manageLabel"
        type="button"
        class="picker-item picker-manage"
        @click="onManage($event)"
      >
        {{ manageLabel }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { bindOutsidePointerDismiss } from '../composables/useOverlayDismiss'
import { useI18n } from '../i18n'

export type PickerItem = { id: number; name: string }

const props = withDefaults(
  defineProps<{
    modelValue: number | null
    items: PickerItem[]
    placeholder: string
    searchPlaceholder: string
    emptyResult: string
    emptyLabel?: string
    allowEmpty?: boolean
    variant?: 'project' | 'env'
    align?: 'left' | 'right'
    createLabel?: string
    manageLabel?: string
    itemActions?: boolean
  }>(),
  {
    allowEmpty: false,
    emptyLabel: '—',
    variant: 'project',
    align: 'left',
    createLabel: '',
    manageLabel: '',
    itemActions: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [number | null]
  create: [MouseEvent]
  manage: [MouseEvent]
  edit: [item: PickerItem, event: MouseEvent]
  delete: [item: PickerItem, event: MouseEvent]
}>()

const { t } = useI18n()
const open = ref(false)
const query = ref('')
const rootRef = ref<HTMLElement | null>(null)

const currentLabel = computed(() => {
  if (props.modelValue == null) return props.placeholder
  return props.items.find((i) => i.id === props.modelValue)?.name || props.placeholder
})

const iconSvg = computed(() =>
  props.variant === 'project'
    ? `<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 8.5L12 4l8 4.5v7L12 20l-8-4.5v-7z"/><path d="M12 12l8-4.5M12 12v8M12 12L4 7.5"/></svg>`
    : `<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="3"/><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1"/></svg>`,
)

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return props.items
  return props.items.filter((i) => i.name.toLowerCase().includes(q))
})

watch(open, (v) => {
  if (v) query.value = ''
})

function toggle() {
  open.value = !open.value
}

function pick(id: number | null) {
  emit('update:modelValue', id)
  open.value = false
}

function onCreate(event: MouseEvent) {
  open.value = false
  emit('create', event)
}

function onManage(event: MouseEvent) {
  open.value = false
  emit('manage', event)
}

function onEdit(item: PickerItem, event: MouseEvent) {
  open.value = false
  emit('edit', item, event)
}

function onDelete(item: PickerItem, event: MouseEvent) {
  open.value = false
  emit('delete', item, event)
}

let unbindOutside: (() => void) | null = null

onMounted(() => {
  unbindOutside = bindOutsidePointerDismiss(
    () => open.value,
    () => [rootRef.value],
    () => {
      open.value = false
    },
  )
})
onUnmounted(() => {
  unbindOutside?.()
})
</script>

<style scoped>
.picker {
  position: relative;
  min-width: 0;
}

.picker-trigger {
  width: 100%;
  max-width: 100%;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  border: 0;
  background: transparent;
  color: inherit;
  padding: 4px 8px;
  border-radius: 6px;
  text-align: left;
}

.picker-trigger:hover {
  background: var(--color-hover);
}

.picker-icon {
  display: inline-flex;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.picker-label {
  flex: 1;
  min-width: 0;
  font-size: 14px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.picker-chevron {
  display: inline-flex;
  color: var(--color-text-secondary);
  transition: transform 0.15s ease;
  flex-shrink: 0;
}

.picker-chevron.open {
  transform: rotate(180deg);
}

.picker-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  z-index: 40;
  width: 260px;
  padding: 8px;
  border-radius: 10px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
  overflow: hidden;
}

.picker-menu.align-right {
  left: auto;
  right: 0;
}

.picker-search {
  width: 100%;
  margin-bottom: 6px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 7px 10px;
  background: var(--color-surface);
  outline: none;
  font-size: 13px;
}

.picker-search:focus {
  border-color: var(--color-primary);
}

.picker-row {
  display: flex;
  align-items: center;
  gap: 2px;
  border-radius: 6px;
}

.picker-row:hover,
.picker-row.active {
  background: var(--color-menu-hover);
}

.picker-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  border: 0;
  background: transparent;
  color: inherit;
  text-align: left;
  padding: 8px 10px;
  border-radius: 6px;
  font-size: 14px;
}

.picker-item:hover,
.picker-item.active {
  background: var(--color-menu-hover);
}

.picker-item-main {
  flex: 1;
  min-width: 0;
  padding-right: 4px;
}

.picker-row:hover .picker-item-main,
.picker-row.active .picker-item-main,
.picker-item-main:hover {
  background: transparent;
}

.picker-item-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.picker-item-actions {
  display: none;
  align-items: center;
  gap: 1px;
  padding-right: 4px;
  flex-shrink: 0;
}

.picker-row:hover .picker-item-actions,
.picker-row.active .picker-item-actions {
  display: inline-flex;
}

.picker-action {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--color-text-secondary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.picker-action:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.picker-action.danger:hover {
  background: var(--color-danger-hover);
  color: var(--color-danger);
}

.picker-empty {
  margin: 4px 0;
  padding: 8px 10px;
  font-size: 13px;
}

.picker-footer {
  margin-top: 4px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border);
}

.picker-create {
  width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 38px;
  border: 1px dashed var(--color-border);
  border-radius: 8px;
  background: transparent;
  color: var(--color-primary-light);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s ease, border-color 0.15s ease;
}

.picker-create:hover {
  background: var(--color-active);
  border-color: var(--color-primary);
}

.picker-manage {
  margin-top: 4px;
  border-top: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text-secondary);
  font-weight: 600;
  justify-content: center;
}

.picker-manage:hover {
  background: var(--color-menu-hover);
  color: var(--color-text);
}
</style>
