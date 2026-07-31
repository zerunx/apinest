import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { setupI18n } from './i18n'
import './styles/global.css'
import './styles/interface-form.css'

// Disable WebView default context menu; custom menus use @contextmenu.prevent themselves.
document.addEventListener('contextmenu', (e) => {
  e.preventDefault()
})

async function bootstrap() {
  const app = createApp(App)
  await setupI18n(app)
  app.use(router)
  app.mount('#app')
}

void bootstrap()
