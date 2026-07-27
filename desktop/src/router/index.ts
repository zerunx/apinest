import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'main',
      component: () => import('../views/MainView.vue'),
    },
    {
      // Kept for old links; login is a dialog on the main shell.
      path: '/login',
      name: 'login',
      redirect: { name: 'main' },
    },
    {
      path: '/settings',
      redirect: { name: 'main' },
    },
  ],
})

export default router
