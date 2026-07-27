<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { CopyDocument, Delete, Edit, Plus, Share } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  deleteProjectShare,
  fetchProjectShares,
  type ProjectShareItem,
} from '@/api/share'
import ProjectShareEditDialog from '@/components/ProjectShareEditDialog.vue'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'

const { t } = useLocale()
const { activeWorkspaceId, activeProject, activeProjectId } = useWorkspaceContext()

const visible = defineModel<boolean>({ default: false })

const loading = ref(false)
const shares = ref<ProjectShareItem[]>([])
const editVisible = ref(false)
const editingShareId = ref<number | null>(null)

const projectName = computed(() => activeProject.value?.name || '')

async function loadShares() {
  if (!activeWorkspaceId.value || !activeProjectId.value) {
    shares.value = []
    return
  }

  loading.value = true
  try {
    shares.value = await fetchProjectShares(activeWorkspaceId.value, activeProjectId.value)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('share.loadFailed'))
  } finally {
    loading.value = false
  }
}

function openCreate() {
  editingShareId.value = null
  editVisible.value = true
}

function openEdit(row: ProjectShareItem) {
  editingShareId.value = row.id
  editVisible.value = true
}

async function copyLink(row: ProjectShareItem) {
  try {
    await navigator.clipboard.writeText(row.share_url)
    ElMessage.success(t('share.copyLinkSuccess'))
  } catch {
    ElMessage.error(t('share.copyFailed'))
  }
}

async function handleDelete(row: ProjectShareItem) {
  if (!activeWorkspaceId.value) return

  try {
    await ElMessageBox.confirm(
      t('share.deleteConfirm', { name: row.name }),
      t('share.deleteTitle'),
      {
        type: 'warning',
        customClass: 'project-share-message-box',
        modalClass: 'project-share-message-box-overlay',
      },
    )
  } catch {
    return
  }

  try {
    await deleteProjectShare(activeWorkspaceId.value, row.id)
    ElMessage.success(t('share.deleteSuccess'))
    await loadShares()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('share.deleteFailed'))
  }
}

function handleEditSaved() {
  editVisible.value = false
  loadShares()
}

watch(visible, (open) => {
  if (open) loadShares()
})
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="t('share.title')"
    width="720px"
    top="10vh"
    class="project-share-dialog"
    destroy-on-close
  >
    <div class="project-share-dialog__toolbar">
      <p class="project-share-dialog__desc">
        {{ t('share.desc', { name: projectName }) }}
      </p>
      <button type="button" class="project-share-dialog__create" @click="openCreate">
        <el-icon :size="14"><Plus /></el-icon>
        <span>{{ t('share.create') }}</span>
      </button>
    </div>

    <div v-loading="loading" class="project-share-dialog__body">
      <div v-if="!shares.length && !loading" class="project-share-dialog__empty">
        <el-icon :size="28"><Share /></el-icon>
        <p>{{ t('share.empty') }}</p>
      </div>

      <div v-else class="project-share-dialog__list">
        <div v-for="row in shares" :key="row.id" class="project-share-dialog__item">
          <strong class="project-share-dialog__name" :title="row.name">{{ row.name }}</strong>
          <div class="project-share-dialog__meta">
            <span
              class="project-share-dialog__badge"
              :class="row.enabled ? 'is-on' : 'is-off'"
            >
              {{ row.enabled ? t('share.enabled') : t('share.disabled') }}
            </span>
            <span v-if="row.has_password" class="project-share-dialog__badge">
              {{ t('share.hasPassword') }}
            </span>
            <span class="project-share-dialog__time">{{ row.created_at }}</span>
            <div class="project-share-dialog__item-actions">
              <button
                type="button"
                class="project-share-dialog__action"
                :title="t('share.copyLink')"
                @click="copyLink(row)"
              >
                <el-icon :size="15"><CopyDocument /></el-icon>
              </button>
              <button
                type="button"
                class="project-share-dialog__action"
                :title="t('common.edit')"
                @click="openEdit(row)"
              >
                <el-icon :size="15"><Edit /></el-icon>
              </button>
              <button
                type="button"
                class="project-share-dialog__action is-danger"
                :title="t('common.delete')"
                @click="handleDelete(row)"
              >
                <el-icon :size="15"><Delete /></el-icon>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <ProjectShareEditDialog
      v-model="editVisible"
      :share-id="editingShareId"
      @saved="handleEditSaved"
    />
  </el-dialog>
</template>

<style scoped>
.project-share-dialog__toolbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.project-share-dialog__desc {
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-secondary);
}

.project-share-dialog__create {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 13px;
  cursor: pointer;
}

.project-share-dialog__create:hover {
  background: var(--color-hover);
}

.project-share-dialog__body {
  min-height: 140px;
  padding-bottom: 8px;
}

.project-share-dialog__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-height: 160px;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.project-share-dialog__empty p {
  margin: 0;
}

.project-share-dialog__list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 320px;
  overflow: auto;
  padding-bottom: 4px;
}

.project-share-dialog__item {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 12px 14px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-workspace-inset);
}

.project-share-dialog__name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}

.project-share-dialog__meta {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 20px;
  margin-left: auto;
}

.project-share-dialog__badge {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  height: 20px;
  padding: 0 8px;
  border-radius: 999px;
  background: var(--color-hover);
  color: var(--color-text-secondary);
  font-size: 12px;
}

.project-share-dialog__badge.is-on {
  color: #16a34a;
  background: rgba(22, 163, 74, 0.12);
}

.project-share-dialog__badge.is-off {
  color: #a8a8a8;
}

.project-share-dialog__time {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.project-share-dialog__item-actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 4px;
}

.project-share-dialog__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  padding: 0;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.project-share-dialog__action:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.project-share-dialog__action.is-danger:hover {
  color: var(--color-danger);
  background: var(--color-danger-hover-bg);
}
</style>

<style>
.project-share-dialog.el-dialog {
  --el-dialog-padding-primary: 0;
  padding: 0;
}

.project-share-dialog .el-dialog__header {
  position: relative;
  display: flex;
  align-items: center;
  margin-right: 0;
  padding: 16px 52px 16px 24px;
  border-bottom: 1px solid var(--color-border);
}

.project-share-dialog .el-dialog__body {
  padding: 20px 24px 32px;
}

.project-share-dialog .el-dialog__headerbtn {
  top: 50%;
  right: 16px;
  width: 28px;
  height: 28px;
  transform: translateY(-50%);
}

.project-share-dialog .el-dialog__headerbtn .el-dialog__close {
  font-size: 18px;
  color: var(--color-text);
  opacity: 0.85;
}

.project-share-dialog .el-dialog__headerbtn:hover .el-dialog__close {
  color: var(--color-text);
  opacity: 1;
}
</style>
