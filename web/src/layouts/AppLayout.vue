<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppContextPanel from '@/components/AppContextPanel.vue'
import AppModuleSidebar from '@/components/AppModuleSidebar.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import CreateProjectDialog from '@/components/CreateProjectDialog.vue'
import CreateWorkspaceDialog from '@/components/CreateWorkspaceDialog.vue'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'

const route = useRoute()
const router = useRouter()
const { bootstrap } = useWorkspaceContext()
const showOverlay = computed(() => route.meta.overlay === true)

function closeOverlay() {
  router.push('/home')
}

onMounted(() => {
  bootstrap()
})
</script>

<template>
  <div class="app-layout">
    <AppTopbar />
    <div class="app-layout__body">
      <div class="app-layout__nav" :class="{ 'app-layout__nav--dimmed': showOverlay }">
        <AppModuleSidebar />
        <AppContextPanel />
      </div>

      <main class="app-layout__main">
        <router-view />
      </main>

      <Transition name="app-layout__backdrop">
        <div
          v-if="showOverlay"
          class="app-layout__backdrop"
          aria-hidden="true"
          @click="closeOverlay"
        />
      </Transition>

      <router-view v-slot="{ Component }" name="overlay">
        <Transition name="app-layout__overlay">
          <component :is="Component" v-if="Component" class="app-layout__overlay" />
        </Transition>
      </router-view>
    </div>

    <CreateWorkspaceDialog />
    <CreateProjectDialog />
  </div>
</template>

<style scoped>
.app-layout {
  height: 100vh;
  max-height: 100vh;
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-layout__body {
  position: relative;
  flex: 1;
  display: flex;
  min-height: 0;
  height: calc(100vh - 56px);
  overflow: hidden;
}

.app-layout__nav {
  display: flex;
  flex-shrink: 0;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  transition: opacity 0.2s ease;
}

.app-layout__nav--dimmed {
  opacity: 0.35;
  pointer-events: none;
}

.app-layout__main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--color-workspace-content);
}

.app-layout__backdrop {
  position: absolute;
  inset: 0;
  z-index: 110;
  cursor: pointer;
}

.app-layout__overlay {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  width: min(1120px, 94vw);
  z-index: 120;
  background: var(--color-bg);
  border-right: 1px solid var(--color-border);
  box-shadow: 8px 0 32px rgba(0, 0, 0, 0.18);
  overflow: hidden;
}
</style>
