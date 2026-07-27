<template>
  <div class="install-page">
    <div class="install-bg">
      <div class="install-bg__orb install-bg__orb--1" />
      <div class="install-bg__orb install-bg__orb--2" />
    </div>

    <header class="install-topbar">
      <div class="install-topbar__inner">
        <img class="install-logo app-logo" src="/nest.png" alt="ApiNest" />
        <div class="install-topbar__text">
          <h1>ApiNest 安装向导</h1>
          <p>开源 API 协作管理平台 · 首次部署请完成以下配置</p>
        </div>
      </div>
    </header>

    <main class="install-main">
      <el-card v-if="alreadyInstalled" class="install-card" shadow="never">
        <el-result icon="success" title="系统已安装" sub-title="ApiNest 已完成初始化，请前往登录页面使用。">
          <template #extra>
            <el-button type="primary" @click="goLogin">前往登录</el-button>
          </template>
        </el-result>
      </el-card>

      <el-card v-else-if="installCredentials" class="install-card install-success" shadow="never">
        <el-result icon="success" title="安装成功" sub-title="请妥善保管以下账号信息，关闭页面后将无法再次查看明文密码。">
          <template #extra>
            <div class="install-credentials">
              <section class="install-credentials__section">
                <h3>平台管理员</h3>
                <p class="install-credentials__hint">用于登录 ApiNest 管理后台。</p>

                <div
                  class="install-credentials__row install-credentials__row--clickable"
                  @click="copyText(installCredentials.admin.username)"
                >
                  <span class="install-credentials__label">用户名</span>
                  <code>{{ installCredentials.admin.username }}</code>
                  <el-button text type="primary" @click.stop="copyText(installCredentials.admin.username)">
                    复制
                  </el-button>
                </div>

                <div
                  class="install-credentials__row install-credentials__row--clickable"
                  @click="copyText(installCredentials.admin.password)"
                >
                  <span class="install-credentials__label">密码</span>
                  <code>{{ installCredentials.admin.password }}</code>
                  <el-button text type="primary" @click.stop="copyText(installCredentials.admin.password)">
                    复制
                  </el-button>
                </div>
              </section>

              <section class="install-credentials__section">
                <h3>应用数据库账号</h3>
                <p class="install-credentials__hint">
                  平台日常连接数据库使用，凭据已写入 <code>runtime/config.yaml</code>。
                </p>

                <div
                  class="install-credentials__row install-credentials__row--clickable"
                  @click="copyText(installCredentials.database.username)"
                >
                  <span class="install-credentials__label">用户名</span>
                  <code>{{ installCredentials.database.username }}</code>
                  <el-button text type="primary" @click.stop="copyText(installCredentials.database.username)">
                    复制
                  </el-button>
                </div>

                <div
                  class="install-credentials__row install-credentials__row--clickable"
                  @click="copyText(installCredentials.database.password)"
                >
                  <span class="install-credentials__label">密码</span>
                  <code>{{ installCredentials.database.password }}</code>
                  <el-button text type="primary" @click.stop="copyText(installCredentials.database.password)">
                    复制
                  </el-button>
                </div>
              </section>
            </div>

            <el-button type="primary" size="large" @click="goLogin">前往登录</el-button>
          </template>
        </el-result>
      </el-card>

      <el-form
        v-else
        ref="formRef"
        :model="form"
        :rules="rules"
        label-position="top"
        class="install-form"
        @submit.prevent
      >
        <!-- 数据库配置 -->
        <el-card class="install-block" shadow="never">
          <div class="install-block__head">
            <span class="install-block__index">01</span>
            <div>
              <h2>数据库连接</h2>
              <p>填写 PostgreSQL 服务信息。安装完成后，平台将使用自动创建的专用账号连接数据库。</p>
            </div>
          </div>

          <div class="form-section">
            <h3 class="form-section__title">服务信息</h3>
            <div class="form-grid">
              <el-form-item label="数据库类型" prop="database.driver" class="form-grid__full">
                <el-select v-model="form.database.driver" popper-class="install-popper">
                  <el-option label="PostgreSQL" value="postgres" />
                  <el-option label="MySQL（即将支持）" value="mysql" disabled />
                </el-select>
              </el-form-item>

              <el-form-item label="主机地址" prop="database.host">
                <el-input v-model="form.database.host" placeholder="localhost 或数据库 IP / 域名" />
              </el-form-item>

              <el-form-item label="端口" prop="database.port">
                <el-input-number
                  v-model="form.database.port"
                  :min="1"
                  :max="65535"
                  :controls="false"
                  style="width: 100%"
                />
              </el-form-item>

              <el-form-item label="数据库名" prop="database.name" class="form-grid__full">
                <el-input v-model="form.database.name" placeholder="例如 nest" />
              </el-form-item>

              <el-form-item label="SSL 模式" prop="database.ssl_mode" class="form-grid__full">
                <el-select v-model="form.database.ssl_mode" popper-class="install-popper">
                  <el-option label="disable — 不使用 SSL（内网/本地推荐）" value="disable" />
                  <el-option label="require — 使用 SSL，不校验证书" value="require" />
                  <el-option label="verify-ca — 使用 SSL 并校验 CA" value="verify-ca" />
                  <el-option label="verify-full — 使用 SSL 并校验 CA 与主机名" value="verify-full" />
                </el-select>
              </el-form-item>
            </div>
          </div>

          <div class="form-section">
            <h3 class="form-section__title">安装用超级用户</h3>
            <p class="form-section__desc">
              需要具备创建数据库、创建用户等权限的账号，仅用于本次安装，不会写入配置文件。
            </p>
            <div class="form-grid">
              <el-form-item label="超级用户名" prop="database.user">
                <el-input v-model="form.database.user" placeholder="例如: postgres" />
              </el-form-item>

              <el-form-item label="超级用户密码" prop="database.password">
                <div class="password-field">
                  <el-input
                    v-model="form.database.password"
                    type="password"
                    show-password
                    placeholder="超级用户密码"
                  />
                  <el-button class="install-action-btn" :loading="testing" @click="handleTestConnection">测试连接</el-button>
                </div>
              </el-form-item>
            </div>
          </div>

          <div class="form-section">
            <h3 class="form-section__title">应用数据库用户</h3>
            <p class="form-section__desc">
              平台日常连接数据库使用的专用账号，可按需修改。
            </p>
            <div class="form-grid">
              <el-form-item label="用户名" prop="app_database.username">
                <el-input v-model="form.app_database.username" placeholder="例如 nest" />
              </el-form-item>

              <el-form-item label="密码" prop="app_database.password">
                <div class="password-field">
                  <el-input
                    v-model="form.app_database.password"
                    type="password"
                    show-password
                    placeholder="至少 6 位"
                  />
                  <el-button class="install-action-btn" @click="regenerateAppPassword">重新生成</el-button>
                </div>
              </el-form-item>
            </div>
          </div>

        </el-card>

        <!-- 管理员账号 -->
        <el-card class="install-block" shadow="never">
          <div class="install-block__head">
            <span class="install-block__index">02</span>
            <div>
              <h2>管理员账号</h2>
              <p>平台登录用的超级管理员，与上面的数据库超级用户无关。</p>
            </div>
          </div>

          <div class="form-grid">
            <el-form-item label="用户名" prop="admin.username" class="form-grid__full">
              <el-input v-model="form.admin.username" placeholder="3-50 个字符" />
            </el-form-item>

            <el-form-item label="登录密码" prop="admin.password">
              <el-input
                v-model="form.admin.password"
                type="password"
                show-password
                placeholder="至少 6 位"
              />
            </el-form-item>

            <el-form-item label="确认密码" prop="admin.confirm_password">
              <el-input
                v-model="form.admin.confirm_password"
                type="password"
                show-password
                placeholder="再次输入密码"
              />
            </el-form-item>
          </div>
        </el-card>

        <!-- 确认安装 -->
        <el-card class="install-block" shadow="never">
          <div class="install-block__head">
            <span class="install-block__index">03</span>
            <div>
              <h2>确认安装</h2>
              <p>请确认以下配置信息，提交后将初始化数据库并创建管理员账号。</p>
            </div>
          </div>

          <div class="form-grid">
            <el-form-item label="站点域名" prop="site_url" class="form-grid__full">
              <el-input
                v-model="form.site_url"
                placeholder="例如 https://api.example.com，用于邀请链接和分享"
              />
            </el-form-item>
          </div>

          <el-descriptions :column="2" border class="install-summary">
            <el-descriptions-item label="站点域名">{{ form.site_url }}</el-descriptions-item>
            <el-descriptions-item label="平台管理员">{{ form.admin.username }}</el-descriptions-item>
            <el-descriptions-item label="数据库类型">
              {{ form.database.driver === 'postgres' ? 'PostgreSQL' : form.database.driver }}
            </el-descriptions-item>
            <el-descriptions-item label="SSL 模式">{{ form.database.ssl_mode }}</el-descriptions-item>
            <el-descriptions-item label="主机地址">
              {{ form.database.host }}:{{ form.database.port }}
            </el-descriptions-item>
            <el-descriptions-item label="数据库名">{{ form.database.name }}</el-descriptions-item>
            <el-descriptions-item label="应用数据库用户">{{ form.app_database.username }}</el-descriptions-item>
            <el-descriptions-item label="应用用户密码">已设置（见上方表单）</el-descriptions-item>
            <el-descriptions-item label="安装用超级用户">{{ form.database.user }}</el-descriptions-item>
          </el-descriptions>

          <el-alert
            title="安装完成后，超级用户凭据不会被保存；应用数据库账号密码将写入 runtime/config.yaml。"
            type="info"
            show-icon
            :closable="false"
            class="install-alert"
          />

          <div class="install-submit">
            <el-button type="primary" size="large" :loading="submitting" @click="handleSubmit">
              开始安装
            </el-button>
          </div>
        </el-card>
      </el-form>

      <footer class="install-footer">
        <span>ApiNest · Open Source API Collaboration Platform</span>
      </footer>
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { markAppInstalled } from '@/router'
import { fetchInstallStatus, submitInstall, testDatabaseConnection } from '@/api/install'
import type { InstallCredentials, InstallPayload } from '@/types/install'

