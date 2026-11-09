use crate::models::{Kv, Request, Response, Variable};
use std::collections::HashMap;

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
    {
        let mut qp = url.query_pairs_mut();
        for p in params {
            if p.enable && !p.key.is_empty() {
                qp.append_pair(&apply_vars(&p.key, vars), &apply_vars(&p.value, vars));
            }
        }
    }
    // append_pair 会把已有 query 追加在原 query 之后，这里需要先清空原 query 再编码
    // 上面的实现已覆盖（append 语义与 Go 版一致：保留原 query 并追加参数）
    url.to_string()
}

fn build_body(req: &Request, vars: &HashMap<&str, &str>) -> Result<(Option<Vec<u8>>, Option<&'static str>), String> {
    match req.body_type.as_str() {
        "json" => Ok((Some(apply_vars(&req.body, vars).into_bytes()), Some("application/json; charset=utf-8"))),
        "raw" => Ok((Some(apply_vars(&req.body, vars).into_bytes()), Some("text/plain; charset=utf-8"))),
        "form" => {
            let kvs: Vec<Kv> = serde_json::from_str(&req.body)
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

/// 执行一次 HTTP 请求，返回结构化响应与原始响应字节
pub fn send(req: &Request, vars: &[Variable]) -> (Response, Vec<u8>) {
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

    let client = match reqwest::blocking::Client::builder()
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

    let (body_bytes, content_type) = match build_body(req, &var_map) {
        Ok(b) => b,
        Err(e) => return (error_response(format!("请求体构造失败: {e}")), Vec::new()),
    };

    let req_size = body_bytes.as_ref().map(|b| b.len()).unwrap_or(0);
    if let Some(bytes) = &body_bytes {
        if let Some(ct) = content_type {
            http_req = http_req.header("Content-Type", ct);
        }
        http_req = http_req.body(bytes.clone());
    }

    let start = std::time::Instant::now();
    let elapsed = start.elapsed().as_millis() as i64;
    let result = http_req.send();
    let elapsed = if elapsed > 0 { elapsed } else { start.elapsed().as_millis() as i64 };

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
            match resp.bytes() {
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
