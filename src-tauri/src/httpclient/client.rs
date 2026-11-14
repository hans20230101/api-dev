use crate::models::{Kv, Request, Response, Variable};
use std::collections::{HashMap, HashSet};

fn build_var_map(vars: &[Variable]) -> HashMap<&str, &str> {
    vars.iter().map(|v| (v.key.as_str(), v.value.as_str())).collect()
}

/// 将字符串中的 {{key}} 替换为对应变量值
fn apply_vars(s: &str, vars: &HashMap<&str, &str>) -> String {
    let mut out = s.to_string();
    for (k, v) in vars {
        let pattern = format!("{{{{{k}}}}}");
        if out.contains(&pattern) {
            out = out.replace(&pattern, v);
        }
    }
    out
}

fn build_url_with_params(
    raw_url: &str,
    params: &[Kv],
    vars: &HashMap<&str, &str>,
) -> String {
    let mut url = match url::Url::parse(raw_url) {
        Ok(u) => u,
        Err(_) => return raw_url.to_string(),
    };
    // 编辑器已把启用的参数同步拼进地址栏 query，这里只追加地址里还没有的键，
    // 避免同一参数在 query 中出现两份；地址自带 query 的旧数据也能原样发送
    let existing: HashSet<String> = url.query_pairs().map(|(k, _)| k.into_owned()).collect();
    {
        let mut qp = url.query_pairs_mut();
        for p in params {
            if p.enable && !p.key.is_empty() {
                let key = apply_vars(&p.key, vars);
                if !existing.contains(&key) {
                    qp.append_pair(&key, &apply_vars(&p.value, vars));
                }
            }
        }
    }
    url.to_string()
}

/// raw 请求体所选格式对应的 Content-Type
fn raw_content_type(ct: &str) -> &'static str {
    match ct {
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "javascript" => "text/javascript; charset=utf-8",
        _ => "text/plain; charset=utf-8",
    }
}

/// 按请求类型返回实际参与发送的请求体数据源：
/// json/form/raw 各用隔离字段，form-data/none 用 body 本身
fn body_source<'a>(req: &'a Request) -> &'a str {
    match req.body_type.as_str() {
        "json" => &req.body_json,
        "form" => &req.body_form,
        "raw" => &req.body_raw,
        _ => &req.body,
    }
}

fn build_body(req: &Request, vars: &HashMap<&str, &str>) -> Result<(Option<Vec<u8>>, Option<&'static str>), String> {
    let body = body_source(req);
    match req.body_type.as_str() {
        "json" => Ok((Some(apply_vars(body, vars).into_bytes()), Some("application/json; charset=utf-8"))),
        "raw" => Ok((Some(apply_vars(body, vars).into_bytes()), Some(raw_content_type(&req.raw_content_type)))),
        "form" => {
            let kvs: Vec<Kv> = serde_json::from_str(body)
                .map_err(|e| format!("form 请求体需为 JSON 数组: {e}"))?;
            let mut form = Vec::new();
            for kv in kvs {
                if kv.enable && !kv.key.is_empty() {
                    let k = apply_vars(&kv.key, vars);
                    let v = apply_vars(&kv.value, vars);
                    if !form.is_empty() {
                        form.push(b'&');
                    }
                    form.extend_from_slice(urlform_encode(&k).as_bytes());
                    form.push(b'=');
                    form.extend_from_slice(urlform_encode(&v).as_bytes());
                }
            }
            Ok((Some(form), Some("application/x-www-form-urlencoded")))
        }
        _ => Ok((None, None)),
    }
}

/// form-data 行反序列化失败时的统一错误提示
fn parse_form_data(body: &str) -> Result<Vec<Kv>, String> {
    serde_json::from_str(body).map_err(|e| format!("form-data 请求体需为 JSON 数组: {e}"))
}

