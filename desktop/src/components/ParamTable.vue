<template>
  <div class="interface-param-table">
    <div class="interface-param-table__head">
      <span>{{ t('create.param.name') }}</span>
      <span>{{ t('create.param.type') }}</span>
      <span>{{ t('create.param.required') }}</span>
      <span>{{ t('create.param.desc') }}</span>
      <span>{{ t('create.param.example') }}</span>
      <span></span>
    </div>
    <div v-for="(row, idx) in model" :key="idx" class="interface-param-table__row">
      <input v-model="row.name" class="native-input" @input="ensureTrailing" />
      <MenuSelect
        v-model="row.type"
        compact
        :options="typeOptions"
      />
      <label class="req">
        <input v-model="row.required" type="checkbox" />
      </label>
      <input v-model="row.description" class="native-input" />
      <input v-model="row.example" class="native-input" />
      <button type="button" class="del" :disabled="model.length <= 1" @click="remove(idx)">×</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '../i18n'
import MenuSelect from './MenuSelect.vue'

export type ParamRow = {
  name: string
  type: string
  required: boolean
  description: string
  example: string
}

const model = defineModel<ParamRow[]>({ required: true })
const { t } = useI18n()
const types = ['string', 'number', 'boolean', 'object', 'array', 'file']
const typeOptions = types.map((tp) => ({ value: tp, label: tp }))

function emptyRow(): ParamRow {
  return { name: '', type: 'string', required: false, description: '', example: '' }
}

function ensureTrailing() {
  const last = model.value[model.value.length - 1]
  if (last && last.name.trim()) model.value.push(emptyRow())
}

function remove(idx: number) {
  if (model.value.length <= 1) return
  model.value.splice(idx, 1)
  if (!model.value.length) model.value.push(emptyRow())
}
</script>

<style scoped>
.interface-param-table__head,
.interface-param-table__row {
  display: grid;
  grid-template-columns: 1.2fr 0.9fr 56px 1.2fr 1fr 28px;
  gap: 6px;
  align-items: center;
}

.interface-param-table__head {
  font-size: 12px;
  color: var(--color-text-secondary);
  padding: 4px 0 8px;
}

.interface-param-table__row {
  margin-bottom: 6px;
}

.req {
  display: grid;
  place-items: center;
}

.del {
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 16px;
  cursor: pointer;
}

.del:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
</style>