function generateRandomPassword(length = 24): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_'
  const bytes = new Uint8Array(length)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, (b) => chars[b % chars.length]).join('')
}

const validateAppDBUsername = (_rule: unknown, value: string, callback: (error?: Error) => void) => {
  if (!/^[a-z][a-z0-9_]*$/.test(value)) {
    callback(new Error('需以小写字母开头，仅包含小写字母、数字和下划线'))
    return
  }
  if (value === form.database.user) {
    callback(new Error('不能与超级用户名相同'))
    return
  }
  callback()
}

const router = useRouter()

const formRef = ref<FormInstance>()
const alreadyInstalled = ref(false)
const testing = ref(false)
const submitting = ref(false)
const installCredentials = ref<InstallCredentials | null>(null)

const form = reactive<InstallPayload>({
  site_url: typeof window !== 'undefined' ? window.location.origin : 'http://localhost',
  database: {
    driver: 'postgres',
    host: 'postgres',
    port: 5432,
    name: 'nest',
    user: 'postgres',
    password: '',
    ssl_mode: 'disable',
  },
  app_database: {
    username: 'nest',
    password: generateRandomPassword(),
  },
  admin: {
    username: 'admin',
    password: '',
    confirm_password: '',
  },
})

function regenerateAppPassword() {
  form.app_database.password = generateRandomPassword()
}

