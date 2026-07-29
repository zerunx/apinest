<template>
  <li class="node">
    <template v-if="item.type === 'folder'">
      <div class="row folder" :class="{ 'menu-open': menuOpen }" @click="open = !open">
        <span class="expand-zone" :style="expandZoneStyle" @click.stop="open = !open">
          <svg
            class="expand-icon"
            :class="{ open }"
            viewBox="0 0 24 24"
            width="12"
            height="12"
            aria-hidden="true"
          >
            <path
              fill="currentColor"
              d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6z"
            />
          </svg>
        </span>
        <svg
          class="folder-icon"
          viewBox="0 0 24 24"
          width="14"
          height="14"
          aria-hidden="true"
        >
          <path
            fill="currentColor"
            d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"
          />
        </svg>
        <span class="name">{{ item.name }}</span>
        <div class="row-menu" ref="menuRef" @click.stop>
          <button
            ref="menuBtnRef"
            type="button"
            class="row-menu-btn"
            :class="{ active: menuOpen }"
            :title="t('tree.more')"
            @click="toggleMenu"
          >
            <span class="row-menu-icon" aria-hidden="true" />
          </button>
          <Teleport to="body">
            <div
              v-if="menuOpen"
              ref="menuPopRef"
              class="row-menu-pop"
              :style="menuStyle"
              @click.stop
            >
              <button type="button" @click="emitAction('add-api')">{{ t('tree.addApi') }}</button>
              <button type="button" @click="emitAction('add-subfolder')">
                {{ t('tree.addSubFolder') }}
              </button>
              <button type="button" @click="emitAction('rename')">{{ t('tree.rename') }}</button>
              <button type="button" class="danger" @click="emitAction('delete')">
                {{ t('tree.delete') }}
              </button>
            </div>
          </Teleport>
        </div>
      </div>
      <ul v-if="open" class="children">
        <TreeNode
          v-for="child in item.children"
          :key="`${child.type}-${child.id}`"
          :item="child"
          :active-id="activeId"
          :depth="depth + 1"
          @select="$emit('select', $event)"
          @action="$emit('action', $event)"
        />
      </ul>
    </template>
    <div
      v-else
      class="row api"
      :class="{ active: activeId === item.id, 'menu-open': menuOpen }"
      @click="$emit('select', item.id)"
    >
      <span class="expand-zone leaf" :style="expandZoneStyle" />
      <span class="method-wrap">
        <span class="method" :class="item.method.toLowerCase()">{{ item.method }}</span>
      </span>
      <span class="name">{{ item.name }}</span>
      <div class="row-menu" ref="menuRef" @click.stop>
        <button
          ref="menuBtnRef"
          type="button"
          class="row-menu-btn"
          :class="{ active: menuOpen }"
          :title="t('tree.more')"
          @click="toggleMenu"
        >
          <span class="row-menu-icon" aria-hidden="true" />
        </button>
        <Teleport to="body">
          <div
            v-if="menuOpen"
            ref="menuPopRef"
            class="row-menu-pop"
            :style="menuStyle"
            @click.stop
          >
            <button type="button" @click="emitAction('edit')">{{ t('tree.edit') }}</button>
            <button type="button" class="danger" @click="emitAction('delete')">
              {{ t('tree.delete') }}
            </button>
          </div>
        </Teleport>
      </div>
    </div>
  </li>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref } from 'vue'
import type { TreeItem } from '../types'
import type { DialogAnchor } from '../composables/useDialogAnchor'
import { bindOutsidePointerDismiss } from '../composables/useOverlayDismiss'
import { useI18n } from '../i18n'

export type TreeAction =
  | {
      type: 'folder'
      id: number
      name: string
      action: 'add-api' | 'add-subfolder' | 'rename' | 'delete'
      anchor?: DialogAnchor | null
    }
  | {
      type: 'api'
      id: number
      name: string
      action: 'edit' | 'delete'
      anchor?: DialogAnchor | null
    }

const props = withDefaults(
  defineProps<{
    item: TreeItem
    activeId: number | null
    depth?: number
  }>(),
  { depth: 0 },
)

const emit = defineEmits<{
  select: [id: number]
  action: [TreeAction]
}>()

const { t } = useI18n()
const open = ref(true)
const menuOpen = ref(false)
const menuRef = ref<HTMLElement | null>(null)
const menuBtnRef = ref<HTMLButtonElement | null>(null)
const menuPopRef = ref<HTMLElement | null>(null)
const menuStyle = reactive({
  top: '0px',
  left: '0px',
})

/** Keep in sync with web AppContextPanel: 12 + depth*16 + 16 */
const INDENT_STEP = 16
const EXPAND_BASE = 28

const expandZoneStyle = computed(() => ({
  width: `${EXPAND_BASE + props.depth * INDENT_STEP}px`,
}))

