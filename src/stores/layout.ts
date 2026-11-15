import { defineStore } from 'pinia'

// 请求/响应区域的垂直分割比例（请求区域占比，百分比）。
// 默认 40% 请求、60% 响应；数值持久化到本地，用户拖动后下次打开沿用。
const STORAGE_KEY = 'app-split-request-pct'

function loadPct(): number {
  const raw = Number(localStorage.getItem(STORAGE_KEY))
  if (Number.isFinite(raw) && raw > 0 && raw < 100) return raw
  return 40
}

export const useLayoutStore = defineStore('layout', {
  state: () => ({
    // 请求区域高度占比（0~100）
    requestPct: loadPct(),
    splitDragging: false,
  }),
  actions: {
    setRequestPct(pct: number) {
      const clamped = Math.min(95, Math.max(5, pct))
      this.requestPct = clamped
      localStorage.setItem(STORAGE_KEY, String(clamped))
    },
    setSplitDragging(v: boolean) {
      this.splitDragging = v
    },
  },
})