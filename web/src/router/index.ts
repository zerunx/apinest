import { createRouter, createWebHistory } from 'vue-router'
import { fetchInstallStatus } from '@/api/install'
import { fetchSiteInfo } from '@/api/auth'
import { getAccessToken } from '@/utils/auth-storage'
import { useAuth } from '@/composables/useAuth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/install',
      name: 'install',
      component: () => import('@/views/install/InstallView.vue'),
      meta: { title: '系统安装', public: true, installOnly: true },
    },
    {
      path: '/login',
      component: () => import('@/layouts/AuthLayout.vue'),
      meta: { public: true },
      children: [
        {
          path: '',
          name: 'login',
          component: () => import('@/views/auth/LoginView.vue'),
          meta: { title: '登录', guestOnly: true },
        },
      ],
    },
    {
      path: '/register',
      component: () => import('@/layouts/AuthLayout.vue'),
      meta: { public: true },
      children: [
        {
          path: '',
          name: 'register',
          component: () => import('@/views/auth/RegisterView.vue'),
          meta: { title: '注册', guestOnly: true },
        },
      ],
    },
    {
      path: '/invite',
      component: () => import('@/layouts/AuthLayout.vue'),
      meta: { public: true },
      children: [
        {
          path: '',
          name: 'invite',
          component: () => import('@/views/auth/InviteAcceptView.vue'),
          meta: { title: '加入工作空间' },
        },
      ],
    },
    {
      path: '/share',
      component: () => import('@/layouts/ShareLayout.vue'),
      meta: { public: true },
      children: [
        {
          path: '',
          name: 'share',
          component: () => import('@/views/share/ShareView.vue'),
          meta: { title: '项目分享' },
        },
      ],
    },
    {
      path: '/',
      component: () => import('@/layouts/AppLayout.vue'),
      meta: { requiresAuth: true },
      children: [
        {
          path: '',
          redirect: '/home',
        },
        {
          path: 'home',
          name: 'home',
          component: () => import('@/views/workspace/WorkspaceView.vue'),
          meta: { title: '工作空间' },
        },
        {
          path: 'admin/settings',
          name: 'admin-settings',
          components: {
            default: () => import('@/views/workspace/WorkspaceView.vue'),
            overlay: () => import('@/views/admin/SettingsView.vue'),
          },
          meta: { title: '系统设置', requiresAdmin: true, overlay: true },
        },
      ],
    },
  ],
})

let installChecked = false
let installed = false

async function loadInstallStatus() {
  try {
    const status = await fetchInstallStatus()
    installed = status.installed
  } catch {
    try {
      const site = await fetchSiteInfo()
      installed = site.installed
    } catch {
      installed = false
    }
  }
  installChecked = true
  return installed
}

/** 安装完成后更新缓存，避免仍被重定向回 /install */
export function markAppInstalled() {
  installed = true
  installChecked = true
}

router.beforeEach(async (to, _from, next) => {
  const title = to.meta.title as string | undefined
  document.title = title ? `${title} - ApiNest` : 'ApiNest'

  if (!installChecked) {
    await loadInstallStatus()
  }

  if (!installed) {
    if (to.path === '/install') {
      next()
      return
    }
    next('/install')
    return
  }

  if (to.path === '/install') {
    next('/login')
    return
  }

  const token = getAccessToken()
  const { bootstrap, user } = useAuth()
  if (token) {
    await bootstrap()
  }

  if (to.meta.guestOnly && token && user.value) {
    next('/home')
    return
  }

  if (to.meta.requiresAuth && !token) {
    next({ path: '/login', query: { redirect: to.fullPath } })
    return
  }

  if (to.meta.requiresAdmin && !user.value?.is_admin) {
    next('/home')
    return
  }

  next()
})

export default router
