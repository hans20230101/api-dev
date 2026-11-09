import { invoke } from '@tauri-apps/api/core'
import type { models } from './models'

// 统一封装对 Tauri 后端命令的调用，函数签名与原 Wails 绑定保持一致

export function ListCollections(): Promise<models.Collection[]> {
  return invoke('list_collections')
}

export function CreateCollection(name: string, parentId: number | null): Promise<models.Collection> {
  return invoke('create_collection', { name, parentId })
}

export function UpdateCollection(id: number, name: string): Promise<void> {
  return invoke('update_collection', { id, name })
}

export function DeleteCollection(id: number): Promise<void> {
  return invoke('delete_collection', { id })
}

export function ListRequests(collectionId: number | null): Promise<models.Request[]> {
  return invoke('list_requests', { collectionId })
}

export function GetRequest(id: number): Promise<models.Request> {
  return invoke('get_request', { id })
}

export function SaveRequest(request: models.Request): Promise<models.Request> {
  return invoke('save_request', { request })
}

export function DeleteRequest(id: number): Promise<void> {
  return invoke('delete_request', { id })
}

export function ListEnvironments(): Promise<models.Environment[]> {
  return invoke('list_environments')
}

export function CreateEnvironment(name: string): Promise<models.Environment> {
  return invoke('create_environment', { name })
}

export function UpdateEnvironment(id: number, name: string): Promise<void> {
  return invoke('update_environment', { id, name })
}

export function DeleteEnvironment(id: number): Promise<void> {
  return invoke('delete_environment', { id })
}

export function SetActiveEnvironment(id: number): Promise<void> {
  return invoke('set_active_environment', { id })
}

export function GetActiveEnvironment(): Promise<models.Environment | null> {
  return invoke('get_active_environment')
}

export function ListVariables(envId: number): Promise<models.Variable[]> {
  return invoke('list_variables', { envId })
}

export function SaveVariable(variable: models.Variable): Promise<models.Variable> {
  return invoke('save_variable', { variable })
}

export function DeleteVariable(id: number): Promise<void> {
  return invoke('delete_variable', { id })
}

export function SendRequest(request: models.Request): Promise<models.Response> {
  return invoke('send_request', { request })
}

export function SaveResponseBody(bodyId: number, path: string): Promise<void> {
  return invoke('save_response_body', { bodyId, path })
}

export function RunCollection(collectionId: number): Promise<models.TestResult[]> {
  return invoke('run_collection', { collectionId })
}

export function ExportAll(): Promise<string> {
  return invoke('export_all')
}

export function ExportCollection(collectionId: number): Promise<string> {
  return invoke('export_collection', { collectionId })
}

export function WriteExportFile(path: string, content: string): Promise<void> {
  return invoke('write_export_file', { path, content })
}

export function ImportAll(jsonStr: string): Promise<void> {
  return invoke('import_all', { jsonStr })
}

export type { models } from './models'
