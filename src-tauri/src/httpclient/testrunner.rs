use crate::httpclient::client;
use crate::models::{Request, Response, TestResult, Variable};

/// 对单个响应执行请求的测试脚本，返回 (断言数, 失败信息)。
/// 脚本语法（每行一条）：
///   assertStatus(200)
///   assertHeader("Content-Type", "application/json")
///   assertBodyContains("ok")
///   assertJsonPath("data.id", 1)
pub fn run_tests(req: &Request, resp: &Response) -> (usize, Option<String>) {
    let mut asserts = 0;
    for line in req.tests_script.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        asserts += 1;
        if let Some(msg) = exec_assert(line, resp) {
            return (asserts, Some(msg));
        }
    }
    (asserts, None)
}

fn exec_assert(line: &str, resp: &Response) -> Option<String> {
    let (fn_name, args_src) = split_call(line)?;
    let args = parse_args(args_src);

    match fn_name {
        "assertStatus" => {
            let expected = args.first()?;
            let code: u16 = expected.parse().unwrap_or(0);
            if resp.status_code != code {
                return Some(format!("状态码期望 {expected}，实际 {}", resp.status_code));
            }
            None
        }
        "assertHeader" => {
            let key = args.first()?.to_lowercase();
            let expected = args.get(1)?;
            let got = resp
                .headers
                .iter()
                .find(|(k, _)| k.to_lowercase() == key)
                .map(|(_, v)| v.as_str())
                .unwrap_or("");
            if !got.contains(expected.as_str()) {
                return Some(format!("响应头 {key} 期望包含 {expected}，实际 {got}"));
            }
            None
        }
        "assertBodyContains" => {
            let expected = args.first()?;
            if !resp.body.contains(expected.as_str()) {
                return Some(format!("响应体未包含: {expected}"));
            }
            None
        }
        "assertJsonPath" => {
            let path = args.first()?;
            let expected = args.get(1)?;
            let got = read_json_path(&resp.body, path)?;
            if !values_equal(&got, expected) {
                return Some(format!("路径 {path} 期望 {expected}，实际 {got}"));
            }
            None
        }
        other => Some(format!("未知断言: {other}")),
    }
}

/// 拆分 `fnName(args)` 形式，返回 (函数名, 参数源串)；不匹配返回 None
fn split_call(line: &str) -> Option<(&str, &str)> {
    let open = line.find('(')?;
    let close = line.rfind(')')?;
    if close < open {
        return None;
    }
    let name = &line[..open];
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') || name.is_empty() {
        return None;
    }
    Some((name, &line[open + 1..close]))
}

/// 简单解析函数参数（支持双引号字符串与数字）
fn parse_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut in_str = false;
    for c in s.chars() {
        match c {
            '"' => in_str = !in_str,
            ',' if !in_str => {
                args.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() || !args.is_empty() {
        args.push(cur.trim().to_string());
    }
    args
}

/// 按点分路径读取 JSON 值（字符串形式返回）；路径不存在返回 None
fn read_json_path(body: &str, path: &str) -> Option<String> {
    let data: serde_json::Value = serde_json::from_str(body).ok()?;
    let mut cur = &data;
    for seg in path.split('.') {
        cur = cur.as_object()?.get(seg.trim())?;
    }
    Some(stringify(cur))
}

fn stringify(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn values_equal(got: &str, expected: &str) -> bool {
    if got == expected {
        return true;
    }
    match (got.parse::<f64>(), expected.parse::<f64>()) {
        (Ok(g), Ok(e)) => g == e,
        _ => false,
    }
}

/// 批量运行请求并返回测试结果
pub fn run_collection(requests: &[Request], vars: &[Variable]) -> Vec<TestResult> {
    requests
        .iter()
        .map(|r| {
            let (resp, _) = client::send(r, vars);
            let mut tr = TestResult {
                request_id: r.id,
                name: r.name.clone(),
                passed: false,
                status_code: resp.status_code,
                time_ms: resp.time_ms,
                asserts: 0,
                failed_msg: None,
                error: None,
            };
            if let Some(err) = &resp.error {
                tr.error = Some(err.clone());
            } else if r.tests_script.trim().is_empty() {
                tr.passed = true; // 无测试脚本则仅校验请求成功
            } else {
                let (asserts, failed_msg) = run_tests(r, &resp);
                tr.asserts = asserts;
                tr.failed_msg = failed_msg;
                tr.passed = tr.failed_msg.is_none();
            }
            tr
        })
        .collect()
}
