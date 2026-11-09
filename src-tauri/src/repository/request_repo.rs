use crate::models::{Kv, Request};
use rusqlite::{params, Connection, OptionalExtension};

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn kv_to_json(kvs: &[Kv]) -> String {
    serde_json::to_string(kvs).unwrap_or_else(|_| "[]".to_string())
}

fn json_to_kvs(s: &str) -> Vec<Kv> {
    serde_json::from_str(s).unwrap_or_default()
}

fn row_to_request(row: &rusqlite::Row) -> rusqlite::Result<Request> {
    Ok(Request {
        id: row.get(0)?,
        collection_id: row.get(1)?,
        name: row.get(2)?,
        method: row.get(3)?,
        url: row.get(4)?,
        headers: json_to_kvs(&row.get::<_, String>(5)?),
        params: json_to_kvs(&row.get::<_, String>(6)?),
        body_type: row.get(7)?,
        body: row.get(8)?,
        tests_script: row.get(9)?,
        created_at: row.get(10)?,
    })
}

const REQUEST_COLS: &str =
    "id, collection_id, name, method, url, headers, params, body_type, body, tests_script, created_at";

pub fn list_requests(conn: &Connection, collection_id: Option<i64>) -> Result<Vec<Request>, String> {
    let list: Vec<Request> = match collection_id {
        Some(cid) => {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {REQUEST_COLS} FROM requests WHERE collection_id = ? ORDER BY name ASC"
                ))
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![cid], row_to_request)
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        }
        None => {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {REQUEST_COLS} FROM requests ORDER BY name ASC"
                ))
                .map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], row_to_request).map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        }
    };
    Ok(list)
}

pub fn get_request(conn: &Connection, id: i64) -> Result<Request, String> {
    conn.query_row(
        &format!("SELECT {REQUEST_COLS} FROM requests WHERE id = ?"),
        params![id],
        row_to_request,
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("请求不存在: {id}"))
}

pub fn save_request(conn: &Connection, mut r: Request) -> Result<Request, String> {
    let headers = kv_to_json(&r.headers);
    let params_json = kv_to_json(&r.params);
    if r.id > 0 {
        conn.execute(
            "UPDATE requests SET collection_id=?, name=?, method=?, url=?, headers=?, params=?, body_type=?, body=?, tests_script=? WHERE id=?",
            params![
                r.collection_id,
                r.name,
                r.method,
                r.url,
                headers,
                params_json,
                r.body_type,
                r.body,
                r.tests_script,
                r.id
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(r)
    } else {
        conn.execute(
            "INSERT INTO requests (collection_id, name, method, url, headers, params, body_type, body, tests_script) VALUES (?,?,?,?,?,?,?,?,?)",
            params![
                r.collection_id,
                r.name,
                r.method,
                r.url,
                headers,
                params_json,
                r.body_type,
                r.body,
                r.tests_script
            ],
        )
        .map_err(|e| e.to_string())?;
        r.id = conn.last_insert_rowid();
        r.created_at = Some(now_str());
        Ok(r)
    }
}

pub fn delete_request(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM requests WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
