<script setup lang="ts">
import { ArrowDown, Check, Folder, Plus, Search } from '@element-plus/icons-vue'
import { computed, ref } from 'vue'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'

const { t } = useLocale()
const {
  workspaces,
  activeWorkspace,
  projects,
  activeProject,
  selectWorkspace,
  selectProject,
  openCreateWorkspace,
  openCreateProject,
} = useWorkspaceContext()

const workspaceOpen = ref(false)
const projectOpen = ref(false)
const workspaceSearch = ref('')
const projectSearch = ref('')

const projectPlaceholder = computed(() => t('workspace.selectProject'))

const filteredWorkspaces = computed(() => {
  const q = workspaceSearch.value.trim().toLowerCase()
  if (!q) return workspaces.value
  return workspaces.value.filter((item) => item.name.toLowerCase().includes(q))
})

const filteredProjects = computed(() => {
  const q = projectSearch.value.trim().toLowerCase()
  if (!q) return projects.value
  return projects.value.filter((item) => item.name.toLowerCase().includes(q))
})

function handleWorkspaceSelect(id: number) {
  selectWorkspace(id)
  workspaceOpen.value = false
}

function handleProjectSelect(id: number | null) {
  selectProject(id)
  projectOpen.value = false
}

function handleCreateWorkspace() {
  workspaceOpen.value = false
  openCreateWorkspace()
}

function handleCreateProject() {
  projectOpen.value = false
  openCreateProject()
}
</script>

<template>
  <div class="ws-project-selector">
    <el-popover
      v-model:visible="workspaceOpen"
      placement="bottom-start"
      :width="300"
      trigger="click"
      popper-class="ws-project-popover"
    >
      <template #reference>
        <button type="button" class="ws-project-selector__trigger">
          <span class="ws-project-selector__label">{{ t('workspace.workspace') }}</span>
          <span class="ws-project-selector__value">{{ activeWorkspace?.name }}</span>
          <el-icon :size="14" class="ws-project-selector__chevron">
            <ArrowDown />
          </el-icon>
        </button>
      </template>

      <div class="ws-project-popover__header">
        <el-icon :size="16"><Search /></el-icon>
        <input
          v-model="workspaceSearch"
          type="text"
          class="ws-project-popover__search"
          :placeholder="t('workspace.searchWorkspace')"
        />
      </div>
      <ul class="ws-project-popover__list">
        <li
          v-for="item in filteredWorkspaces"
          :key="item.id"
          class="ws-project-popover__item"
          :class="{ 'ws-project-popover__item--active': item.id === activeWorkspace?.id }"
          @click="handleWorkspaceSelect(item.id)"
        >
          <span class="ws-project-popover__item-name">{{ item.name }}</span>
          <el-icon v-if="item.id === activeWorkspace?.id" :size="16" class="ws-project-popover__check">
            <Check />
          </el-icon>
        </li>
      </ul>
      <div class="ws-project-popover__footer">
        <button type="button" class="ws-project-popover__create" @click="handleCreateWorkspace">
          <el-icon :size="16"><Plus /></el-icon>
          <span>{{ t('workspace.createWorkspace') }}</span>
        </button>
      </div>
    </el-popover>

    <span class="ws-project-selector__sep">/</span>

    <el-popover
      v-model:visible="projectOpen"
      placement="bottom-start"
      :width="300"
      trigger="click"
      popper-class="ws-project-popover"
    >
      <template #reference>
        <button type="button" class="ws-project-selector__trigger">
          <span class="ws-project-selector__label">{{ t('workspace.project') }}</span>
          <span
            class="ws-project-selector__value"
            :class="{ 'ws-project-selector__value--placeholder': !activeProject }"
          >
            {{ activeProject?.name || projectPlaceholder }}
          </span>
          <el-icon :size="14" class="ws-project-selector__chevron">
            <ArrowDown />
          </el-icon>
        </button>
      </template>

      <div class="ws-project-popover__header">
        <el-icon :size="16"><Search /></el-icon>
        <input
          v-model="projectSearch"
          type="text"
          class="ws-project-popover__search"
          :placeholder="t('workspace.searchProject')"
        />
      </div>
      <ul class="ws-project-popover__list">
        <li
          class="ws-project-popover__item"
          :class="{ 'ws-project-popover__item--active': !activeProject }"
          @click="handleProjectSelect(null)"
        >
          <span class="ws-project-popover__item-name ws-project-popover__item-name--muted">
            {{ t('workspace.allProjects') }}
          </span>
          <el-icon v-if="!activeProject" :size="16" class="ws-project-popover__check">
            <Check />
          </el-icon>
        </li>
        <li
          v-for="item in filteredProjects"
          :key="item.id"
          class="ws-project-popover__item"
          :class="{ 'ws-project-popover__item--active': item.id === activeProject?.id }"
          @click="handleProjectSelect(item.id)"
        >
          <el-icon :size="16" class="ws-project-popover__folder"><Folder /></el-icon>
          <span class="ws-project-popover__item-name">{{ item.name }}</span>
          <el-icon v-if="item.id === activeProject?.id" :size="16" class="ws-project-popover__check">
            <Check />
          </el-icon>
        </li>
      </ul>
      <div class="ws-project-popover__footer">
        <button type="button" class="ws-project-popover__create" @click="handleCreateProject">
          <span>{{ t('workspace.createProject') }}</span>
        </button>
      </div>
    </el-popover>
  </div>
</template>

<style scoped>
.ws-project-selector {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  margin-left: 24px;
}

.ws-project-selector__trigger {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  max-width: 260px;
  padding: 6px 10px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.ws-project-selector__trigger:hover {
  background: var(--color-hover);
}

.ws-project-selector__label {
  font-size: 13px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.ws-project-selector__value {
  font-size: 15px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ws-project-selector__value--placeholder {
  color: var(--color-text-secondary);
  font-weight: 500;
}

.ws-project-selector__chevron {
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.ws-project-selector__sep {
  color: var(--color-text-secondary);
  font-size: 14px;
  user-select: none;
}
</style>
