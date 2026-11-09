use rusqlite::Connection;

pub struct Db(pub Connection);

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS collections (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	parent_id INTEGER REFERENCES collections(id) ON DELETE CASCADE,
	name TEXT NOT NULL,
	created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS requests (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL,
	name TEXT NOT NULL DEFAULT '',
	method TEXT NOT NULL DEFAULT 'GET',
	url TEXT NOT NULL DEFAULT '',
	headers TEXT NOT NULL DEFAULT '[]',
	params TEXT NOT NULL DEFAULT '[]',
	body_type TEXT NOT NULL DEFAULT 'none',
	body TEXT NOT NULL DEFAULT '',
	tests_script TEXT NOT NULL DEFAULT '',
	created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS environments (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL,
	is_active INTEGER NOT NULL DEFAULT 0,
	created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS variables (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	environment_id INTEGER NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
	key TEXT NOT NULL,
	value TEXT NOT NULL DEFAULT '',
	type TEXT NOT NULL DEFAULT 'string'
);
"#;

/// 判断目录是否真的可写（有些受限环境 exists() 为真但写入被拒）
pub fn is_writable(dir: &std::path::Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".write-probe");
    let ok = std::fs::write(&probe, b"x").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

/// 解析数据目录：优先系统应用数据目录，不可用时回退到可执行文件旁的 db/，
/// 再回退到当前工作目录下的 db/。保证「应用总能启动」。
pub fn resolve_data_dir(preferred: Option<std::path::PathBuf>) -> std::path::PathBuf {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(p) = preferred {
        candidates.push(p);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("db"));
        }
    }
    candidates.push(std::path::PathBuf::from("db"));
    candidates.push(std::env::temp_dir().join("api-dev-db"));

    for dir in candidates.iter() {
        if is_writable(dir) {
            return dir.clone();
        }
    }
    // 全部失败时仍返回首选路径，让后续错误信息更直观
    candidates
        .first()
        .cloned()
        .unwrap_or_else(|| std::path::PathBuf::from("db"))
}

pub fn init(data_dir: &std::path::Path) -> Result<Db, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("data.db");

    // 迁移旧版（Wails）数据库：若目标不存在而旧 db/data.db 存在，则拷贝过来
    if !db_path.exists() {
        let legacy = std::path::PathBuf::from("db").join("data.db");
        if legacy.exists() && legacy != db_path {
            let _ = std::fs::copy(&legacy, &db_path);
        }
    }

    let conn = Connection::open(&db_path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch(SCHEMA)?;
    Ok(Db(conn))
}
