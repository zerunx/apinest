<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'

const { t } = useLocale()
const { createWorkspaceVisible, submitCreateWorkspace } = useWorkspaceContext()

const form = reactive({ name: '' })
const submitting = ref(false)

watch(createWorkspaceVisible, (visible) => {
  if (visible) form.name = ''
})

async function handleSubmit() {
  const name = form.name.trim()
  if (!name) {
    ElMessage.warning(t('workspace.nameRequired'))
    return
  }

  submitting.value = true
  try {
    await submitCreateWorkspace(name)
    ElMessage.success(t('workspace.createWorkspaceSuccess'))
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.createWorkspaceFailed'))
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <el-dialog
    v-model="createWorkspaceVisible"
    :title="t('workspace.createWorkspace')"
    width="420px"
    destroy-on-close
  >
    <el-form label-position="top" @submit.prevent="handleSubmit">
      <el-form-item :label="t('workspace.workspaceName')" required>
        <el-input
          v-model="form.name"
          :placeholder="t('workspace.workspaceNamePlaceholder')"
          maxlength="100"
          show-word-limit
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="createWorkspaceVisible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="submitting" @click="handleSubmit">
        {{ t('common.confirm') }}
      </el-button>
    </template>
  </el-dialog>
</template>
