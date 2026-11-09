use crate::models::{Collection, Environment, Variable};
use rusqlite::{params, Connection, OptionalExtension};

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ===== 集合 =====

pub fn list_collections(conn: &Connection) -> Result<Vec<Collection>, String> {
    let mut stmt = conn
        .prepare("SELECT id, parent_id, name, created_at FROM collections ORDER BY name ASC")
        .map_err(|e| e.to_string())?;
    let list = stmt
        .query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get::<_, Option<String>>(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(list)
}

pub fn create_collection(
    conn: &Connection,
    name: &str,
    parent_id: Option<i64>,
) -> Result<Collection, String> {
    conn.execute(
        "INSERT INTO collections (parent_id, name) VALUES (?, ?)",
        params![parent_id, name],
    )
    .map_err(|e| e.to_string())?;
    Ok(Collection {
        id: conn.last_insert_rowid(),
        parent_id: parent_id,
        name: name.to_string(),
        created_at: Some(now_str()),
    })
}

pub fn update_collection(conn: &Connection, id: i64, name: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE collections SET name = ? WHERE id = ?",
        params![name, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_collection(conn: &mut Connection, id: i64) -> Result<(), String> {
    // 级联删除：该集合及其所有子集合、以及这些集合下的全部请求都会被删除
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut ids = Vec::new();
    collect_descendant_ids(&tx, id, &mut ids)?;
    for cid in &ids {
        tx.execute("DELETE FROM requests WHERE collection_id = ?", params![cid])
            .map_err(|e| e.to_string())?;
    }
    // 先删子集合再删父集合，避免外键约束
    for cid in ids.iter().rev() {
        tx.execute("DELETE FROM collections WHERE id = ?", params![cid])
            .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}

/// 收集指定集合及其所有后代集合的 id（父在前、子在后）
fn collect_descendant_ids(conn: &Connection, id: i64, out: &mut Vec<i64>) -> Result<(), String> {
    out.push(id);
    let mut stmt = conn
        .prepare("SELECT id FROM collections WHERE parent_id = ?")
        .map_err(|e| e.to_string())?;
    let children: Vec<i64> = stmt
        .query_map(params![id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    for child in children {
        collect_descendant_ids(conn, child, out)?;
    }
    Ok(())
}

// ===== 环境 =====

pub fn list_environments(conn: &Connection) -> Result<Vec<Environment>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, is_active, created_at FROM environments ORDER BY name ASC")
        .map_err(|e| e.to_string())?;
    let list = stmt
        .query_map([], |row| {
            Ok(Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                is_active: row.get::<_, i64>(2)? == 1,
                created_at: row.get::<_, Option<String>>(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(list)
}

pub fn create_environment(conn: &Connection, name: &str) -> Result<Environment, String> {
    conn.execute("INSERT INTO environments (name) VALUES (?)", params![name])
        .map_err(|e| e.to_string())?;
    Ok(Environment {
        id: conn.last_insert_rowid(),
        name: name.to_string(),
        is_active: false,
        created_at: Some(now_str()),
    })
}

pub fn update_environment(conn: &Connection, id: i64, name: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE environments SET name = ? WHERE id = ?",
        params![name, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_environment(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM environments WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn set_active_environment(conn: &mut Connection, id: i64) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("UPDATE environments SET is_active = 0", [])
        .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE environments SET is_active = 1 WHERE id = ?",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub fn get_active_environment(conn: &Connection) -> Result<Option<Environment>, String> {
    conn.query_row(
        "SELECT id, name, is_active, created_at FROM environments WHERE is_active = 1 LIMIT 1",
        [],
        |row| {
            Ok(Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                is_active: row.get::<_, i64>(2)? == 1,
                created_at: row.get::<_, Option<String>>(3)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

// ===== 变量 =====

pub fn list_variables(conn: &Connection, env_id: i64) -> Result<Vec<Variable>, String> {
    let mut stmt = conn
        .prepare("SELECT id, environment_id, key, value, type FROM variables WHERE environment_id = ? ORDER BY key ASC")
        .map_err(|e| e.to_string())?;
    let list = stmt
        .query_map(params![env_id], |row| {
            Ok(Variable {
                id: row.get(0)?,
                environment_id: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                var_type: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(list)
}

pub fn save_variable(conn: &Connection, mut v: Variable) -> Result<Variable, String> {
    if v.id > 0 {
        conn.execute(
            "UPDATE variables SET environment_id=?, key=?, value=?, type=? WHERE id=?",
            params![v.environment_id, v.key, v.value, v.var_type, v.id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO variables (environment_id, key, value, type) VALUES (?,?,?,?)",
            params![v.environment_id, v.key, v.value, v.var_type],
        )
        .map_err(|e| e.to_string())?;
        v.id = conn.last_insert_rowid();
    }
    Ok(v)
}

pub fn delete_variable(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM variables WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_active_variables(conn: &Connection) -> Result<Vec<Variable>, String> {
    match get_active_environment(conn)? {
        Some(env) => list_variables(conn, env.id),
        None => Ok(Vec::new()),
    }
}
