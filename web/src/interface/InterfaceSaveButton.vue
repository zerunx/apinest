<script setup lang="ts">
import { useLocale } from '@/composables/useLocale'

defineProps<{
  saving?: boolean
  saved?: boolean
  disabled?: boolean
}>()

const emit = defineEmits<{
  click: []
}>()

const { t } = useLocale()
</script>

<template>
  <button
    type="button"
    class="interface-url-actions__btn interface-url-actions__save"
    :class="{ 'is-busy': saving, 'is-saved': saved }"
    :disabled="saving || disabled"
    @click="emit('click')"
  >
    <span class="interface-url-actions__save-label">
      <template v-if="saved">
        <span class="interface-url-actions__save-check" aria-hidden="true">✓</span>
        {{ t('common.saved') }}
      </template>
      <template v-else>
        {{ t('common.save') }}
      </template>
    </span>
  </button>
</template>
