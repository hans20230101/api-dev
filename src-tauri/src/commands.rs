use crate::db::{init, Db};
use crate::models::{Collection, Environment, ExportData, Request, Response, TestResult, Variable};
use crate::repository::{env_repo, request_repo};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use tauri::{Manager, State};

pub struct AppState {
    pub db: Mutex<Db>,
    pub data_dir: std::path::PathBuf,
    /// 最近响应的原始字节缓存，供「保存文件」写入磁盘
    pub response_cache: Mutex<ResponseCache>,
}

type Guard<'a> = std::sync::MutexGuard<'a, Db>;

/// 响应字节缓存：按发送顺序淘汰最旧的记录，避免内存无界增长
pub struct ResponseCache {
    next_id: u64,
    entries: HashMap<u64, Vec<u8>>,
    order: VecDeque<u64>,
}

const RESPONSE_CACHE_MAX: usize = 20;

impl ResponseCache {
    fn new() -> Self {
        Self {
            next_id: 1,
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn insert(&mut self, bytes: Vec<u8>) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == 0 {
            self.next_id = 1;
        }
        self.entries.insert(id, bytes);
        self.order.push_back(id);
        while self.order.len() > RESPONSE_CACHE_MAX {
            if let Some(old) = self.order.pop_front() {
                self.entries.remove(&old);
            }
        }
        id
    }

    fn get(&self, id: u64) -> Option<&Vec<u8>> {
        self.entries.get(&id)
    }
}

fn db_lock<'a>(state: &'a State<AppState>) -> Result<Guard<'a>, String> {
    state.db.lock().map_err(|e| format!("数据库锁错误: {e}"))
}

/// 返回已解析的数据库目录（供窗口 WebView 数据目录回退使用）
pub fn data_dir(app: &tauri::App) -> std::path::PathBuf {
    app.state::<AppState>().data_dir.clone()
}

// ===== 集合 =====

#[tauri::command]
pub fn list_collections(state: State<AppState>) -> Result<Vec<Collection>, String> {
    let guard = db_lock(&state)?;
    env_repo::list_collections(&guard.0)
}

#[tauri::command]
pub fn create_collection(
    state: State<AppState>,
    name: String,
    parent_id: Option<i64>,
) -> Result<Collection, String> {
    let guard = db_lock(&state)?;
    env_repo::create_collection(&guard.0, &name, parent_id)
}

#[tauri::command]
pub fn update_collection(state: State<AppState>, id: i64, name: String) -> Result<(), String> {
    let guard = db_lock(&state)?;
    env_repo::update_collection(&guard.0, id, &name)
}

#[tauri::command]
pub fn delete_collection(state: State<AppState>, id: i64) -> Result<(), String> {
    let mut guard = db_lock(&state)?;
    env_repo::delete_collection(&mut guard.0, id)
}

// ===== 请求 =====

#[tauri::command]
pub fn list_requests(
    state: State<AppState>,
    collection_id: Option<i64>,
) -> Result<Vec<Request>, String> {
    let guard = db_lock(&state)?;
    request_repo::list_requests(&guard.0, collection_id)
}

#[tauri::command]
pub fn get_request(state: State<AppState>, id: i64) -> Result<Request, String> {
    let guard = db_lock(&state)?;
    request_repo::get_request(&guard.0, id)
}

#[tauri::command]
pub fn save_request(state: State<AppState>, request: Request) -> Result<Request, String> {
    let guard = db_lock(&state)?;
    request_repo::save_request(&guard.0, request)
}

#[tauri::command]
pub fn delete_request(state: State<AppState>, id: i64) -> Result<(), String> {
    let guard = db_lock(&state)?;
    request_repo::delete_request(&guard.0, id)
}

// ===== 环境 =====

#[tauri::command]
pub fn list_environments(state: State<AppState>) -> Result<Vec<Environment>, String> {
    let guard = db_lock(&state)?;
    env_repo::list_environments(&guard.0)
}

#[tauri::command]
pub fn create_environment(state: State<AppState>, name: String) -> Result<Environment, String> {
    let guard = db_lock(&state)?;
    env_repo::create_environment(&guard.0, &name)
}

#[tauri::command]
pub fn update_environment(state: State<AppState>, id: i64, name: String) -> Result<(), String> {
    let guard = db_lock(&state)?;
    env_repo::update_environment(&guard.0, id, &name)
}

#[tauri::command]
pub fn delete_environment(state: State<AppState>, id: i64) -> Result<(), String> {
    let guard = db_lock(&state)?;
    env_repo::delete_environment(&guard.0, id)
}

#[tauri::command]
pub fn set_active_environment(state: State<AppState>, id: i64) -> Result<(), String> {
    let mut guard = db_lock(&state)?;
    env_repo::set_active_environment(&mut guard.0, id)
}

#[tauri::command]
pub fn get_active_environment(state: State<AppState>) -> Result<Option<Environment>, String> {
    let guard = db_lock(&state)?;
    env_repo::get_active_environment(&guard.0)
}

// ===== 变量 =====

#[tauri::command]
pub fn list_variables(state: State<AppState>, env_id: i64) -> Result<Vec<Variable>, String> {
    let guard = db_lock(&state)?;
    env_repo::list_variables(&guard.0, env_id)
}

#[tauri::command]
pub fn save_variable(state: State<AppState>, variable: Variable) -> Result<Variable, String> {
    let guard = db_lock(&state)?;
    env_repo::save_variable(&guard.0, variable)
}

