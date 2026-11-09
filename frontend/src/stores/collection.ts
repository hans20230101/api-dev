import { defineStore } from 'pinia'
import {
  CreateCollection, ListCollections, UpdateCollection, DeleteCollection,
  ListRequests, GetRequest, SaveRequest, DeleteRequest,
  models
} from '@/api/wails'

export const useCollectionStore = defineStore('collection', {
  state: () => ({
    collections: [] as models.Collection[],
    requests: [] as models.Request[],
    selectedCollectionId: null as number | null,
    currentRequest: null as models.Request | null,
  }),
  actions: {
    async loadCollections() {
      this.collections = await ListCollections()
    },
    async addCollection(name: string, parentId: number | null) {
      const c = await CreateCollection(name, parentId)
      this.collections.push(c)
    },
    async renameCollection(id: number, name: string) {
      await UpdateCollection(id, name)
      const c = this.collections.find(x => x.id === id)
      if (c) c.name = name
    },
    async removeCollection(id: number) {
      await DeleteCollection(id)
      this.collections = this.collections.filter(x => x.id !== id)
      if (this.selectedCollectionId === id) {
        this.selectedCollectionId = null
        this.requests = []
        this.currentRequest = null
      }
    },
    async selectCollection(id: number | null) {
      this.selectedCollectionId = id
      this.requests = id ? await ListRequests(id) : []
      this.currentRequest = null
    },
    newRequest() {
      this.currentRequest = {
        name: '新建请求',
        method: 'GET',
        url: '',
        headers: [],
        params: [],
        body_type: 'none',
        body: '',
        tests_script: '',
        collection_id: this.selectedCollectionId,
      } as unknown as models.Request
    },
    async selectRequest(id: number) {
      this.currentRequest = await GetRequest(id)
    },
    async saveCurrent() {
      if (!this.currentRequest) return null
      const saved = await SaveRequest(this.currentRequest)
      this.currentRequest = saved
      if (this.selectedCollectionId != null) {
        this.requests = await ListRequests(this.selectedCollectionId)
      }
      return saved
    },
    async removeRequest(id: number) {
      await DeleteRequest(id)
      this.requests = this.requests.filter(x => x.id !== id)
      if (this.currentRequest?.id === id) this.currentRequest = null
    },
  },
})
