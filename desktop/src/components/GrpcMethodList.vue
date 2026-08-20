<template>
  <aside class="grpc-method-pane">
    <div class="grpc-method-pane__head">
      <span class="grpc-method-pane__title">{{ protoName || t('grpc.protoList') }}</span>
      <button type="button" class="grpc-method-pane__import" @click="$emit('import')">
        {{ t('grpc.import') }}
      </button>
    </div>
    <div v-if="!services.length" class="grpc-method-pane__empty">
      <p class="muted">{{ t('grpc.importHint') }}</p>
      <button type="button" class="btn btn-primary grpc-method-pane__cta" @click="$emit('import')">
        {{ t('grpc.importProto') }}
      </button>
    </div>
    <div v-else class="grpc-method-pane__tree">
      <div class="grpc-node grpc-node--file">
        <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
          <path
            fill="currentColor"
            d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11z"
          />
        </svg>
        <span>{{ protoName || t('grpc.protoFile') }}</span>
      </div>
      <div v-for="svc in services" :key="svc.full_name" class="grpc-service">
        <button type="button" class="grpc-node grpc-node--service" @click="toggle(svc.full_name)">
          <svg
            class="grpc-node__chevron"
            :class="{ open: isOpen(svc.full_name) }"
            viewBox="0 0 24 24"
            width="12"
            height="12"
            aria-hidden="true"
          >
            <path fill="currentColor" d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6z" />
          </svg>
          <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
            <path
              fill="currentColor"
              d="M12 2L3 7v6c0 5 3.8 9.7 9 11 5.2-1.3 9-6 9-11V7l-9-5zm0 2.2L19 8v5.1c0 3.9-2.8 7.6-7 8.8-4.2-1.2-7-4.9-7-8.8V8l7-3.8z"
            />
          </svg>
          <span>{{ svc.full_name }}</span>
        </button>
        <button
          v-for="m in svc.methods"
          v-show="isOpen(svc.full_name)"
          :key="m.rpc_path"
          type="button"
          class="grpc-node grpc-node--rpc"
          :class="{ active: m.rpc_path === selectedRpc }"
          @click="$emit('select', m.rpc_path)"
        >
          <svg viewBox="0 0 24 24" width="13" height="13" aria-hidden="true">
            <path
              fill="currentColor"
              d="M7 7h3V4l5 5-5 5V11H7V7zm10 10h-3v3l-5-5 5-5v3h3v4z"
            />
          </svg>
          <span>{{ m.name }}</span>
        </button>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from '../i18n'
import type { GrpcService } from '../utils/grpc'

const props = defineProps<{
  protoName: string
  services: GrpcService[]
  selectedRpc: string
}>()

defineEmits<{
  select: [rpcPath: string]
  import: []
}>()

const { t } = useI18n()
const open = ref<Record<string, boolean>>({})

function isOpen(key: string) {
  return open.value[key] !== false
}

function toggle(key: string) {
  open.value[key] = !isOpen(key)
}

watch(
  () => props.services.map((s) => s.full_name).join(','),
  () => {
    const next: Record<string, boolean> = {}
    for (const svc of props.services) next[svc.full_name] = true
    open.value = next
  },
  { immediate: true },
)
</script>

<style scoped>
.grpc-method-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  height: 100%;
  border: 1px solid var(--color-interface-bar-border);
  border-radius: 4px;
  background: var(--color-workspace-content);
  overflow: hidden;
}

.grpc-method-pane__head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  padding: 8px 10px;
  border-bottom: 1px solid var(--color-interface-panel-border);
}

.grpc-method-pane__title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.grpc-method-pane__import {
  border: 0;
  background: transparent;
  color: var(--color-interface-accent);
  font-size: 12px;
  cursor: pointer;
  padding: 2px 0;
  flex-shrink: 0;
}

.grpc-method-pane__import:hover {
  text-decoration: underline;
}

.grpc-method-pane__empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 16px;
  text-align: center;
}

.grpc-method-pane__empty p {
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
}

.grpc-method-pane__cta {
  min-height: 32px;
  padding: 0 12px;
  font-size: 13px;
}

.grpc-method-pane__tree {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 6px 0 10px;
}

.grpc-service {
  display: flex;
  flex-direction: column;
}

.grpc-node {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--color-interface-field-text);
  font: inherit;
  font-size: 13px;
  text-align: left;
  padding: 5px 10px;
  cursor: pointer;
}

.grpc-node span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.grpc-node svg {
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.grpc-node--file {
  cursor: default;
  color: var(--color-text-secondary);
  font-weight: 600;
  padding-left: 10px;
}

.grpc-node--service {
  padding-left: 8px;
}

.grpc-node--rpc {
  padding-left: 36px;
  color: var(--color-interface-field-text);
}

.grpc-node--rpc svg {
  color: var(--method-get);
}

.grpc-node--rpc:hover,
.grpc-node--service:hover {
  background: var(--color-hover);
}

.grpc-node--rpc.active {
  background: color-mix(in srgb, var(--color-interface-accent) 16%, transparent);
  color: var(--color-interface-accent);
}

.grpc-node__chevron {
  transition: transform 0.15s ease;
  transform: rotate(-90deg);
}

.grpc-node__chevron.open {
  transform: rotate(0deg);
}
</style>
