export namespace models {
  export interface KV {
    key: string
    value: string
    enable: boolean
    /** form-data 行是否为文件项（value 存文件路径） */
    file?: boolean
  }

  export interface Collection {
    id: number
    parent_id?: number | null
    name: string
    created_at?: string | null
  }

  export interface Environment {
    id: number
    name: string
    is_active: boolean
    created_at?: string | null
  }

  export interface Request {
    id: number
    collection_id?: number | null
    name: string
    method: string
    url: string
    headers: KV[]
    params: KV[]
    body_type: string
    /** 当前类型的活动数据（form-data/none），或 json/form/raw 隔离数据的镜像 */
    body: string
    /** json 类型的请求体（按类型隔离） */
    body_json?: string
    /** form 类型的请求体（按类型隔离） */
    body_form?: string
    /** raw 类型的请求体（按类型隔离） */
    body_raw?: string
    /** raw 类型的内容格式（text/json/xml/html/javascript），决定发送时的 Content-Type */
    raw_content_type?: string
    tests_script: string
    created_at?: string | null
  }

  export interface Variable {
    id: number
    environment_id: number
    key: string
    value: string
    type: string
  }

  export interface Response {
    status_code: number
    status: string
    time_ms: number
    req_size: number
    resp_size: number
    headers: Record<string, string>
    body: string
    body_id?: number
    content_type?: string
    is_binary: boolean
    filename?: string
    error?: string
  }

  export interface TestResult {
    request_id: number
    name: string
    passed: boolean
    status_code: number
    time_ms: number
    asserts: number
    failed_msg?: string
    error?: string
  }
}
