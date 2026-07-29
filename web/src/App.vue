<script setup lang="ts">
import { useLocale } from '@/composables/useLocale'

const { elementLocale } = useLocale()
</script>

<template>
  <el-config-provider :locale="elementLocale">
    <router-view />
  </el-config-provider>
</template>
