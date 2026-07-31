<template>
  <section class="panel">
    <div class="url-row">
      <div class="url-bar">
        <MethodPicker :model-value="method" @update:model-value="onMethod" />
        <input class="url" :value="url" :placeholder="t('method.url')" @input="onUrl" />
      </div>
      <button
        class="btn"
        type="button"
        :disabled="!dirty"
        :title="t('request.save')"
        @click="$emit('save')"
      >
        {{ t('request.save') }}
      </button>
      <button class="btn btn-primary" type="button" :disabled="sending" @click="$emit('send')">
        {{ sending ? t('request.sending') : t('request.send') }}
      </button>
    </div>

    <div class="tabs">
      <button
        type="button"
        class="tab"
        :class="{ active: tab === 'headers' }"
        @click="tab = 'headers'"
      >
        {{ t('request.headers') }}
      </button>
      <button type="button" class="tab" :class="{ active: tab === 'body' }" @click="tab = 'body'">
        {{ t('request.body') }}
      </button>
    </div>

    <div v-if="tab === 'headers'" class="headers">
      <div v-for="(h, idx) in headers" :key="idx" class="header-row">
        <input
          type="checkbox"
          :checked="h.enabled !== false"
          @change="toggle(idx, ($event.target as HTMLInputElement).checked)"
        />
        <input
          class="input"
          :value="h.key"
          :placeholder="t('common.key')"
          @input="patch(idx, 'key', ($event.target as HTMLInputElement).value)"
        />
        <input
          class="input"
          :value="h.value"
          :placeholder="t('common.value')"
          @input="patch(idx, 'value', ($event.target as HTMLInputElement).value)"
        />
        <button class="btn" type="button" @click="remove(idx)">×</button>
      </div>
      <button class="btn" type="button" @click="add">{{ t('common.add') }}</button>
    </div>

    <textarea
      v-else
      class="body"
      :value="body"
      spellcheck="false"
      @input="onBody"
    />
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../i18n'
import MethodPicker from './MethodPicker.vue'
import type { HeaderPair } from '../types'

const props = defineProps<{
  name: string
  method: string
  url: string
  headers: HeaderPair[]
  body: string
  sending: boolean
  dirty: boolean
}>()

const emit = defineEmits<{
  'update:name': [string]
  'update:method': [string]
  'update:url': [string]
  'update:headers': [HeaderPair[]]
  'update:body': [string]
  send: []
  save: []
}>()

const { t } = useI18n()
const tab = ref<'headers' | 'body'>('headers')

function onMethod(m: string) {
  emit('update:method', m)
}
function onUrl(e: Event) {
  emit('update:url', (e.target as HTMLInputElement).value)
}
function onBody(e: Event) {
  emit('update:body', (e.target as HTMLTextAreaElement).value)
}

function patch(idx: number, key: 'key' | 'value', value: string) {
  const next = props.headers.map((h, i) => (i === idx ? { ...h, [key]: value } : h))
  emit('update:headers', next)
}

function toggle(idx: number, enabled: boolean) {
  const next = props.headers.map((h, i) => (i === idx ? { ...h, enabled } : h))
  emit('update:headers', next)
}

function remove(idx: number) {
  emit(
    'update:headers',
    props.headers.filter((_, i) => i !== idx),
  )
}

function add() {
  emit('update:headers', [...props.headers, { key: '', value: '', enabled: true }])
}
</script>

<style scoped>
.panel {
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-content);
  padding: 10px 12px;
  gap: 8px;
  font-size: 13px;
}

.url-row {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 8px;
  align-items: stretch;
  min-height: 34px;
}

.url-bar {
  display: flex;
  align-items: stretch;
  min-width: 0;
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  background: var(--color-url-bar);
  /* keep visible so method dropdown is not clipped */
  overflow: visible;
}

.url-bar .url {
  flex: 1;
  min-width: 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  padding: 0 10px 0 12px;
  font-size: 15px;
  outline: none;
}

.url-bar:focus-within {
  border-color: color-mix(in srgb, var(--color-primary) 55%, var(--color-border));
}

.url-row .btn {
  font-size: 13px;
}

.tabs {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--color-border);
}

.tab {
  border: 0;
  background: transparent;
  padding: 6px 10px;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.tab.active {
  color: var(--color-primary);
  box-shadow: inset 0 -2px 0 var(--color-primary);
}

.headers {
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-top: 6px;
}

.header-row {
  display: grid;
  grid-template-columns: 20px 1fr 1.4fr 36px;
  gap: 6px;
  align-items: center;
}

.header-row .input,
.headers .btn {
  font-size: 13px;
}

.body {
  flex: 1;
  min-height: 120px;
  resize: none;
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  padding: 10px;
  font-family: var(--font-mono);
  font-size: 13px;
  background: var(--color-content);
}
</style>
