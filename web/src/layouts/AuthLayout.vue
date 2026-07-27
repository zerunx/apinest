<template>
  <div class="auth-page">
    <div class="auth-page__left">
      <div class="auth-page__brand">
        <div class="auth-page__header">
          <div class="auth-page__mark">
            <img class="auth-page__logo app-logo" src="/nest.png" alt="" />
          </div>
          <h1 class="auth-page__name">ApiNest</h1>
        </div>
        <p class="auth-page__slogan">设计 · 调试 · 协作 · 交付</p>
      </div>
    </div>

    <div class="auth-page__right">
      <div class="auth-panel">
        <router-view />
      </div>
    </div>
  </div>
</template>

<style scoped>
.auth-page {
  min-height: 100vh;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(420px, 520px);
  color-scheme: light;
  --color-text: #1e293b;
  --color-text-secondary: #64748b;
  --el-text-color-primary: #0f172a;
  --el-text-color-regular: #475569;
  --el-text-color-placeholder: #94a3b8;
  --el-input-text-color: #1e293b;
}

.auth-page__left {
  position: relative;
  background:
    radial-gradient(circle at 20% 20%, rgba(37, 99, 235, 0.28), transparent 40%),
    radial-gradient(circle at 80% 70%, rgba(124, 58, 237, 0.22), transparent 35%),
    linear-gradient(135deg, #0f172a, #1e293b 55%, #0f172a);
  color: #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 48px;
}

.auth-page__brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  text-align: center;
}

.auth-page__header {
  display: flex;
  align-items: center;
  gap: 16px;
}

.auth-page__mark {
  flex-shrink: 0;
  width: 52px;
  height: 52px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.96);
  box-shadow:
    0 12px 28px rgba(15, 23, 42, 0.24),
    inset 0 0 0 1px rgba(255, 255, 255, 0.65);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px 8px 8px;
}

.auth-page__logo {
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
}

.auth-page__name {
  margin: 0;
  font-size: 34px;
  font-weight: 800;
  letter-spacing: -0.03em;
  line-height: 1;
  background: linear-gradient(135deg, #ffffff 0%, #dbeafe 55%, #93c5fd 100%);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.auth-page__slogan {
  margin: 0;
  color: #64748b;
  font-size: 14px;
  letter-spacing: 0.28em;
}

.auth-page__right {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 24px;
  background: #ffffff;
  color: #1e293b;
}

.auth-panel {
  width: 100%;
  max-width: 380px;
}

.auth-page :deep(.el-input__wrapper),
.auth-page :deep(.el-select__wrapper) {
  background-color: #ffffff !important;
  box-shadow: 0 0 0 1px #e2e8f0 inset !important;
}

.auth-page :deep(.el-input__inner),
.auth-page :deep(.el-input input),
.auth-page :deep(.el-input__inner:not(:placeholder-shown)),
.auth-page :deep(.el-input input:not(:placeholder-shown)),
.auth-page :deep(.el-select__selected-item),
.auth-page :deep(.el-select__placeholder) {
  color: #1e293b !important;
  -webkit-text-fill-color: #1e293b !important;
}

.auth-page :deep(.el-input__inner:placeholder-shown),
.auth-page :deep(.el-input input:placeholder-shown) {
  color: #94a3b8 !important;
  -webkit-text-fill-color: #94a3b8 !important;
}

.auth-page :deep(.el-input__inner::placeholder),
.auth-page :deep(.el-input input::placeholder),
.auth-page :deep(.el-input__inner::-webkit-input-placeholder),
.auth-page :deep(.el-input input::-webkit-input-placeholder) {
  color: #94a3b8 !important;
  -webkit-text-fill-color: #94a3b8 !important;
  opacity: 1;
}

.auth-page :deep(.el-form-item__label) {
  color: #334155 !important;
}

@media (max-width: 900px) {
  .auth-page {
    grid-template-columns: 1fr;
  }

  .auth-page__left {
    display: none;
  }
}
</style>
