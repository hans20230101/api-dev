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
    let body_type: String = row.get(7)?;
    let body_col: String = row.get(8)?;
    let body_json: String = row.get(9)?;
    let body_form: String = row.get(10)?;
    let body_raw: String = row.get(11)?;
    // 读取时按类型路由：body 始终是当前类型的活动数据
    let body = match body_type.as_str() {
        "json" => body_json.clone(),
        "form" => body_form.clone(),
        "raw" => body_raw.clone(),
        _ => body_col, // form-data / none 沿用原 body 列
    };
    Ok(Request {
        id: row.get(0)?,
        collection_id: row.get(1)?,
        name: row.get(2)?,
        method: row.get(3)?,
        url: row.get(4)?,
        headers: json_to_kvs(&row.get::<_, String>(5)?),
        params: json_to_kvs(&row.get::<_, String>(6)?),
        body_type,
        body,
        body_json,
        body_form,
        body_raw,
        tests_script: row.get(12)?,
        created_at: row.get(13)?,
        raw_content_type: row.get(14)?,
    })
}

const REQUEST_COLS: &str =
    "id, collection_id, name, method, url, headers, params, body_type, body, body_json, body_form, body_raw, tests_script, created_at, raw_content_type";

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
    // 前端请求对象内各类型数据永久驻留各自字段（body_json/body_form/body_raw/body），
    // 保存时全部原样入库，不做任何合并或清空——保证各类型编辑内容互不丢失
    if r.id > 0 {
        conn.execute(
            "UPDATE requests SET collection_id=?, name=?, method=?, url=?, headers=?, params=?, body_type=?, body=?, body_json=?, body_form=?, body_raw=?, tests_script=?, raw_content_type=? WHERE id=?",
            params![
                r.collection_id,
                r.name,
                r.method,
                r.url,
                headers,
                params_json,
                r.body_type,
                r.body,
                r.body_json,
                r.body_form,
                r.body_raw,
                r.tests_script,
                r.raw_content_type,
                r.id
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(r)
    } else {
        conn.execute(
            "INSERT INTO requests (collection_id, name, method, url, headers, params, body_type, body, body_json, body_form, body_raw, tests_script, raw_content_type) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                r.collection_id,
                r.name,
                r.method,
                r.url,
                headers,
                params_json,
                r.body_type,
                r.body,
                r.body_json,
                r.body_form,
                r.body_raw,
                r.tests_script,
                r.raw_content_type
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
