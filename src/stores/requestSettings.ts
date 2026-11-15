import { defineStore } from 'pinia'
import { GetRequestTimeout, SetRequestTimeout } from '@/api/wails'

const STORAGE_KEY = 'app-request-timeout-secs'

/** 请求设置：超时时间（秒），后端默认 30s；本地缓存用于面板秒开 */
export const useRequestSettingsStore = defineStore('requestSettings', {
  state: () => ({
    timeoutSecs: Number(localStorage.getItem(STORAGE_KEY)) || 30,
    loaded: false,
  }),
  actions: {
    async load() {
      try {
        const v = await GetRequestTimeout()
        this.timeoutSecs = v
        localStorage.setItem(STORAGE_KEY, String(v))
        this.loaded = true
      } catch {
        // 后端不可用时保留本地缓存值
      }
    },
    async save(secs: number) {
      const t = Math.min(3600, Math.max(1, Math.round(secs)))
      this.timeoutSecs = t
      localStorage.setItem(STORAGE_KEY, String(t))
      await SetRequestTimeout(t)
    },
  },
})
