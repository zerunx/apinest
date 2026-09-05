<script setup lang="ts">
import { computed } from 'vue'
import { CopyDocument } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useLocale } from '@/composables/useLocale'

const props = withDefaults(
  defineProps<{
    value?: string
    empty?: string
  }>(),
  {
    value: '',
    empty: '',
  },
)

const { t } = useLocale()

const trimmed = computed(() => props.value.trim())
const copyable = computed(() => !!trimmed.value)
const display = computed(() => (copyable.value ? props.value : props.empty))

async function copy(event?: Event) {
  if (!copyable.value) return
  try {
    await navigator.clipboard.writeText(trimmed.value)
    ElMessage.success(t('workspace.interfaceForm.rawCopySuccess'))
  } catch {
    ElMessage.error(t('workspace.interfaceForm.rawCopyFailed'))
  } finally {
    ;(event?.currentTarget as HTMLButtonElement | undefined)?.blur()
  }
}
</script>

<template>
  <div
    v-if="copyable || empty"
    class="interface-doc-copyable"
    :class="{ 'interface-doc-copyable--empty': !copyable }"
  >
    <span class="interface-doc-copyable__text">{{ display }}</span>
    <button
      v-if="copyable"
      type="button"
      class="interface-doc-copyable__btn"
      tabindex="-1"
      :title="t('share.public.copyHint')"
      @click.stop="copy($event)"
    >
      <el-icon :size="13"><CopyDocument /></el-icon>
    </button>
  </div>
</template>
