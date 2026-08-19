<template>
  <section class="panel">
    <div class="head">
      <h3>{{ t('response.title') }}</h3>
      <template v-if="response">
        <span class="pill" :class="statusClass">
          {{ t('response.status') }} {{ response.status }} {{ response.status_text }}
        </span>
        <span class="muted">{{ t('response.time') }} {{ response.duration_ms }} ms</span>
      </template>
    </div>

    <template v-if="response">
      <div class="tabs">
        <button type="button" class="tab" :class="{ active: tab === 'body' }" @click="tab = 'body'">
          Body
        </button>
        <button
          type="button"
          class="tab"
          :class="{ active: tab === 'headers' }"
          @click="tab = 'headers'"
        >
          Headers
        </button>
      </div>

      <pre v-if="tab === 'body'" class="body">{{ prettyBody }}</pre>
      <div v-else class="headers">
        <div v-for="(h, i) in response.headers" :key="i" class="hrow">
          <strong>{{ h.key }}</strong>
          <span>{{ h.value }}</span>
        </div>
      </div>
    </template>
    <p v-else class="muted empty">{{ t('response.empty') }}</p>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../i18n'
import type { SendResponse } from '../types'

const props = defineProps<{
  response: SendResponse | null
}>()

const { t } = useI18n()
const tab = ref<'body' | 'headers'>('body')

const prettyBody = computed(() => {
  const raw = props.response?.body ?? ''
  try {
    return JSON.stringify(JSON.parse(raw), null, 4)
  } catch {
    return raw
  }
})

const statusClass = computed(() => {
  const s = props.response?.status ?? 0
  if (s >= 200 && s < 300) return 'ok'
  if (s >= 400) return 'err'
  if (s === 0) return 'err'
  return 'warn'
})
</script>

<style scoped>
.panel {
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--color-content);
  padding: 10px 12px;
}

.head {
  display: flex;
  align-items: center;
  gap: 12px;
}

h3 {
  margin: 0;
  font-size: 14px;
}

.pill {
  font-family: var(--font-mono);
  font-size: 13px;
  padding: 2px 8px;
  border-radius: 999px;
  background: #e8e8e8;
}

.pill.ok {
  background: rgba(10, 122, 62, 0.12);
  color: var(--method-get);
}

.pill.warn {
  background: rgba(194, 122, 0, 0.12);
  color: var(--method-post);
}

.pill.err {
  background: rgba(196, 43, 28, 0.12);
  color: var(--color-danger);
}

.tabs {
  display: flex;
  gap: 4px;
  margin-top: 8px;
  border-bottom: 1px solid var(--color-border);
}

.tab {
  border: 0;
  background: transparent;
  padding: 8px 12px;
  color: var(--color-text-secondary);
}

.tab.active {
  color: var(--color-primary);
  box-shadow: inset 0 -2px 0 var(--color-primary);
}

.body {
  flex: 1;
  overflow: auto;
  margin: 8px 0 0;
  padding: 10px;
  background: var(--color-content);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  font-family: var(--font-mono);
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-word;
}

.headers {
  flex: 1;
  overflow: auto;
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.hrow {
  display: grid;
  grid-template-columns: 180px 1fr;
  gap: 8px;
  font-size: 13px;
  font-family: var(--font-mono);
  padding: 6px 8px;
  background: var(--color-content);
  border: 1px solid var(--color-border);
  border-radius: 4px;
}

.empty {
  margin-top: 16px;
}
</style>
