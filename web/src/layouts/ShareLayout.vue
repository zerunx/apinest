<template>
  <div class="share-layout">
    <div class="share-layout__top">
      <div class="share-layout__brand">
        <div class="share-layout__mark">
          <img class="share-layout__logo app-logo" src="/nest.png" alt="" />
        </div>
        <div>
          <h1 class="share-layout__name">ApiNest</h1>
          <p class="share-layout__slogan">设计 · 调试 · 协作 · 交付</p>
        </div>
      </div>

      <div v-if="sharePageTitle" class="share-layout__title-box">
        <h2 class="share-layout__project">{{ sharePageTitle }}</h2>
        <p v-if="sharePageSubtitle" class="share-layout__share-name">{{ sharePageSubtitle }}</p>
      </div>

      <button
        type="button"
        class="share-layout__theme"
        :title="theme === 'dark' ? t('topbar.themeDark') : t('topbar.themeLight')"
        @click="toggleTheme"
      >
        <el-icon :size="18">
          <Sunny v-if="theme === 'dark'" />
          <Moon v-else />
        </el-icon>
      </button>
    </div>
    <div class="share-layout__content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Moon, Sunny } from '@element-plus/icons-vue'
import { useLocale } from '@/composables/useLocale'
import { useSharePageMeta } from '@/composables/useSharePageMeta'
import { useTheme } from '@/composables/useTheme'

const { t } = useLocale()
const { theme, toggleTheme } = useTheme()
const { sharePageTitle, sharePageSubtitle } = useSharePageMeta()
</script>

<style scoped>
.share-layout {
  height: 100vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background:
    radial-gradient(circle at 12% 18%, rgba(37, 99, 235, 0.14), transparent 36%),
    radial-gradient(circle at 88% 78%, rgba(15, 23, 42, 0.06), transparent 30%),
    linear-gradient(180deg, var(--color-bg) 0%, var(--color-workspace-content) 100%);
  color: var(--color-text);
}

.share-layout__top {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 16px;
  padding: 16px 28px 8px;
  flex-shrink: 0;
}

.share-layout__brand {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-shrink: 0;
  justify-self: start;
}

.share-layout__mark {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: var(--color-surface);
  box-shadow: var(--shadow-card);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px 6px 6px;
}

.share-layout__logo {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.share-layout__name {
  margin: 0;
  font-size: 20px;
  font-weight: 800;
  color: var(--color-text);
}

.share-layout__slogan {
  margin: 2px 0 0;
  color: var(--color-text-secondary);
  font-size: 12px;
  letter-spacing: 0.18em;
}

.share-layout__title-box {
  justify-self: center;
  min-width: 0;
  max-width: min(440px, 48vw);
  padding: 8px 18px;
  border: 1px solid rgba(255, 255, 255, 0.45);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(16px) saturate(1.4);
  -webkit-backdrop-filter: blur(16px) saturate(1.4);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.5) inset,
    0 8px 24px rgba(15, 23, 42, 0.06);
  text-align: left;
}

:root[data-theme='dark'] .share-layout__title-box {
  border-color: rgba(255, 255, 255, 0.22);
  background: rgba(255, 255, 255, 0.16);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.14) inset,
    0 8px 24px rgba(0, 0, 0, 0.22);
}

.share-layout__project {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text);
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.share-layout__share-name {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.share-layout__theme {
  width: 36px;
  height: 36px;
  justify-self: end;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface);
  color: var(--color-text-secondary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.share-layout__theme:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.share-layout__content {
  flex: 1;
  min-height: 0;
  display: flex;
  justify-content: center;
  padding: 8px 28px 20px;
  overflow: hidden;
}

@media (max-width: 720px) {
  .share-layout__top {
    grid-template-columns: 1fr auto;
    grid-template-areas:
      'brand theme'
      'title title';
  }

  .share-layout__brand {
    grid-area: brand;
  }

  .share-layout__theme {
    grid-area: theme;
  }

  .share-layout__title-box {
    grid-area: title;
    max-width: none;
    width: 100%;
  }

  .share-layout__slogan {
    display: none;
  }
}
</style>
