<script setup lang="ts">
import { computed } from 'vue'
import { useAuth } from '@/composables/useAuth'
import { useLocale } from '@/composables/useLocale'

const { user } = useAuth()
const { t } = useLocale()

const welcomeText = computed(() =>
  t('home.welcome', { name: user.value?.name || user.value?.account || '' }),
)
</script>

<template>
  <div class="home-view">
    <el-card shadow="never" class="home-view__card">
      <h1>{{ welcomeText }}</h1>
      <p>{{ t('home.desc') }}</p>
    </el-card>
  </div>
</template>

<style scoped>
.home-view {
  max-width: 960px;
  margin: 0 auto;
  padding: 24px;
}

.home-view__card {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-surface);
}

.home-view__card h1 {
  margin: 0 0 12px;
  font-size: 24px;
}

.home-view__card p {
  margin: 0;
  color: var(--color-text-secondary);
  line-height: 1.6;
}
</style>
