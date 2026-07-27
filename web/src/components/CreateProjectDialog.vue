<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'

const { t } = useLocale()
const { createProjectVisible, submitCreateProject, activeWorkspace } = useWorkspaceContext()

const form = reactive({ name: '' })
const submitting = ref(false)

watch(createProjectVisible, (visible) => {
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
    await submitCreateProject(name)
    ElMessage.success(t('workspace.createProjectSuccess'))
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.createProjectFailed'))
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <el-dialog
    v-model="createProjectVisible"
    :title="t('workspace.createProject')"
    width="420px"
    destroy-on-close
  >
    <p v-if="activeWorkspace" class="create-project-dialog__hint">
      {{ t('workspace.createProjectIn', { name: activeWorkspace.name }) }}
    </p>
    <el-form label-position="top" @submit.prevent="handleSubmit">
      <el-form-item :label="t('workspace.projectName')" required>
        <el-input
          v-model="form.name"
          :placeholder="t('workspace.projectNamePlaceholder')"
          maxlength="100"
          show-word-limit
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="createProjectVisible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="submitting" @click="handleSubmit">
        {{ t('common.confirm') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.create-project-dialog__hint {
  margin: 0 0 16px;
  font-size: 14px;
  color: var(--color-text-secondary);
}
</style>
