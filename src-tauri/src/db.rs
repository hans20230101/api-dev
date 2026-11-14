use rusqlite::{params, Connection};

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
	body_json TEXT NOT NULL DEFAULT '',
	body_form TEXT NOT NULL DEFAULT '',
	body_raw TEXT NOT NULL DEFAULT '',
	raw_content_type TEXT NOT NULL DEFAULT 'text',
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
    migrate_requests_body_columns(&conn)?;
    migrate_requests_raw_content_type(&conn)?;
    Ok(Db(conn))
}

/// 旧库升级：为 requests 补建 raw_content_type 列（新库建表时列已存在，ALTER 失败可忽略）
fn migrate_requests_raw_content_type(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('requests') WHERE name = 'raw_content_type'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .map_err(|e| e.to_string())?;
    if !exists {
        conn.execute(
            "ALTER TABLE requests ADD COLUMN raw_content_type TEXT NOT NULL DEFAULT 'text'",
            [],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 旧库升级：为 requests 补建 body_json/body_form/body_raw 列，并把已存的 body
/// 按 body_type 归入对应列（新库建表时列已存在，ALTER 失败可忽略）
fn migrate_requests_body_columns(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    for col in ["body_json", "body_form", "body_raw"] {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('requests') WHERE name = ?",
                params![col],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)
            .map_err(|e| e.to_string())?;
        if !exists {
            conn.execute(
                &format!("ALTER TABLE requests ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"),
                [],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    // 一次性归位：body 按 body_type 写入对应新列（列原本为空才写入，避免重复迁移覆盖）
    conn.execute(
        "UPDATE requests SET body_json = body WHERE body_type = 'json' AND body_json = '' AND body != ''",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE requests SET body_form = body WHERE body_type = 'form' AND body_form = '' AND body != ''",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE requests SET body_raw = body WHERE body_type = 'raw' AND body_raw = '' AND body != ''",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
