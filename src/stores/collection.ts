import { defineStore } from 'pinia'
import {
  CreateCollection, ListCollections, UpdateCollection, DeleteCollection,
  ListRequests, GetRequest, SaveRequest, DeleteRequest,
  models
} from '@/api/wails'
import { useTabsStore } from '@/stores/tabs'

export const useCollectionStore = defineStore('collection', {
  state: () => ({
    collections: [] as models.Collection[],
    requests: [] as models.Request[],
    selectedCollectionId: null as number | null,
  }),
  actions: {
    async loadCollections() {
      this.collections = await ListCollections()
    },
    async loadRequests() {
      this.requests = await ListRequests(null)
    },
    async loadAll() {
      await Promise.all([this.loadCollections(), this.loadRequests()])
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
      const doomed = new Set<number>()
      const collect = (cid: number) => {
        if (doomed.has(cid)) return
        doomed.add(cid)
        this.collections.filter(c => c.parent_id === cid).forEach(c => collect(c.id))
      }
      collect(id)
      await DeleteCollection(id)
      if (this.selectedCollectionId != null && doomed.has(this.selectedCollectionId)) {
        this.selectedCollectionId = null
      }
      // 联动关闭被删集合（含子集合）下所有已打开的请求选项卡
      const tabs = useTabsStore()
      tabs.tabs
        .filter(t => t.collection_id != null && doomed.has(t.collection_id))
        .map(t => t.id)
        .forEach(tid => tabs.closeTab(tid))
      await this.loadAll()
    },
    selectCollection(id: number | null) {
      this.selectedCollectionId = id
    },
    async addRequest(collectionId: number) {
      const payload: models.Request = {
        id: 0,
        collection_id: collectionId,
        name: '新建请求',
        method: 'GET',
        url: '',
        headers: [],
        params: [],
        body_type: 'none',
        raw_content_type: 'text',
        body: '',
        tests_script: '',
        auth: {
          auth_type: 'none',
          token: '',
          username: '',
          password: '',
          key: '',
          value: '',
          location: 'header',
        },
      }
      const saved = await SaveRequest(payload)
      this.selectedCollectionId = collectionId
      const tabs = useTabsStore()
      tabs.openTab(saved)
      // 标记为新建态：各 KV 编辑器默认显示一行空白行，首次保存后转为修改态
      tabs.newIds.push(saved.id)
      await this.loadRequests()
      return saved
    },
    async selectRequest(id: number) {
      const tabs = useTabsStore()
      // 已打开的请求直接激活，保留编辑中的内容
      if (tabs.tabs.some(t => t.id === id)) {
        tabs.activeTabId = id
        return
      }
      const req = await GetRequest(id)
      tabs.openTab(req)
      this.selectedCollectionId = req.collection_id ?? null
    },
    async renameRequest(id: number, name: string) {
      const req = await GetRequest(id)
      req.name = name
      await SaveRequest(req)
      useTabsStore().renameTab(id, name)
      await this.loadRequests()
    },
    async duplicateRequest(id: number, name?: string) {
      const original = await GetRequest(id)
      const duplicate = {
        ...original,
        id: 0,
        name: name?.trim() || `${original.name} (副本)`,
      }
      const saved = await SaveRequest(duplicate)
      useTabsStore().openTab(saved)
      this.selectedCollectionId = saved.collection_id ?? null
      await this.loadRequests()
      return saved
    },
    // 保存指定请求（来自某个选项卡的编辑副本），并同步选项卡与列表
    async saveRequest(req: models.Request) {
      const saved = await SaveRequest(req)
      useTabsStore().syncSaved(saved)
      this.selectedCollectionId = saved.collection_id ?? null
      await this.loadRequests()
      return saved
    },
    // 拖拽移动请求到目标集合；targetCollectionId 为 null 表示移到「未归类请求」
    async moveRequest(id: number, targetCollectionId: number | null) {
      const tabs = useTabsStore()
      // 优先使用已打开选项卡的编辑副本，避免拖拽覆盖未保存的修改
      const req = tabs.tabs.find(t => t.id === id) ?? await GetRequest(id)
      if (req.collection_id === targetCollectionId) return
      req.collection_id = targetCollectionId
      const saved = await SaveRequest(req)
      tabs.syncSaved(saved)
      await this.loadRequests()
      return saved
    },
    async removeRequest(id: number) {
      await DeleteRequest(id)
      useTabsStore().closeTab(id)
      await this.loadRequests()
    },
  },
})