/// 构建 multipart/form-data 请求体，返回 (body, boundary)：
/// 文本行走变量替换，文件行从磁盘读取字节
fn build_multipart(req: &Request, vars: &HashMap<&str, &str>) -> Result<(Vec<u8>, String), String> {
    let boundary = generate_boundary();
    let kvs = parse_form_data(body_source(req))?;
    let mut out = Vec::new();
    for kv in kvs {
        if !kv.enable || kv.key.is_empty() {
            continue;
        }
        let name = apply_vars(&kv.key, vars);
        out.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        if kv.file == Some(true) {
            let path = kv.value.trim();
            let bytes = std::fs::read(path)
                .map_err(|e| format!("读取文件失败 {path}: {e}"))?;
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".to_string());
            out.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n", escape_header(&name), escape_header(&filename)).as_bytes(),
            );
            let ct = guess_mime(&filename);
            out.extend_from_slice(format!("Content-Type: {ct}\r\n\r\n").as_bytes());
            out.extend_from_slice(&bytes);
        } else {
            let v = apply_vars(&kv.value, vars);
            out.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", escape_header(&name)).as_bytes(),
            );
            out.extend_from_slice(apply_vars(&v, vars).as_bytes());
        }
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    Ok((out, boundary))
}

/// 生成不易与内容冲突的 multipart 边界
fn generate_boundary() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("----api-dev-boundary-{t:x}{}{:x}", std::process::id(), t as u64)
}

/// 转义 header 值中的引号/反斜杠，避免注入破坏 multipart 结构
fn escape_header(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 按扩展名粗略推断文件 MIME 类型
fn guess_mime(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" | "md" | "log" | "csv" => "text/plain",
        "html" | "htm" => "text/html",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "js" => "text/javascript",
        "css" => "text/css",
        _ => "application/octet-stream",
    }
}

fn urlform_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn try_pretty_json(s: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| s.to_string()),
        Err(_) => s.to_string(),
    }
}

/// 判断 Content-Type 是否为可直接展示的文本类型
fn is_text_content(ct: Option<&str>) -> bool {
    match ct {
        Some(s) => {
            let s = s.to_ascii_lowercase();
            s.starts_with("text/")
                || s.contains("json")
                || s.contains("xml")
                || s.contains("javascript")
                || s.contains("x-www-form-urlencoded")
                || s.contains("svg")
        }
        None => true,
    }
}

/// 过滤文件名中的非法字符并去掉路径部分
fn sanitize_filename(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("").trim();
    base
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        .collect()
}

/// 简单的百分号解码（用于 filename*=UTF-8''xxx）
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Ok(hi), Ok(lo)) = (
                u8::from_str_radix(&s[i + 1..i + 2], 16),
                u8::from_str_radix(&s[i + 2..i + 3], 16),
            ) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// 从 Content-Disposition 解析保存时的默认文件名
fn parse_filename(headers: &HashMap<String, String>) -> Option<String> {
    let cd = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-disposition"))?
        .1;
    if let Some(i) = cd.find("filename*=") {
        let rest = &cd[i + 10..];
        let val = rest.split(';').next().unwrap_or("").trim();
        let val = val.split("''").last().unwrap_or(val).trim_matches('"');
        let cleaned = sanitize_filename(&percent_decode(val));
        if !cleaned.is_empty() {
            return Some(cleaned);
        }
    }
    if let Some(i) = cd.find("filename=") {
        let rest = &cd[i + 9..];
        let val = rest
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"');
        let cleaned = sanitize_filename(val);
        if !cleaned.is_empty() {
            return Some(cleaned);
        }
    }
    None
}

