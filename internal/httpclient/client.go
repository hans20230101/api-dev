package httpclient

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/url"
	"sort"
	"strings"
	"time"

	"api-dev/internal/models"
)

// Send 执行一次 HTTP 请求，返回结构化响应。
// vars 用于 {{var}} 模板替换（通常来自当前激活环境）。
func Send(req models.Request, vars []models.Variable) models.Response {
	// 1. 构建变量映射表
	varMap := buildVarMap(vars)

	// 2. 替换 URL 与参数中的 {{var}}
	rawURL := applyVars(req.URL, varMap)
	finalURL := buildURLWithParams(rawURL, req.Params, varMap)

	httpReq, err := http.NewRequest(string(req.Method), finalURL, nil)
	if err != nil {
		return models.Response{Error: "请求构造失败: " + err.Error()}
	}

	// 3. 设置请求头
	for _, h := range req.Headers {
		if !h.Enable || h.Key == "" {
			continue
		}
		httpReq.Header.Set(applyVars(h.Key, varMap), applyVars(h.Value, varMap))
	}

	// 4. 设置 Body
	bodyBytes, contentType, err := buildBody(req, varMap)
	if err != nil {
		return models.Response{Error: "请求体构造失败: " + err.Error()}
	}
	if bodyBytes != nil {
		httpReq.Body = io.NopCloser(bytes.NewReader(bodyBytes))
		httpReq.ContentLength = int64(len(bodyBytes))
		if contentType != "" {
			httpReq.Header.Set("Content-Type", contentType)
		}
	}

	// 5. 发送
	client := &http.Client{Timeout: 30 * time.Second}
	start := time.Now()
	resp, err := client.Do(httpReq)
	elapsed := time.Since(start).Milliseconds()
	if err != nil {
		return models.Response{Error: "请求发送失败: " + err.Error(), TimeMs: elapsed}
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	headers := map[string]string{}
	for k := range resp.Header {
		headers[k] = strings.Join(resp.Header[k], ", ")
	}

	return models.Response{
		StatusCode: resp.StatusCode,
		Status:     resp.Status,
		TimeMs:     elapsed,
		ReqSize:    len(bodyBytes),
		RespSize:   len(body),
		Headers:    headers,
		Body:       tryPrettyJSON(string(body)),
	}
}

func buildVarMap(vars []models.Variable) map[string]string {
	m := map[string]string{}
	for _, v := range vars {
		m[v.Key] = v.Value
	}
	return m
}

// applyVars 将字符串中的 {{key}} 替换为对应变量值
func applyVars(s string, vars map[string]string) string {
	for k, v := range vars {
		s = strings.ReplaceAll(s, "{{"+k+"}}", v)
	}
	return s
}

// buildURLWithParams 拼接 query 参数（已启用的）
func buildURLWithParams(rawURL string, params []models.KV, vars map[string]string) string {
	u, err := url.Parse(rawURL)
	if err != nil {
		return rawURL
	}
	q := u.Query()
	for _, p := range params {
		if !p.Enable || p.Key == "" {
			continue
		}
		q.Add(applyVars(p.Key, vars), applyVars(p.Value, vars))
	}
	u.RawQuery = q.Encode()
	return u.String()
}

// buildBody 根据 BodyType 构造请求体与 Content-Type
func buildBody(req models.Request, vars map[string]string) ([]byte, string, error) {
	switch req.BodyType {
	case "none", "":
		return nil, "", nil
	case "json":
		body := applyVars(req.Body, vars)
		return []byte(body), "application/json; charset=utf-8", nil
	case "raw":
		body := applyVars(req.Body, vars)
		return []byte(body), "text/plain; charset=utf-8", nil
	case "form":
		// Body 视为 JSON 数组 [{key,value}]，转成 urlencoded
		var kvs []models.KV
		if err := json.Unmarshal([]byte(req.Body), &kvs); err != nil {
			return nil, "", err
		}
		form := url.Values{}
		for _, kv := range kvs {
			if !kv.Enable || kv.Key == "" {
				continue
			}
			form.Add(applyVars(kv.Key, vars), applyVars(kv.Value, vars))
		}
		return []byte(form.Encode()), "application/x-www-form-urlencoded", nil
	}
	return nil, "", nil
}

// tryPrettyJSON 尝试将 JSON 字符串美化
func tryPrettyJSON(s string) string {
	var buf bytes.Buffer
	if err := json.Indent(&buf, []byte(s), "", "  "); err == nil {
		return buf.String()
	}
	return s
}

// SortedHeaders 返回按 key 排序的响应头列表，便于前端展示
func SortedHeaders(h map[string]string) []models.KV {
	keys := make([]string, 0, len(h))
	for k := range h {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	out := make([]models.KV, 0, len(keys))
	for _, k := range keys {
		out = append(out, models.KV{Key: k, Value: h[k], Enable: true})
	}
	return out
}
