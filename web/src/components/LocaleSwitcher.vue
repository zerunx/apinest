<script setup lang="ts">
import { computed } from 'vue'
import { Check } from '@element-plus/icons-vue'
import { useLocale, type LocaleCode } from '@/composables/useLocale'

const { locale, setLocale } = useLocale()

const currentLabel = computed(() => (locale.value === 'zh-CN' ? '中' : 'EN'))

function handleSelect(code: LocaleCode) {
  setLocale(code)
}
</script>

<template>
  <el-popover
    placement="bottom-end"
    :width="120"
    trigger="click"
    popper-class="app-action-dropdown locale-switcher-popover"
  >
    <template #reference>
      <button type="button" class="locale-switcher" aria-label="Language">
        <span class="locale-switcher__current">{{ currentLabel }}</span>
      </button>
    </template>

    <ul class="locale-switcher__menu">
      <li
        class="locale-switcher__item"
        :class="{ 'locale-switcher__item--active': locale === 'zh-CN' }"
        @click="handleSelect('zh-CN')"
      >
        <span>中文</span>
        <el-icon v-if="locale === 'zh-CN'" :size="14"><Check /></el-icon>
      </li>
      <li
        class="locale-switcher__item"
        :class="{ 'locale-switcher__item--active': locale === 'en-US' }"
        @click="handleSelect('en-US')"
      >
        <span>English</span>
        <el-icon v-if="locale === 'en-US'" :size="14"><Check /></el-icon>
      </li>
    </ul>
  </el-popover>
</template>

<style scoped>
.locale-switcher {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 36px;
  height: 36px;
  padding: 0 10px;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.locale-switcher:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.locale-switcher__current {
  font-size: 14px;
  font-weight: 700;
}

.locale-switcher__menu {
  list-style: none;
  margin: 0;
  padding: 4px;
}

.locale-switcher__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  font-size: 14px;
  color: var(--color-text);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.locale-switcher__item:hover {
  background: var(--color-hover);
}

.locale-switcher__item--active {
  color: var(--color-primary-light);
  font-weight: 500;
}
</style>
