<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { useAuth } from '@/composables/useAuth'

const router = useRouter()
const route = useRoute()
const { register } = useAuth()

const formRef = ref<FormInstance>()
const loading = ref(false)

const form = reactive({
  name: '',
  account: '',
  email: '',
  password: '',
  confirm_password: '',
})

const validateConfirmPassword = (_rule: unknown, value: string, callback: (error?: Error) => void) => {
  if (value !== form.password) {
    callback(new Error('两次输入的密码不一致'))
    return
  }
  callback()
}

const rules: FormRules = {
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  account: [
    { required: true, message: '请输入账号', trigger: 'blur' },
    { min: 3, max: 50, message: '账号长度为 3-50 个字符', trigger: 'blur' },
  ],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '邮箱格式不正确', trigger: 'blur' },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码至少 6 位', trigger: 'blur' },
  ],
  confirm_password: [
    { required: true, message: '请再次输入密码', trigger: 'blur' },
    { validator: validateConfirmPassword, trigger: 'blur' },
  ],
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
    await register(form)
    ElMessage.success('注册成功')
    const redirect = (route.query.redirect as string) || '/home'
    router.push(redirect)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '注册失败')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="register-view">
    <h2>注册账号</h2>
    <p class="register-view__desc">创建 ApiNest 账号，注册后可创建工作空间并协作开发 API。</p>

    <el-form ref="formRef" :model="form" :rules="rules" label-position="top" @submit.prevent>
      <el-form-item label="名称" prop="name">
        <el-input v-model="form.name" placeholder="显示名称" />
      </el-form-item>

      <el-form-item label="账号" prop="account">
        <el-input v-model="form.account" placeholder="登录账号，支持字母、数字、下划线" />
      </el-form-item>

      <el-form-item label="邮箱" prop="email">
        <el-input v-model="form.email" placeholder="example@email.com" />
      </el-form-item>

      <el-form-item label="密码" prop="password">
        <el-input v-model="form.password" type="password" show-password placeholder="至少 6 位" />
      </el-form-item>

      <el-form-item label="确认密码" prop="confirm_password">
        <el-input
          v-model="form.confirm_password"
          type="password"
          show-password
          placeholder="再次输入密码"
          @keyup.enter="handleSubmit"
        />
      </el-form-item>

      <el-button type="primary" class="register-view__submit" :loading="loading" @click="handleSubmit">
        注册
      </el-button>
    </el-form>

    <p class="register-view__footer">
      已有账号？
      <router-link to="/login">返回登录</router-link>
    </p>
  </div>
</template>

<style scoped>
.register-view h2 {
  margin: 0 0 8px;
  font-size: 28px;
  font-weight: 700;
  color: #0f172a;
}

.register-view__desc {
  margin: 0 0 24px;
  color: #64748b;
  font-size: 14px;
  line-height: 1.5;
}

.register-view__submit {
  width: 100%;
  margin-top: 4px;
}

.register-view__footer {
  margin: 20px 0 0;
  text-align: center;
  color: #64748b;
  font-size: 14px;
}
</style>
