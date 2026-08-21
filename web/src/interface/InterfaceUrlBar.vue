<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import type { HttpMethod, HttpProtocol } from '@/types/workspace'
import {
  applyProtocolScheme,
  isProtocolOnlySelect,
  methodsForProtocol,
  protocolLabel,
  switchableProtocols,
} from '@/utils/http-method'

const props = defineProps<{
  protocol: HttpProtocol
  method: HttpMethod
  url: string
  readonly?: boolean
  createMode?: boolean
}>()

const emit = defineEmits<{
  'update:protocol': [value: HttpProtocol]
  'update:method': [value: HttpMethod]
  'update:url': [value: string]
}>()

const protocolOptions = computed(() => switchableProtocols(props.protocol, !!props.createMode))
const isProtocolOnly = computed(() => isProtocolOnlySelect(props.protocol))
const methodOptions = computed(() => methodsForProtocol(props.protocol))

const protocolOpen = ref(false)
const methodOpen = ref(false)

function onProtocolChange(next: HttpProtocol) {
  emit('update:protocol', next)
  const allowed = methodsForProtocol(next)
  if (!allowed.includes(props.method)) {
    emit('update:method', (allowed[0] ?? 'GET') as HttpMethod)
  }
  const nextUrl = applyProtocolScheme(props.url, next)
  if (nextUrl !== props.url) {
    emit('update:url', nextUrl)
  }
}

function onMenuVisible(open: boolean, which: 'protocol' | 'method') {
  if (which === 'protocol') protocolOpen.value = open
  else methodOpen.value = open
  if (!open) {
    void nextTick(() => {
      const el = document.activeElement
      if (el instanceof HTMLElement) el.blur()
    })
  }
}
</script>

<template>
  <div class="interface-url-bar">
    <div
      class="interface-url-bar__selects"
      :class="{ 'interface-url-bar__selects--single': isProtocolOnly }"
    >
      <el-select
        :model-value="protocol"
        class="interface-url-bar__protocol"
        :class="{ 'is-dropdown-visible': protocolOpen }"
        size="default"
        :disabled="readonly"
        popper-class="app-action-dropdown"
        @visible-change="onMenuVisible($event, 'protocol')"
        @update:model-value="onProtocolChange"
      >
        <el-option
          v-for="item in protocolOptions"
          :key="item"
          :label="protocolLabel(item)"
          :value="item"
        />
      </el-select>
      <el-select
        v-if="!isProtocolOnly"
        :model-value="method"
        class="interface-url-bar__method"
        :class="{ 'is-dropdown-visible': methodOpen }"
        size="default"
        :disabled="readonly"
        popper-class="app-action-dropdown"
        @visible-change="onMenuVisible($event, 'method')"
        @update:model-value="emit('update:method', $event)"
      >
        <el-option v-for="item in methodOptions" :key="item" :label="item" :value="item" />
      </el-select>
    </div>
    <el-input
      :model-value="url"
      class="interface-url-bar__url"
      :readonly="readonly"
      @update:model-value="emit('update:url', $event)"
    />
  </div>
</template>