const validateConfirmPassword = (_rule: unknown, value: string, callback: (error?: Error) => void) => {
  if (value !== form.admin.password) {
    callback(new Error('两次输入的密码不一致'))
    return
  }
  callback()
}

const rules: FormRules = {
  site_url: [{ required: true, message: '请输入站点域名', trigger: 'blur' }],
  'database.driver': [{ required: true, message: '请选择数据库类型', trigger: 'change' }],
  'database.host': [{ required: true, message: '请输入主机地址', trigger: 'blur' }],
  'database.port': [{ required: true, message: '请输入端口', trigger: 'change' }],
  'database.name': [{ required: true, message: '请输入数据库名', trigger: 'blur' }],
  'database.user': [{ required: true, message: '请输入超级用户名', trigger: 'blur' }],
  'database.password': [{ required: true, message: '请输入超级用户密码', trigger: 'blur' }],
  'database.ssl_mode': [{ required: true, message: '请选择 SSL 模式', trigger: 'change' }],
  'app_database.username': [
    { required: true, message: '请输入应用数据库用户名', trigger: 'blur' },
    { min: 1, max: 63, message: '用户名长度不能超过 63 个字符', trigger: 'blur' },
    { validator: validateAppDBUsername, trigger: 'blur' },
  ],
  'app_database.password': [
    { required: true, message: '请输入应用数据库密码', trigger: 'blur' },
    { min: 6, message: '密码至少 6 位', trigger: 'blur' },
  ],
  'admin.username': [
    { required: true, message: '请输入管理员用户名', trigger: 'blur' },
    { min: 3, max: 50, message: '用户名长度为 3-50 个字符', trigger: 'blur' },
  ],
  'admin.password': [
    { required: true, message: '请输入登录密码', trigger: 'blur' },
    { min: 6, message: '密码至少 6 位', trigger: 'blur' },
  ],
  'admin.confirm_password': [
    { required: true, message: '请再次输入密码', trigger: 'blur' },
    { validator: validateConfirmPassword, trigger: 'blur' },
  ],
}

