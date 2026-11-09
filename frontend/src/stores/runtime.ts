import { defineStore } from 'pinia'
import { SendRequest, RunCollection, models } from '@/api/wails'

export const useRuntimeStore = defineStore('runtime', {
  state: () => ({
    response: null as models.Response | null,
    sending: false,
    testResults: [] as models.TestResult[],
    running: false,
  }),
  actions: {
    async send(req: models.Request) {
      this.sending = true
      try {
        this.response = await SendRequest(req)
      } finally {
        this.sending = false
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
    clearResponse() {
      this.response = null
    },
  },
})
