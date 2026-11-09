import { defineStore } from 'pinia'
import {
  ListEnvironments, CreateEnvironment, UpdateEnvironment, DeleteEnvironment,
  SetActiveEnvironment, GetActiveEnvironment,
  ListVariables, SaveVariable, DeleteVariable,
  models
} from '@/api/wails'

export const useEnvironmentStore = defineStore('environment', {
  state: () => ({
    environments: [] as models.Environment[],
    activeEnvId: null as number | null,
    variables: [] as models.Variable[],
  }),
  getters: {
    activeEnv(): models.Environment | null {
      return this.environments.find(e => e.id === this.activeEnvId) ?? null
    },
    varMap(): Record<string, string> {
      const m: Record<string, string> = {}
      for (const v of this.variables) m[v.key] = v.value
      return m
    },
  },
  actions: {
    async load() {
      this.environments = await ListEnvironments()
      const active = await GetActiveEnvironment()
      this.activeEnvId = active?.id ?? null
      if (this.activeEnvId != null) {
        this.variables = await ListVariables(this.activeEnvId)
      } else {
        this.variables = []
      }
    },
    async add(name: string) {
      const e = await CreateEnvironment(name)
      this.environments.push(e)
    },
    async rename(id: number, name: string) {
      await UpdateEnvironment(id, name)
      const e = this.environments.find(x => x.id === id)
      if (e) e.name = name
    },
    async remove(id: number) {
      await DeleteEnvironment(id)
      this.environments = this.environments.filter(x => x.id !== id)
      if (this.activeEnvId === id) {
        this.activeEnvId = null
        this.variables = []
      }
    },
    async setActive(id: number | null) {
      if (id == null) return
      await SetActiveEnvironment(id)
      this.activeEnvId = id
      this.variables = await ListVariables(id)
    },
    async loadVariables(envId: number) {
      this.variables = await ListVariables(envId)
    },
    async saveVariable(v: models.Variable) {
      const saved = await SaveVariable(v)
      const idx = this.variables.findIndex(x => x.id === saved.id)
      if (idx >= 0) this.variables[idx] = saved
      else this.variables.push(saved)
    },
    async removeVariable(id: number) {
      await DeleteVariable(id)
      this.variables = this.variables.filter(x => x.id !== id)
    },
  },
})