const allFields = [
  'site_url',
  'database.driver',
  'database.host',
  'database.port',
  'database.name',
  'database.user',
  'database.password',
  'database.ssl_mode',
  'app_database.username',
  'app_database.password',
  'admin.username',
  'admin.password',
  'admin.confirm_password',
] as const

const databaseFields = [
  'database.driver',
  'database.host',
  'database.port',
  'database.name',
  'database.user',
  'database.password',
  'database.ssl_mode',
] as const

async function validateFields(fields: readonly string[]) {
  if (!formRef.value) return false

  try {
    await formRef.value.validateField(fields as string[])
    return true
  } catch {
    return false
  }
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败，请手动复制')
  }
}

async function handleTestConnection() {
  const valid = await validateFields(databaseFields)
  if (!valid) return

  testing.value = true
  try {
    const result = await testDatabaseConnection(form.database)
    ElMessage.success(result.message || '数据库连接成功')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '连接失败')
  } finally {
    testing.value = false
  }
}

async function handleSubmit() {
  const valid = await validateFields(allFields)
  if (!valid) {
    ElMessage.warning('请检查表单填写是否完整')
    return
  }

  submitting.value = true
  try {
    await submitInstall(form)
    installCredentials.value = {
      admin: {
        username: form.admin.username,
        password: form.admin.password,
      },
      database: {
        username: form.app_database.username,
        password: form.app_database.password,
      },
    }
    markAppInstalled()
    ElMessage.success('安装成功')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '安装失败')
  } finally {
    submitting.value = false
  }
}

function goLogin() {
  markAppInstalled()
  router.push('/login')
}

onMounted(async () => {
  try {
    const status = await fetchInstallStatus()
    alreadyInstalled.value = status.installed
    if (status.installed) {
      markAppInstalled()
    }
  } catch {
    // 后端未启动时仍展示安装表单
  }
})
</script>

