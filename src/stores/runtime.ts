import { defineStore } from 'pinia'
import { SendRequest, CancelRequest, RunCollection, models } from '@/api/wails'

export const useRuntimeStore = defineStore('runtime', {
  state: () => ({
    // 响应按请求 id 隔离，多个选项卡各自的响应互不覆盖
    responses: {} as Record<number, models.Response>,
    sendingIds: [] as number[],
    testResults: [] as models.TestResult[],
    running: false,
  }),
  getters: {
    // 该请求是否正在发送（支持多选项卡并行发送）
    sending: (state) => (id: number) => state.sendingIds.includes(id),
  },
  actions: {
    async send(req: models.Request) {
      this.sendingIds.push(req.id)
      try {
        this.responses[req.id] = await SendRequest(req)
      } finally {
        this.sendingIds = this.sendingIds.filter(x => x !== req.id)
      }
    },
    // 取消进行中的请求：后端断开连接并以「请求已取消」响应返回
    async cancel(id: number) {
      try {
        await CancelRequest(id)
      } catch {
        // 请求恰好已完成时无需处理
      }
    },
    async runCollection(collectionId: number) {
      this.running = true
      try {
        this.testResults = await RunCollection(collectionId)
      } finally {
        this.running = false
      }
    },
    clearResponse(id: number) {
      delete this.responses[id]
    },
  },
})
