import { defineStore } from 'pinia'
import { SyncThemeChecks } from '@/api/wails'

export type ThemeMode = 'light' | 'dark' | 'system'

const STORAGE_KEY = 'app-theme-mode'

let media: MediaQueryList | null = null

/** 把生效主题落到 <html class="dark"> 上，Element Plus 的暗色变量随之启用 */
function resolveEffective(mode: ThemeMode): boolean {
  if (mode === 'dark') return true
  if (mode === 'light') return false
  if (!media) media = window.matchMedia('(prefers-color-scheme: dark)')
  return media.matches
}

function apply(mode: ThemeMode) {
  const effective = resolveEffective(mode)
  document.documentElement.classList.toggle('dark', effective)
  // 随系统时保持监听，系统切换主题时实时跟随
  if (mode === 'system') {
    if (!media) media = window.matchMedia('(prefers-color-scheme: dark)')
    if (media && !media.onchange) {
      media.onchange = () => apply(storeMode())
    }
  }
}

function storeMode(): ThemeMode {
  return (localStorage.getItem(STORAGE_KEY) as ThemeMode) || 'system'
}

export const useThemeStore = defineStore('theme', {
  state: () => ({
    mode: storeMode() as ThemeMode,
  }),
  getters: {
    // 当前实际生效的明暗（供图标展示用）
    effectiveDark(): boolean {
      return this.mode === 'dark' || (this.mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
    },
  },
  actions: {
    init() {
      apply(this.mode)
    },
    setMode(mode: ThemeMode) {
      this.mode = mode
      localStorage.setItem(STORAGE_KEY, mode)
      apply(mode)
      // 同步原生菜单「设置→主题」勾选（后端仅勾选当前项，保证互斥单选）
      SyncThemeChecks(mode).catch(() => {})
    },
  },
})
