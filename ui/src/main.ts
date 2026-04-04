import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { router } from './router'
import './assets/styles/base.css'
import App from './App.vue'
import { useErrorStore } from './stores/error.store'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)

const errorStore = useErrorStore()

app.config.errorHandler = (err, instance, info) => {
  console.error("Vue Global Error:", err, info)
  errorStore.logError(err instanceof Error ? err.message : String(err), 'ui', info)
}

window.addEventListener('unhandledrejection', (event) => {
  console.error("Unhandled Promise Rejection:", event.reason)
  errorStore.logError(event.reason instanceof Error ? event.reason.message : String(event.reason), 'unknown', 'Promise Rejection')
})

app.mount('#app')
