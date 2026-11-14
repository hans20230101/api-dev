import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import App from './App.vue'
import './style.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import { useThemeStore } from './stores/theme'

const app = createApp(App)
app.use(createPinia())
app.use(ElementPlus)
// 全局注册 Element Plus 图标
for (const [key, comp] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, comp as any)
}
app.mount('#app')

// 挂载后应用持久化的主题模式（白天/黑夜/随系统）
useThemeStore().init()

// 默认禁用 WebView 右键菜单（保持桌面应用观感；F12 调试入口见「设置→调试」）
window.addEventListener('contextmenu', e => e.preventDefault())