<style scoped>
.install-page {
  position: relative;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  color-scheme: light;
  --color-primary: #2563eb;
  --color-text: #1e293b;
  --color-text-secondary: #64748b;
  --color-border: #e2e8f0;
  --color-surface: #ffffff;
  --shadow-card: 0 12px 32px rgba(15, 23, 42, 0.08);
  /* Override global dark theme so Element Plus result text stays readable */
  --el-text-color-primary: #0f172a;
  --el-text-color-regular: #475569;
  --el-text-color-secondary: #64748b;
  color: var(--color-text);
  background: #f8fafc;
}

.install-page :deep(.el-card) {
  background-color: #ffffff;
  color: var(--color-text);
}

/* ── 文本输入框（不含下拉框） ── */
.install-form :deep(.el-input .el-input__wrapper) {
  background-color: #ffffff !important;
  box-shadow: 0 0 0 1px var(--color-border) inset !important;
}

.install-form :deep(.el-input .el-input__inner) {
  color: var(--color-text) !important;
  -webkit-text-fill-color: var(--color-text) !important;
}

.install-form :deep(.el-input .el-input__inner::placeholder) {
  color: var(--color-text-secondary) !important;
  -webkit-text-fill-color: var(--color-text-secondary) !important;
}

/* ── 下拉框：占位符浅色，选中值为深色 ── */
.install-form :deep(.el-select .el-select__placeholder.is-transparent) {
  color: var(--color-text-secondary);
}

.install-form :deep(.el-select .el-select__selected-item.el-select__placeholder:not(.is-transparent)) {
  color: var(--color-text) !important;
  -webkit-text-fill-color: var(--color-text) !important;
}

.install-form :deep(.el-input-number .el-input__wrapper) {
  background-color: #ffffff !important;
}

.install-page :deep(.el-descriptions__cell) {
  background: #ffffff;
  color: var(--color-text);
}

.install-page :deep(.el-descriptions__label) {
  background: #f8fafc;
  color: var(--color-text-secondary);
}

.install-page :deep(.el-result__title),
.install-page :deep(.el-result__title p) {
  color: var(--el-text-color-primary) !important;
}

.install-page :deep(.el-result__subtitle),
.install-page :deep(.el-result__subtitle p) {
  color: var(--el-text-color-regular) !important;
}

.install-bg {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background: #f8fafc;
}

.install-bg__orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.4;
}

.install-bg__orb--1 {
  width: 480px;
  height: 480px;
  background: #93c5fd;
  top: -160px;
  right: -100px;
}

.install-bg__orb--2 {
  width: 400px;
  height: 400px;
  background: #c4b5fd;
  bottom: -120px;
  left: -80px;
}

.install-topbar {
  position: sticky;
  top: 0;
  z-index: 100;
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--color-border);
}

.install-topbar__inner {
  max-width: 880px;
  margin: 0 auto;
  padding: 20px 24px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.install-logo {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  object-fit: contain;
  display: block;
}

.install-topbar__text h1 {
  margin: 0 0 4px;
  font-size: 22px;
  font-weight: 700;
  line-height: 1.3;
}

.install-topbar__text p {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.install-main {
  position: relative;
  z-index: 1;
  flex: 1;
  max-width: 880px;
  width: 100%;
  margin: 0 auto;
  padding: 32px 24px 40px;
}

.install-card,
.install-block {
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-card);
  background: #ffffff;
}

.install-block {
  margin-bottom: 20px;
}

.install-block :deep(.el-card__body) {
  padding: 28px 32px;
}

.install-block__head {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--color-border);
}

