import { defineStore } from 'pinia'
import { models } from '@/api/wails'

// 请求选项卡：每个选项卡持有一份独立的编辑副本，互不影响。
// snapshots 记录每个选项卡最近一次保存的内容，用于标记「未保存」状态。
export const useTabsStore = defineStore('tabs', {
  state: () => ({
    tabs: [] as models.Request[],
    activeTabId: null as number | null,
    snapshots: {} as Record<number, string>,
  }),
  getters: {
    activeTab(state): models.Request | null {
      return state.tabs.find(t => t.id === state.activeTabId) ?? null
    },
  },
  actions: {
    // 打开或激活选项卡；已打开的请求只切换激活，不覆盖编辑中的内容
    openTab(req: models.Request) {
      if (!this.tabs.some(t => t.id === req.id)) {
        this.tabs.push(req)
        this.snapshots[req.id] = JSON.stringify(req)
      }
      this.activeTabId = req.id
    },
    isDirty(id: number): boolean {
      const tab = this.tabs.find(t => t.id === id)
      return !!tab && JSON.stringify(tab) !== this.snapshots[id]
    },
    // 保存成功后同步选项卡内容并清除未保存标记
    syncSaved(req: models.Request) {
      const tab = this.tabs.find(t => t.id === req.id)
      if (!tab) return
      Object.assign(tab, req)
      this.snapshots[req.id] = JSON.stringify(tab)
    },
    renameTab(id: number, name: string) {
      const tab = this.tabs.find(t => t.id === id)
      if (tab) {
        tab.name = name
        this.snapshots[id] = JSON.stringify(tab)
      }
    },
    closeTab(id: number) {
      const idx = this.tabs.findIndex(t => t.id === id)
      if (idx < 0) return
      this.tabs.splice(idx, 1)
      delete this.snapshots[id]
      if (this.activeTabId === id) {
        const next = this.tabs[Math.min(idx, this.tabs.length - 1)]
        this.activeTabId = next ? next.id : null
      }
    },
  },
})
