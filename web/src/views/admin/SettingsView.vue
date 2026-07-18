<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Close, FolderOpened, Plus, User } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { fetchAdminUsers, fetchAdminWorkspaces, transferWorkspace } from '@/api/admin'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { AdminUserItem, AdminWorkspaceItem } from '@/types/admin'

type MenuKey = 'users' | 'workspaces'

const router = useRouter()
const { t } = useLocale()
const { openCreateWorkspace } = useWorkspaceContext()

const activeMenu = ref<MenuKey>('users')
const users = ref<AdminUserItem[]>([])
const workspaces = ref<AdminWorkspaceItem[]>([])
const loadingUsers = ref(false)
const loadingWorkspaces = ref(false)
const transferringId = ref<number | null>(null)

const menus = computed(() => [
  { key: 'users' as MenuKey, label: t('admin.users'), icon: User },
  { key: 'workspaces' as MenuKey, label: t('admin.workspaces'), icon: FolderOpened },
])

async function loadUsers() {
  loadingUsers.value = true
  try {
    users.value = await fetchAdminUsers()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('admin.loadUsersFailed'))
  } finally {
    loadingUsers.value = false
  }
}

async function loadWorkspaces() {
  loadingWorkspaces.value = true
  try {
    workspaces.value = await fetchAdminWorkspaces()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('admin.loadWorkspacesFailed'))
  } finally {
    loadingWorkspaces.value = false
  }
}

async function handleTransfer(workspace: AdminWorkspaceItem, ownerId: number) {
  transferringId.value = workspace.id
  try {
    await transferWorkspace(workspace.id, ownerId)
    ElMessage.success(t('admin.transferSuccess'))
    await loadWorkspaces()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('admin.transferFailed'))
  } finally {
    transferringId.value = null
  }
}

function switchMenu(key: MenuKey) {
  activeMenu.value = key
}

function closeSettings() {
  router.push('/home')
}

function handleCreateWorkspaceAdmin() {
  closeSettings()
  openCreateWorkspace()
}

onMounted(async () => {
  await Promise.all([loadUsers(), loadWorkspaces()])
})
</script>

<template>
  <div class="admin-settings">
    <header class="admin-settings__header">
      <div class="admin-settings__header-info">
        <h1>{{ t('admin.title') }}</h1>
        <p>{{ t('admin.desc') }}</p>
      </div>
      <button type="button" class="admin-settings__close" @click="closeSettings">
        <el-icon :size="18"><Close /></el-icon>
      </button>
    </header>

    <div class="admin-settings__body">
      <aside class="admin-settings__sidebar">
        <nav class="admin-settings__menu">
          <button
            v-for="item in menus"
            :key="item.key"
            type="button"
            class="admin-settings__menu-item"
            :class="{ 'admin-settings__menu-item--active': activeMenu === item.key }"
            @click="switchMenu(item.key)"
          >
            <el-icon :size="18"><component :is="item.icon" /></el-icon>
            <span>{{ item.label }}</span>
          </button>
        </nav>
      </aside>

      <section class="admin-settings__content">
        <el-card v-if="activeMenu === 'users'" shadow="never" class="admin-settings__panel">
          <el-table v-loading="loadingUsers" :data="users" stripe>
            <el-table-column prop="id" :label="t('admin.columns.id')" width="80" />
            <el-table-column prop="name" :label="t('admin.columns.name')" min-width="120" />
            <el-table-column prop="account" :label="t('admin.columns.account')" min-width="120" />
            <el-table-column prop="email" :label="t('admin.columns.email')" min-width="180" />
            <el-table-column :label="t('admin.columns.role')" width="100">
              <template #default="{ row }">
                <el-tag :type="row.is_admin ? 'danger' : 'info'" size="small">
                  {{ row.is_admin ? t('common.admin') : t('common.user') }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-card>

        <el-card v-else shadow="never" class="admin-settings__panel">
          <div class="admin-settings__panel-toolbar">
            <span>{{ t('admin.workspaces') }}</span>
            <button type="button" class="admin-settings__create-btn" @click="handleCreateWorkspaceAdmin">
              <el-icon :size="16"><Plus /></el-icon>
              <span>{{ t('workspace.createWorkspace') }}</span>
            </button>
          </div>
          <el-table v-loading="loadingWorkspaces" :data="workspaces" stripe>
            <el-table-column prop="id" :label="t('admin.columns.id')" width="80" />
            <el-table-column prop="name" :label="t('admin.columns.name')" min-width="160" />
            <el-table-column prop="owner_name" :label="t('admin.columns.owner')" min-width="140" />
            <el-table-column prop="created_at" :label="t('admin.columns.createdAt')" min-width="180" />
            <el-table-column :label="t('admin.transferTo')" min-width="220">
              <template #default="{ row }">
                <el-select
                  :placeholder="t('admin.selectUser')"
                  style="width: 100%"
                  popper-class="app-action-dropdown"
                  :loading="transferringId === row.id"
                  @change="(value: number) => handleTransfer(row, value)"
                >
                  <el-option
                    v-for="user in users"
                    :key="user.id"
                    :label="`${user.name || user.account} (${user.account})`"
                    :value="user.id"
                  />
                </el-select>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </section>
    </div>
  </div>
</template>

<style scoped>
.admin-settings {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.admin-settings__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
  flex-shrink: 0;
}

.admin-settings__header-info h1 {
  margin: 0 0 6px;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text);
}

.admin-settings__header-info p {
  margin: 0;
  font-size: 14px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.admin-settings__close {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.admin-settings__close:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.admin-settings__body {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}

.admin-settings__sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--color-sidebar);
  border-right: 1px solid var(--color-border);
  padding: 16px 10px;
}

.admin-settings__menu {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.admin-settings__menu-item {
  width: 100%;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  border-radius: 8px;
  padding: 12px 14px;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  font-size: 15px;
  text-align: left;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.admin-settings__menu-item:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.admin-settings__menu-item--active {
  background: var(--color-active);
  color: var(--color-primary-light);
  font-weight: 500;
}

.admin-settings__content {
  flex: 1;
  min-width: 0;
  padding: 20px;
  background: var(--color-bg);
  overflow: auto;
}

.admin-settings__panel {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-surface);
  box-shadow: none;
}

.admin-settings__panel-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.admin-settings__create-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 36px;
  padding: 0 14px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface);
  color: var(--color-primary-light);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s ease, border-color 0.15s ease;
}

.admin-settings__create-btn:hover {
  background: var(--color-active);
  border-color: var(--color-primary);
}
</style>
