<template>
  <div class="app-root" :data-locale="locale">
    <TitleBar />
    <div class="app-body">
      <router-view />
    </div>
    <LoginDialog />
    <AppToast />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import TitleBar from './components/TitleBar.vue'
import LoginDialog from './components/LoginDialog.vue'
import AppToast from './components/AppToast.vue'
import { useI18n } from './i18n'
import { initTheme } from './composables/useTheme'
import { useAppUi } from './composables/useAppUi'
import { usePreventReload } from './composables/usePreventReload'
import { useEscapeMinimize } from './composables/useEscapeMinimize'

const { locale } = useI18n()
const { refreshSession } = useAppUi()

usePreventReload()
useEscapeMinimize()

onMounted(async () => {
  await initTheme()
  await refreshSession()
})
</script>

<style scoped>
.app-root {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.app-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
</style>
