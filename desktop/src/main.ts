import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { setupI18n } from './i18n'
import './styles/global.css'
import './styles/interface-form.css'

const app = createApp(App)
setupI18n(app)
app.use(router)
app.mount('#app')