.install-block__index {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: #eff6ff;
  color: var(--color-primary);
  font-size: 14px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

.install-block__head h2 {
  margin: 0 0 6px;
  font-size: 18px;
  font-weight: 600;
}

.install-block__head p {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

.form-section {
  margin-bottom: 24px;
}

.form-section__title {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}

.form-section__desc {
  margin: -4px 0 12px;
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.form-section__desc code {
  padding: 1px 6px;
  border-radius: 4px;
  background: #f1f5f9;
  font-size: 12px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0 20px;
}

.form-grid__full {
  grid-column: 1 / -1;
}

.password-field {
  display: flex;
  gap: 8px;
  width: 100%;
}

.password-field :deep(.el-input) {
  flex: 1;
}

.password-field :deep(.install-action-btn.el-button) {
  --el-button-bg-color: #ffffff;
  --el-button-border-color: #e2e8f0;
  --el-button-text-color: #1e293b;
  --el-button-hover-bg-color: #f8fafc;
  --el-button-hover-border-color: #cbd5e1;
  --el-button-hover-text-color: #1e293b;
  --el-button-active-bg-color: #f1f5f9;
  --el-button-active-border-color: #cbd5e1;
  --el-button-active-text-color: #1e293b;
  background-color: #ffffff !important;
  border-color: #e2e8f0 !important;
  color: #1e293b !important;
}

.password-field :deep(.install-action-btn.el-button:hover),
.password-field :deep(.install-action-btn.el-button:focus) {
  background-color: #f8fafc !important;
  border-color: #cbd5e1 !important;
  color: #1e293b !important;
}

.install-form :deep(.el-form-item) {
  margin-bottom: 20px;
}

.install-form :deep(.el-form-item__label) {
  font-weight: 500;
  color: var(--color-text);
  padding-bottom: 6px;
  line-height: 1.4;
}

.install-form :deep(.el-select),
.install-form :deep(.el-input) {
  width: 100%;
}

.install-block__action {
  display: flex;
  justify-content: flex-end;
  padding-top: 4px;
}

.install-summary {
  margin-bottom: 20px;
}

.install-alert {
  margin-bottom: 24px;
}

.install-submit {
  display: flex;
  justify-content: center;
  padding-top: 4px;
}

.install-submit .el-button {
  min-width: 200px;
}

.install-success :deep(.el-result__extra) {
  width: 100%;
  max-width: 560px;
}

.install-credentials {
  width: 100%;
  text-align: left;
  margin-bottom: 24px;
}

.install-credentials__section {
  margin-bottom: 24px;
}

.install-credentials__section:last-child {
  margin-bottom: 0;
}

.install-credentials h3 {
  margin: 0 0 6px;
  font-size: 16px;
}

.install-credentials__hint {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.install-credentials__hint code {
  padding: 1px 6px;
  border-radius: 4px;
  background: #f1f5f9;
  font-size: 12px;
}

.install-credentials__row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  margin-bottom: 8px;
}

.install-credentials__row--clickable {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.install-credentials__row--clickable:hover {
  background: #f8fafc;
}

.install-credentials__label {
  flex-shrink: 0;
  width: 56px;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.install-credentials__row code {
  flex: 1;
  min-width: 0;
  padding: 0;
  background: transparent;
  font-size: 13px;
  word-break: break-all;
}

.install-footer {
  margin-top: 8px;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 13px;
}

@media (max-width: 640px) {
  .install-topbar__inner {
    padding: 16px;
  }

  .install-topbar__text h1 {
    font-size: 18px;
  }

  .install-main {
    padding: 20px 16px 32px;
  }

  .install-block :deep(.el-card__body) {
    padding: 20px;
  }

  .form-grid {
    grid-template-columns: 1fr;
  }

  .form-grid__full {
    grid-column: auto;
  }

  .install-summary :deep(.el-descriptions) {
    --el-descriptions-item-bordered-label-width: 100px;
  }

  .install-credentials__row {
    flex-wrap: wrap;
  }

  .install-credentials__row code {
    width: 100%;
  }
}
</style>

<style>
/* 弹出层外框（el-popper），非输入框本身 */
.install-popper.el-select__popper {
  border: none !important;
  box-shadow: none !important;
}

.install-popper.el-select-dropdown {
  background: #ffffff;
  border: none;
  box-shadow: none;
}

.install-popper .el-select-dropdown__item {
  color: #1e293b;
}

.install-popper .el-select-dropdown__item.is-hovering,
.install-popper .el-select-dropdown__item:hover {
  background: #f1f5f9;
}
</style>