function placeMenu() {
  const btn = menuBtnRef.value
  if (!btn) return
  const rect = btn.getBoundingClientRect()
  const menuEl = menuPopRef.value
  const menuWidth = menuEl?.offsetWidth ?? 132
  const menuHeight = menuEl?.offsetHeight ?? 120
  const padding = 8

  let left = rect.right - menuWidth
  let top = rect.bottom + 4

  if (left < padding) left = padding
  if (left + menuWidth > window.innerWidth - padding) {
    left = window.innerWidth - menuWidth - padding
  }
  if (top + menuHeight > window.innerHeight - padding) {
    top = Math.max(padding, rect.top - menuHeight - 4)
  }

  menuStyle.top = `${top}px`
  menuStyle.left = `${left}px`
}

async function toggleMenu() {
  menuOpen.value = !menuOpen.value
  if (menuOpen.value) {
    await nextTick()
    placeMenu()
    await nextTick()
    placeMenu()
  }
}

function actionAnchor(): DialogAnchor | null {
  const btn = menuBtnRef.value
  if (!btn) return null
  const rect = btn.getBoundingClientRect()
  return { x: rect.left, y: rect.bottom + 4 }
}

function emitAction(action: TreeAction['action']) {
  menuOpen.value = false
  const anchor = actionAnchor()
  if (props.item.type === 'folder') {
    emit('action', {
      type: 'folder',
      id: props.item.id,
      name: props.item.name,
      action: action as 'add-api' | 'add-subfolder' | 'rename' | 'delete',
      anchor,
    })
  } else {
    emit('action', {
      type: 'api',
      id: props.item.id,
      name: props.item.name,
      action: action as 'edit' | 'delete',
      anchor,
    })
  }
}

function onReposition() {
  if (menuOpen.value) placeMenu()
}

let unbindOutside: (() => void) | null = null

onMounted(() => {
  unbindOutside = bindOutsidePointerDismiss(
    () => menuOpen.value,
    () => [menuRef.value, menuPopRef.value],
    () => {
      menuOpen.value = false
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

<script lang="ts">
export default {
  name: 'TreeNode',
}
</script>

<style scoped>
.node {
  list-style: none;
}

.children {
  list-style: none;
  margin: 0;
  padding: 0;
}

.row {
  width: 100%;
  height: 30px;
  display: flex;
  align-items: center;
  gap: 6px;
  border: 0;
  background: transparent;
  text-align: left;
  padding: 0 4px 0 0;
  border-radius: 0;
  cursor: pointer;
  user-select: none;
  position: relative;
}

.row.folder {
  gap: 4px;
}

.method-wrap {
  box-sizing: border-box;
  width: 48px;
  min-width: 48px;
  max-width: 48px;
  height: 100%;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  overflow: hidden;
}

.row.api .method {
  display: block;
  width: 100%;
  min-width: 0;
  overflow: hidden;
  text-align: left;
  font-size: 12px;
  font-weight: 700;
  line-height: 1;
  font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, Consolas, monospace);
}

.row:hover,
.row.menu-open {
  background: var(--color-hover);
}

.row.active {
  background: var(--color-active);
}

.expand-zone {
  flex-shrink: 0;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  color: var(--color-text-secondary);
  padding-right: 2px;
  box-sizing: border-box;
}

.expand-zone.leaf {
  pointer-events: none;
}

.expand-icon {
  display: block;
  transition: transform 0.15s ease;
  transform: rotate(-90deg);
}

.expand-icon.open {
  transform: rotate(0deg);
}

.folder-icon {
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  min-width: 0;
  flex: 1;
}

.row-menu {
  margin-left: auto;
  flex-shrink: 0;
  opacity: 0;
  position: relative;
  transition: opacity 0.15s ease;
}

.row:hover .row-menu,
.row.menu-open .row-menu,
.row.active .row-menu {
  opacity: 1;
}

.row-menu-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.row-menu-btn:hover,
.row-menu-btn.active {
  background: color-mix(in srgb, var(--color-text) 10%, transparent);
}

.row-menu-icon {
  position: relative;
  display: block;
  width: 14px;
  height: 12px;
}

.row-menu-icon::before {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  height: 2px;
  border-radius: 1px;
  background: currentColor;
  box-shadow:
    0 5px 0 currentColor,
    0 10px 0 currentColor;
}

.row-menu-pop {
  position: fixed;
  z-index: 4000;
  min-width: 132px;
  padding: 6px;
  border-radius: 8px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.row-menu-pop button {
  border: 0;
  background: transparent;
  color: inherit;
  text-align: left;
  padding: 7px 10px;
  border-radius: 5px;
  font-size: 13px;
  cursor: pointer;
}

.row-menu-pop button:hover {
  background: var(--color-menu-hover);
}

.row-menu-pop button.danger {
  color: var(--color-danger);
}
</style>
