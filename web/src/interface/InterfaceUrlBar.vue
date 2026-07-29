<script setup lang="ts">
import type { HttpMethod, HttpProtocol } from '@/types/workspace'

defineProps<{
  protocol: HttpProtocol
  method: HttpMethod
  url: string
  readonly?: boolean
}>()

const emit = defineEmits<{
  'update:protocol': [value: HttpProtocol]
  'update:method': [value: HttpMethod]
  'update:url': [value: string]
}>()

const protocolOptions: HttpProtocol[] = ['HTTP', 'HTTPS']
const methodOptions: HttpMethod[] = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS']
</script>

<template>
  <div class="interface-url-bar">
    <div class="interface-url-bar__selects">
      <el-select
        :model-value="protocol"
        class="interface-url-bar__protocol"
        size="default"
        :disabled="readonly"
        popper-class="app-action-dropdown"
        @update:model-value="emit('update:protocol', $event)"
      >
        <el-option v-for="item in protocolOptions" :key="item" :label="item" :value="item" />
      </el-select>
      <el-select
        :model-value="method"
        class="interface-url-bar__method"
        size="default"
        :disabled="readonly"
        popper-class="app-action-dropdown"
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
