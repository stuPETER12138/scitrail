import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import QuickCapture from './QuickCapture.vue'
import router from './router'
import './styles/tokens.css'

// 速录小窗是独立窗口：Rust 侧以 index.html#/quick 打开，不走主应用的路由
if (window.location.hash.startsWith('#/quick')) {
  createApp(QuickCapture).mount('#app')
} else {
  const app = createApp(App)
  app.use(createPinia())
  app.use(router)
  app.mount('#app')
}