#[tauri::command]
pub fn delete_variable(state: State<AppState>, id: i64) -> Result<(), String> {
    let guard = db_lock(&state)?;
    env_repo::delete_variable(&guard.0, id)
}

// ===== 发送与测试 =====

/// 发送单个请求（自动使用当前激活环境的变量）
#[tauri::command]
pub fn send_request(state: State<AppState>, request: Request) -> Result<Response, String> {
    let guard = db_lock(&state)?;
    let vars = env_repo::get_active_variables(&guard.0)?;
    let (mut resp, bytes) = client::send(&request, &vars);
    if resp.error.is_none() {
        let mut cache = state
            .response_cache
            .lock()
            .map_err(|e| format!("响应缓存锁错误: {e}"))?;
        resp.body_id = Some(cache.insert(bytes));
    }
    Ok(resp)
}

/// 将缓存的响应原始字节写入用户选择的文件（后端写文件无路径权限限制）
#[tauri::command]
pub fn save_response_body(
    state: State<AppState>,
    body_id: u64,
    path: String,
) -> Result<(), String> {
    let bytes = {
        let cache = state
            .response_cache
            .lock()
            .map_err(|e| format!("响应缓存锁错误: {e}"))?;
        match cache.get(body_id) {
            Some(b) => b.clone(),
            None => return Err("响应内容已失效，请重新发送请求后再保存".to_string()),
        }
    };
    std::fs::write(&path, bytes).map_err(|e| format!("写入文件失败: {e}"))
}

/// 批量运行某集合下全部请求并返回测试结果
#[tauri::command]
pub fn run_collection(
    state: State<AppState>,
    collection_id: i64,
) -> Result<Vec<TestResult>, String> {
    let guard = db_lock(&state)?;
    let reqs = request_repo::list_requests(&guard.0, Some(collection_id))?;
    if reqs.is_empty() {
        return Ok(Vec::new());
    }
    let vars = env_repo::get_active_variables(&guard.0)?;
    Ok(testrunner::run_collection(&reqs, &vars))
}

// ===== 导入导出 =====

/// 导出全部数据为 JSON 字符串
#[tauri::command]
pub fn export_all(state: State<AppState>) -> Result<String, String> {
    let guard = db_lock(&state)?;
    let collections = env_repo::list_collections(&guard.0)?;
    let requests = request_repo::list_requests(&guard.0, None)?;
    let environments = env_repo::list_environments(&guard.0)?;
    let mut variables = Vec::new();
    for e in &environments {
        variables.extend(env_repo::list_variables(&guard.0, e.id)?);
    }
    let data = ExportData {
        collections,
        requests,
        environments,
        variables,
    };
    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

/// 导出单个顶层集合（包含其所有子集合和请求）
#[tauri::command]
pub fn export_collection(state: State<AppState>, collection_id: i64) -> Result<String, String> {
    let guard = db_lock(&state)?;
    let all_collections = env_repo::list_collections(&guard.0)?;

    // 递归收集该集合及其所有子集合
    let mut target_collections = Vec::new();
    collect_collections_recursive(collection_id, &all_collections, &mut target_collections);

    // 收集这些集合下的所有请求
    let mut requests = Vec::new();
    for coll in &target_collections {
        let reqs = request_repo::list_requests(&guard.0, Some(coll.id))?;
        requests.extend(reqs);
    }

    let data = ExportData {
        collections: target_collections,
        requests,
        environments: Vec::new(),
        variables: Vec::new(),
    };
    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

/// 从 JSON 字符串导入数据（按现有结构重建，变量随环境重建后 ID 变化暂不迁移）
#[tauri::command]
pub fn import_all(state: State<AppState>, json_str: String) -> Result<(), String> {
    let data: ExportData =
        serde_json::from_str(&json_str).map_err(|e| format!("JSON 解析失败: {e}"))?;
    let guard = db_lock(&state)?;
    for c in &data.collections {
        env_repo::create_collection(&guard.0, &c.name, c.parent_id)?;
    }
    for r in &data.requests {
        request_repo::save_request(&guard.0, r.clone())?;
    }
    for e in &data.environments {
        env_repo::create_environment(&guard.0, &e.name)?;
    }
    Ok(())
}

/// 将导出的 JSON 写入用户通过保存对话框选择的文件（后端写文件无路径权限限制）
#[tauri::command]
pub fn write_export_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {e}"))
}

use crate::httpclient::{client, testrunner};

/// 应用启动：初始化 SQLite 数据库。
/// 优先使用系统应用数据目录，不可写时回退到 exe 旁 / 工作目录下的 db/。
pub fn setup(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let preferred = app.path().app_data_dir().ok();
    let data_dir = crate::db::resolve_data_dir(preferred);
    let db = init(&data_dir)?;
    app.manage(AppState {
        db: Mutex::new(db),
        data_dir,
        response_cache: Mutex::new(ResponseCache::new()),
    });
    Ok(())
}

/// 仅供探测/测试使用的数据库初始化入口
pub fn init_db_for_test(
    dir: &std::path::Path,
) -> Result<Db, Box<dyn std::error::Error>> {
    crate::db::init(dir)
}

/// 递归收集指定集合及其所有子集合
fn collect_collections_recursive(
    collection_id: i64,
    all_collections: &[Collection],
    result: &mut Vec<Collection>,
) {
    if let Some(coll) = all_collections.iter().find(|c| c.id == collection_id) {
        result.push(coll.clone());
        // 递归收集所有子集合
        for child in all_collections.iter().filter(|c| c.parent_id == Some(collection_id)) {
            collect_collections_recursive(child.id, all_collections, result);
        }
    }
}
