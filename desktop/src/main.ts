import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { setupI18n } from './i18n'
import './styles/global.css'
import './styles/interface-form.css'

document.addEventListener('contextmenu', (e) => {
  e.preventDefault()
})

function dismissBootSplash() {
  const splash = document.getElementById('app-boot-splash')
  if (!splash) return
  splash.classList.add('is-leaving')
  window.setTimeout(() => splash.remove(), 200)
}

function waitForSessionReady(timeoutMs = 12000) {
  return new Promise<void>((resolve) => {
    let settled = false
    const done = () => {
      if (settled) return
      settled = true
      resolve()
    }
    window.__NEST_MARK_SESSION_READY__ = done
    window.setTimeout(done, timeoutMs)
  })
}

async function bootstrap() {
  const sessionReady = waitForSessionReady()
  const app = createApp(App)
  await setupI18n(app)
  app.use(router)
  app.mount('#app')
  // Keep splash up until workspace tabs are restored so the empty home does not flash.
  await sessionReady
  requestAnimationFrame(() => dismissBootSplash())
}

void bootstrap()
