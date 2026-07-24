<template>
  <div
    class="param-table__resize-overlay"
    :style="{ gridTemplateColumns: columns }"
    aria-hidden="true"
  >
    <template v-for="(handle, idx) in handles" :key="idx">
      <span v-if="!handle"></span>
      <span v-else class="param-table__resize-slot">
        <i class="param-table__resize-handle" @mousedown="emit('resize', handle, $event)" />
      </span>
    </template>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  columns: string
  handles: (string | false)[]
}>()

const emit = defineEmits<{
  resize: [column: string, event: MouseEvent]
}>()
</script>

<style scoped>
.param-table__resize-overlay {
  display: grid;
  position: absolute;
  inset: 0;
  z-index: 2;
  pointer-events: none;
}

.param-table__resize-slot {
  position: relative;
  pointer-events: none;
}

.param-table__resize-handle {
  position: absolute;
  top: 0;
  right: -3px;
  width: 6px;
  height: 100%;
  cursor: col-resize;
  pointer-events: auto;
}

.param-table__resize-handle:hover {
  background: color-mix(in srgb, var(--color-interface-accent) 35%, transparent);
}
</style>
