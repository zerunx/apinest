<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from 'vue'
import type { ElTree } from 'element-plus'
import { ElMessage } from 'element-plus'
import { fetchFolderTree } from '@/api/folder'
import {
  createProjectShare,
  fetchProjectShareDetail,
  updateProjectShare,
} from '@/api/share'
import { useLocale } from '@/composables/useLocale'
import { parseApiId, useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { ApiTreeNode } from '@/types/workspace'

const props = defineProps<{
  shareId?: number | null
}>()

const emit = defineEmits<{
  saved: []
}>()

const { t } = useLocale()
const { activeWorkspaceId, activeProjectId, activeProject } = useWorkspaceContext()

const visible = defineModel<boolean>({ default: false })

const loading = ref(false)
const submitting = ref(false)
const treeData = ref<ApiTreeNode[]>([])
const treeRef = ref<InstanceType<typeof ElTree>>()
const defaultCheckedKeys = ref<string[]>([])

const SHARE_NAME_MAX = 50

const form = reactive({
  name: '',
  enabled: true,
  password: '',
  clearPassword: false,
  permission: 1,
  interfaceIds: [] as number[],
})

const isEdit = computed(() => Boolean(props.shareId))
const dialogTitle = computed(() =>
  isEdit.value ? t('share.editTitle') : t('share.createTitle'),
)

const hasExistingPassword = ref(false)

function defaultShareName() {
  const projectName = activeProject.value?.name?.trim() || t('workspace.project')
  return t('share.defaultName', { name: projectName }).slice(0, SHARE_NAME_MAX)
}

const allApiKeys = computed(() => collectApiKeys(treeData.value))

const allChecked = computed(() => {
  if (!allApiKeys.value.length) return false
  return form.interfaceIds.length === allApiKeys.value.length
})

function collectApiKeys(nodes: ApiTreeNode[]): string[] {
  const keys: string[] = []
  for (const node of nodes) {
    if (node.type === 'api') {
      keys.push(node.id)
    }
    if (node.children?.length) {
      keys.push(...collectApiKeys(node.children))
    }
  }
  return keys
}

function collectApiIds(nodes: ApiTreeNode[]): number[] {
  return collectApiKeys(nodes)
    .map((key) => parseApiId(key))
    .filter((id): id is number => id != null)
}

async function loadTree() {
  if (!activeWorkspaceId.value || !activeProjectId.value) {
    treeData.value = []
    return
  }
  treeData.value = await fetchFolderTree(activeWorkspaceId.value, activeProjectId.value)
}

async function loadDetail() {
  if (!activeWorkspaceId.value || !props.shareId) return

  const detail = await fetchProjectShareDetail(activeWorkspaceId.value, props.shareId)
  form.name = detail.name
  form.enabled = detail.enabled
  form.permission = detail.permission || 1
  form.interfaceIds = [...(detail.interface_ids || [])]
  form.password = ''
  form.clearPassword = false
  hasExistingPassword.value = detail.has_password
  defaultCheckedKeys.value = form.interfaceIds.map((id) => `api-${id}`)
}

function resetForm() {
  form.name = ''
  form.enabled = true
  form.password = ''
  form.clearPassword = false
  form.permission = 1
  form.interfaceIds = []
  defaultCheckedKeys.value = []
  hasExistingPassword.value = false
}

function syncCheckedInterfaces() {
  const keys = (treeRef.value?.getCheckedKeys(false) || []) as string[]
  form.interfaceIds = keys
    .map((key) => parseApiId(key))
    .filter((id): id is number => id != null)
}

function handleCheck() {
  syncCheckedInterfaces()
}

function toggleAll(checked: boolean) {
  if (!treeRef.value) return
  if (checked) {
    treeRef.value.setCheckedKeys(allApiKeys.value)
  } else {
    treeRef.value.setCheckedKeys([])
  }
  syncCheckedInterfaces()
}

async function handleSubmit() {
  if (!activeWorkspaceId.value || !activeProjectId.value) return

  syncCheckedInterfaces()

  const name = form.name.trim()
  if (!name) {
    ElMessage.warning(t('share.nameRequired'))
    return
  }
  if (name.length > SHARE_NAME_MAX) {
    ElMessage.warning(t('share.nameTooLong'))
    return
  }
  if (!form.interfaceIds.length) {
    ElMessage.warning(t('share.interfacesRequired'))
    return
  }

  submitting.value = true
  try {
    if (isEdit.value && props.shareId) {
      let password: string | null | undefined
      if (form.clearPassword) {
        password = ''
      } else if (form.password.trim()) {
        password = form.password.trim()
      } else {
        password = undefined
      }

      await updateProjectShare({
        workspace_id: activeWorkspaceId.value,
        share_id: props.shareId,
        name,
        enabled: form.enabled,
        password,
        permission: form.permission,
        interface_ids: form.interfaceIds,
      })
      ElMessage.success(t('share.updateSuccess'))
    } else {
      await createProjectShare({
        workspace_id: activeWorkspaceId.value,
        project_id: activeProjectId.value,
        name,
        enabled: form.enabled,
        password: form.password.trim() || undefined,
        permission: form.permission,
        interface_ids: form.interfaceIds,
      })
      ElMessage.success(t('share.createSuccess'))
    }
    visible.value = false
    emit('saved')
  } catch (error) {
    ElMessage.error(
      error instanceof Error
        ? error.message
        : isEdit.value
          ? t('share.updateFailed')
          : t('share.createFailed'),
    )
  } finally {
    submitting.value = false
  }
}

watch(visible, async (open) => {
  if (!open) return
  loading.value = true
  resetForm()
  try {
    await loadTree()
    if (props.shareId) {
      await loadDetail()
    } else {
      form.name = defaultShareName()
      const ids = collectApiIds(treeData.value)
      form.interfaceIds = ids
      defaultCheckedKeys.value = ids.map((id) => `api-${id}`)
    }
    await nextTick()
    treeRef.value?.setCheckedKeys(defaultCheckedKeys.value)
    syncCheckedInterfaces()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('share.loadFailed'))
    visible.value = false
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="dialogTitle"
    width="640px"
    top="8vh"
    class="project-share-edit-dialog"
    append-to-body
    destroy-on-close
  >
    <div v-loading="loading" class="project-share-edit">
      <div class="project-share-edit__row">
        <div class="project-share-edit__field project-share-edit__field--grow">
          <label>{{ t('share.name') }}</label>
          <el-input
            v-model="form.name"
            :placeholder="t('share.namePlaceholder')"
            maxlength="50"
            show-word-limit
          />
        </div>
        <div class="project-share-edit__field project-share-edit__field--status">
          <label>{{ t('share.enabledLabel') }}</label>
          <div class="project-share-edit__switch">
            <el-switch v-model="form.enabled" />
            <span class="project-share-edit__switch-text">
              {{ form.enabled ? t('share.enabled') : t('share.disabled') }}
            </span>
          </div>
        </div>
      </div>

      <div class="project-share-edit__field">
        <label>{{ t('share.password') }}</label>
        <el-input
          v-model="form.password"
          type="password"
          show-password
          :placeholder="
            isEdit && hasExistingPassword
              ? t('share.passwordKeepPlaceholder')
              : t('share.passwordPlaceholder')
          "
          :disabled="form.clearPassword"
        />
        <label v-if="isEdit && hasExistingPassword" class="project-share-edit__check">
          <el-checkbox v-model="form.clearPassword">{{ t('share.clearPassword') }}</el-checkbox>
        </label>
      </div>

      <div class="project-share-edit__field">
        <div class="project-share-edit__interfaces-head">
          <label>{{ t('share.interfaces') }}</label>
          <el-checkbox :model-value="allChecked" @change="(v: boolean) => toggleAll(v)">
            {{ t('share.selectAll') }}
          </el-checkbox>
        </div>
        <div class="project-share-edit__tree-wrap">
          <div v-if="!treeData.length" class="project-share-edit__empty">
            {{ t('share.noInterfaces') }}
          </div>
          <el-tree
            v-else
            ref="treeRef"
            class="project-share-edit__tree"
            :data="treeData"
            node-key="id"
            show-checkbox
            default-expand-all
            :props="{ children: 'children', label: 'name' }"
            :default-checked-keys="defaultCheckedKeys"
            @check="handleCheck"
          >
            <template #default="{ data }">
              <div class="project-share-edit__tree-node">
                <span
                  v-if="data.type === 'api' && data.method"
                  class="project-share-edit__method-wrap"
                >
                  <span
                    class="http-method-badge"
                    :class="`http-method-badge--${String(data.method).toLowerCase()}`"
                  >
                    {{ data.method }}
                  </span>
                </span>
                <span class="project-share-edit__tree-label">{{ data.name }}</span>
              </div>
            </template>
          </el-tree>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="project-share-edit__footer">
        <button type="button" class="project-share-edit__btn" @click="visible = false">
          {{ t('common.cancel') }}
        </button>
        <button
          type="button"
          class="project-share-edit__btn project-share-edit__btn--primary"
          :disabled="submitting"
          @click="handleSubmit"
        >
          {{ t('common.save') }}
        </button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.project-share-edit {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 200px;
}

.project-share-edit__row {
  display: flex;
  align-items: flex-end;
  gap: 16px;
}

.project-share-edit__field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.project-share-edit__field--grow {
  flex: 1;
  min-width: 0;
}

.project-share-edit__field--status {
  flex-shrink: 0;
  width: 120px;
}

.project-share-edit__field > label {
  font-size: 13px;
  color: var(--color-text);
}

.project-share-edit__switch {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 32px;
}

.project-share-edit__switch-text {
  font-size: 13px;
  color: var(--color-text-secondary);
}

.project-share-edit__check {
  margin-top: 2px;
}

.project-share-edit__interfaces-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.project-share-edit__tree-wrap {
  max-height: 300px;
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-workspace-inset);
  padding: 8px;
}

.project-share-edit__empty {
  padding: 24px;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.project-share-edit__tree {
  background: transparent;
  --el-tree-node-hover-bg-color: var(--color-hover);
  color: var(--color-text);
}

.project-share-edit__tree :deep(.el-tree-node__content) {
  height: 32px;
  border-radius: 6px;
  background: transparent;
}

.project-share-edit__tree :deep(.el-tree-node__content:hover) {
  background-color: var(--color-hover);
}

.project-share-edit__tree :deep(.el-tree-node:focus > .el-tree-node__content) {
  background-color: var(--color-hover);
}

.project-share-edit__tree-node {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.project-share-edit__method-wrap {
  width: 52px;
  min-width: 52px;
  flex-shrink: 0;
  display: inline-flex;
  justify-content: center;
}

.project-share-edit__tree-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  color: var(--color-text);
}

.project-share-edit__footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.project-share-edit__btn {
  min-width: 88px;
  height: 34px;
  padding: 0 14px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 13px;
  cursor: pointer;
}

.project-share-edit__btn:hover {
  background: var(--color-hover);
}

.project-share-edit__btn--primary {
  background: var(--color-text);
  border-color: var(--color-text);
  color: var(--color-bg);
}

.project-share-edit__btn--primary:hover {
  opacity: 0.88;
  background: var(--color-text);
  border-color: var(--color-text);
  color: var(--color-bg);
}

.project-share-edit__btn--primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>

<style>
.project-share-edit-dialog.el-dialog {
  --el-dialog-padding-primary: 0;
  padding: 0;
}

.project-share-edit-dialog .el-dialog__header {
  position: relative;
  display: flex;
  align-items: center;
  margin-right: 0;
  padding: 16px 52px 16px 24px;
  border-bottom: 1px solid var(--color-border);
}

.project-share-edit-dialog .el-dialog__body {
  padding: 20px 24px;
}

.project-share-edit-dialog .el-dialog__footer {
  padding: 16px 24px;
  border-top: 1px solid var(--color-border);
}

.project-share-edit-dialog .el-dialog__headerbtn {
  top: 50%;
  right: 16px;
  width: 28px;
  height: 28px;
  transform: translateY(-50%);
}

.project-share-edit-dialog .el-dialog__headerbtn .el-dialog__close {
  font-size: 18px;
  color: var(--color-text);
  opacity: 0.85;
}

.project-share-edit-dialog .el-dialog__headerbtn:hover .el-dialog__close {
  color: var(--color-text);
  opacity: 1;
}

.project-share-edit-dialog .el-input__wrapper {
  background-color: var(--color-surface) !important;
  box-shadow: 0 0 0 1px var(--color-border) inset !important;
}

.project-share-edit-dialog .el-input__inner {
  color: var(--color-text) !important;
  -webkit-text-fill-color: var(--color-text) !important;
}

.project-share-edit-dialog .el-input__inner::placeholder {
  color: var(--color-text-secondary) !important;
  -webkit-text-fill-color: var(--color-text-secondary) !important;
  opacity: 1;
}

.project-share-edit-dialog .el-input__count-inner {
  color: var(--color-text-secondary);
}
</style>
