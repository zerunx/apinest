<template>
  <div
    class="param-table__resize-overlay"
    :style="{ gridTemplateColumns: columns }"
    aria-hidden="true"
  >
    <template v-for="(handle, idx) in handles" :key="idx">
      <span v-if="!handle"></span>
      <span v-else class="param-table__resize-slot">
        <i
          class="param-table__resize-handle"
          :class="{ 'param-table__resize-handle--active': active === handle }"
          @mousedown="onStart(handle, $event)"
        />
      </span>
    </template>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'

defineProps<{
  columns: string
  handles: (string | false)[]
}>()

const emit = defineEmits<{
  resize: [column: string, event: MouseEvent]
}>()

const active = ref<string | null>(null)

function onStart(column: string, e: MouseEvent) {
  active.value = column
  emit('resize', column, e)
  window.addEventListener('mouseup', onUp)
}

function onUp() {
  active.value = null
  window.removeEventListener('mouseup', onUp)
}

onBeforeUnmount(onUp)
</script>

<style scoped>
.param-table__resize-overlay {
  display: grid;
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: var(--param-table-head-height, 34px);
  z-index: 3;
  pointer-events: none;
}

.param-table__resize-slot {
  position: relative;
  pointer-events: none;
}

.param-table__resize-handle {
  position: absolute;
  top: 0;
  right: -4px;
  width: 8px;
  height: 100%;
  cursor: col-resize;
  pointer-events: auto;
  background: transparent;
}

.param-table__resize-handle::after {
  content: '';
  position: absolute;
  top: 10px;
  bottom: 10px;
  left: 50%;
  width: 1px;
  transform: translateX(-50%);
  background: var(--color-interface-bar-border);
  opacity: 0.85;
  border-radius: 1px;
  pointer-events: none;
  transition:
    width 0.12s ease,
    background-color 0.12s ease,
    opacity 0.12s ease,
    top 0.12s ease,
    bottom 0.12s ease;
}

.param-table__resize-handle:hover::after,
.param-table__resize-handle--active::after {
  width: 2px;
  top: 6px;
  bottom: 6px;
  background: var(--color-primary);
  opacity: 1;
}
</style>