/// 执行一次 HTTP 请求，返回结构化响应与原始响应字节。
/// 异步实现：发送阶段可被取消令牌中断（取消即丢弃 Future，断开连接）；
/// 同步上下文（批量测试）请调用 [`send`] 包装。
pub async fn send_async(req: &Request, vars: &[Variable]) -> (Response, Vec<u8>) {
    let var_map = build_var_map(vars);
    let raw_url = apply_vars(&req.url, &var_map);
    let final_url = build_url_with_params(&raw_url, &req.params, &var_map);

    let method = match reqwest::Method::from_bytes(req.method.as_bytes()) {
        Ok(m) => m,
        Err(e) => {
            return (
                Response {
                    status_code: 0,
                    status: String::new(),
                    time_ms: 0,
                    req_size: 0,
                    resp_size: 0,
                    headers: HashMap::new(),
                    body: String::new(),
                    body_id: None,
                    content_type: None,
                    is_binary: false,
                    filename: None,
                    error: Some(format!("请求构造失败: {e}")),
                },
                Vec::new(),
            );
        }
    };

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (error_response(format!("客户端构建失败: {e}")), Vec::new());
        }
    };

    let mut http_req = client.request(method, &final_url);

    for h in &req.headers {
        if h.enable && !h.key.is_empty() {
            http_req = http_req.header(apply_vars(&h.key, &var_map), apply_vars(&h.value, &var_map));
        }
    }

    let (body_bytes, content_type) = if req.body_type == "form-data" {
        match build_multipart(req, &var_map) {
            Ok((bytes, boundary)) => (
                Some(bytes),
                Some(&*Box::leak(
                    format!("multipart/form-data; boundary={boundary}").into_boxed_str(),
                )),
            ),
            Err(e) => return (error_response(format!("请求体构造失败: {e}")), Vec::new()),
        }
    } else {
        match build_body(req, &var_map) {
            Ok(b) => b,
            Err(e) => return (error_response(format!("请求体构造失败: {e}")), Vec::new()),
        }
    };

    let req_size = body_bytes.as_ref().map(|b| b.len()).unwrap_or(0);
    if let Some(bytes) = &body_bytes {
        if let Some(ct) = content_type {
            http_req = http_req.header("Content-Type", ct);
        }
        http_req = http_req.body(bytes.clone());
    }
    let start = std::time::Instant::now();
    let result = http_req.send().await;
    let elapsed = start.elapsed().as_millis() as i64;

    match result {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            let status_text = resp.status().to_string();
            let mut headers = HashMap::new();
            for (k, v) in resp.headers() {
                let val = v.to_str().unwrap_or("").to_string();
                headers
                    .entry(k.to_string())
                    .and_modify(|existing: &mut String| {
                        existing.push_str(", ");
                        existing.push_str(&val);
                    })
                    .or_insert(val);
            }
            let resp_content_type = headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                .map(|(_, v)| v.clone());
            let filename = parse_filename(&headers);
            match resp.bytes().await {
                Ok(body) => {
                    let text = is_text_content(resp_content_type.as_deref())
                        && std::str::from_utf8(&body).is_ok()
                        && !body.iter().any(|&b| b == 0);
                    let response_body = if text {
                        try_pretty_json(&String::from_utf8_lossy(&body))
                    } else {
                        String::new()
                    };
                    let resp_size = body.len();
                    (
                        Response {
                            status_code,
                            status: status_text,
                            time_ms: elapsed,
                            req_size,
                            resp_size,
                            headers,
                            body: response_body,
                            body_id: None,
                            content_type: resp_content_type,
                            is_binary: !text,
                            filename,
                            error: None,
                        },
                        body.to_vec(),
                    )
                }
                Err(e) => (
                    Response {
                        status_code,
                        status: status_text,
                        time_ms: elapsed,
                        req_size,
                        resp_size: 0,
                        headers,
                        body: String::new(),
                        body_id: None,
                        content_type: resp_content_type,
                        is_binary: false,
                        filename,
                        error: Some(format!("读取响应失败: {e}")),
                    },
                    Vec::new(),
                ),
            }
        }
        Err(e) => (
            Response {
                status_code: 0,
                status: String::new(),
                time_ms: elapsed,
                req_size,
                resp_size: 0,
                headers: HashMap::new(),
                body: String::new(),
                body_id: None,
                content_type: None,
                is_binary: false,
                filename: None,
                error: Some(format!("请求发送失败: {e}")),
            },
            Vec::new(),
        ),
    }
}

/// 同步包装：在 tauri 异步运行时上执行 [`send_async`]，供批量测试等同步上下文调用
pub fn send(req: &Request, vars: &[Variable]) -> (Response, Vec<u8>) {
    tauri::async_runtime::block_on(send_async(req, vars))
}

/// 用户取消请求时返回的占位响应（error 非空，不会进入响应缓存）
pub fn cancelled_response() -> Response {
    error_response("请求已取消".to_string())
}

fn error_response(msg: String) -> Response {
    Response {
        status_code: 0,
        status: String::new(),
        time_ms: 0,
        req_size: 0,
        resp_size: 0,
        headers: HashMap::new(),
        body: String::new(),
        body_id: None,
        content_type: None,
        is_binary: false,
        filename: None,
        error: Some(msg),
    }
}
