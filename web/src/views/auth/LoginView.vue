<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { useAuth } from '@/composables/useAuth'

const router = useRouter()
const route = useRoute()
const { login } = useAuth()

const formRef = ref<FormInstance>()
const loading = ref(false)

const form = reactive({
  account: '',
  password: '',
})

const rules: FormRules = {
  account: [{ required: true, message: '请输入账号', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }],
}

async function handleSubmit() {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  loading.value = true
  try {
    await login(form)
    ElMessage.success('登录成功')
    const redirect = (route.query.redirect as string) || '/home'
    router.push(redirect)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '登录失败')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="login-view">
    <h2>登录</h2>
    <p class="login-view__desc">使用账号和密码登录，管理员与普通用户使用同一入口。</p>

    <el-form ref="formRef" :model="form" :rules="rules" label-position="top" @submit.prevent>
      <el-form-item label="账号" prop="account">
        <el-input v-model="form.account" placeholder="登录账号" autocomplete="username" />
      </el-form-item>

      <el-form-item label="密码" prop="password">
        <el-input
          v-model="form.password"
          type="password"
          show-password
          placeholder="登录密码"
          autocomplete="current-password"
          @keyup.enter="handleSubmit"
        />
      </el-form-item>

      <el-button type="primary" class="login-view__submit" :loading="loading" @click="handleSubmit">
        登录
      </el-button>
    </el-form>

    <p class="login-view__footer">
      还没有账号？
      <router-link to="/register">立即注册</router-link>
    </p>
  </div>
</template>

<style scoped>
.login-view h2 {
  margin: 0 0 8px;
  font-size: 28px;
  font-weight: 700;
  color: #0f172a;
}

.login-view__desc {
  margin: 0 0 24px;
  color: #64748b;
  font-size: 14px;
  line-height: 1.5;
}

.login-view__submit {
  width: 100%;
  margin-top: 4px;
}

.login-view__footer {
  margin: 20px 0 0;
  text-align: center;
  color: #64748b;
  font-size: 14px;
}
</style>
