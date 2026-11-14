use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kv {
    pub key: String,
    pub value: String,
    pub enable: bool,
    /// form-data 行是否为文件项（value 存文件路径），旧数据无此字段时默认 None
    #[serde(default)]
    pub file: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: i64,
    pub collection_id: Option<i64>,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<Kv>,
    pub params: Vec<Kv>,
    pub body_type: String,
    /// 当前类型（form-data 或未隔离类型）的活动数据；json/form/raw 的内容存于专用字段
    pub body: String,
    /// json 类型的请求体（按类型隔离）
    #[serde(default)]
    pub body_json: String,
    /// form 类型的请求体（KV 数组 JSON，按类型隔离）
    #[serde(default)]
    pub body_form: String,
    /// raw 类型的请求体（按类型隔离）
    #[serde(default)]
    pub body_raw: String,
    /// raw 类型的内容格式（text/json/xml/html/javascript），决定发送时的 Content-Type
    #[serde(default = "default_raw_content_type")]
    pub raw_content_type: String,
    pub tests_script: String,
    pub created_at: Option<String>,
}

/// raw 请求体默认按纯文本发送（旧数据无此字段时使用）
fn default_raw_content_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: i64,
    pub environment_id: i64,
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub var_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status_code: u16,
    pub status: String,
    pub time_ms: i64,
    pub req_size: usize,
    pub resp_size: usize,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
    /// 原始响应字节在缓存中的句柄，供「保存文件」使用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// 响应是否为二进制内容（此时 body 为空，需保存文件查看）
    pub is_binary: bool,
    /// 从 Content-Disposition 解析出的文件名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub request_id: i64,
    pub name: String,
    pub passed: bool,
    pub status_code: u16,
    pub time_ms: i64,
    pub asserts: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub collections: Vec<Collection>,
    pub requests: Vec<Request>,
    pub environments: Vec<Environment>,
    pub variables: Vec<Variable>,
}
