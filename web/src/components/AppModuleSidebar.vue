<script setup lang="ts">
import { Connection, FolderOpened, Lightning, Operation } from '@element-plus/icons-vue'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { AppModule } from '@/types/workspace'

const { t } = useLocale()
const { activeModule, setModule } = useWorkspaceContext()

const modules: { key: AppModule; icon: typeof Connection; labelKey: string }[] = [
  { key: 'api', icon: Connection, labelKey: 'workspace.modules.api' },
  { key: 'quick-test', icon: Lightning, labelKey: 'workspace.modules.quickTest' },
  { key: 'environment', icon: Operation, labelKey: 'workspace.modules.environment' },
  { key: 'project', icon: FolderOpened, labelKey: 'workspace.modules.project' },
]
</script>

<template>
  <aside class="module-sidebar">
    <nav class="module-sidebar__nav">
      <button
        v-for="item in modules"
        :key="item.key"
        type="button"
        class="module-sidebar__item"
        :class="{ 'module-sidebar__item--active': activeModule === item.key }"
        @click="setModule(item.key)"
      >
        <el-icon :size="20">
          <component :is="item.icon" />
        </el-icon>
        <span class="module-sidebar__label">{{ t(item.labelKey) }}</span>
      </button>
    </nav>
  </aside>
</template>

<style scoped>
.module-sidebar {
  width: 148px;
  height: 100%;
  min-height: 0;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border);
  background: var(--color-sidebar);
  display: flex;
  flex-direction: column;
}

.module-sidebar__nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 10px;
}

.module-sidebar__item {
  width: 100%;
  min-height: 44px;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.module-sidebar__item:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.module-sidebar__item--active {
  background: var(--color-active);
  color: var(--color-primary-light);
}

.module-sidebar__item--active .module-sidebar__label {
  font-weight: 600;
}

.module-sidebar__label {
  font-size: 14px;
  line-height: 1.3;
  text-align: left;
}
</style>
