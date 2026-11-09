package httpclient

import (
	"encoding/json"
	"regexp"
	"strconv"
	"strings"

	"api-dev/internal/models"
)

// RunTests 对单个响应执行请求的测试脚本，返回断言数与失败信息。
// 脚本语法（每行一条）：
//   assertStatus(200)
//   assertHeader("Content-Type", "application/json")
//   assertBodyContains("ok")
//   assertJsonPath("data.id", 1)
func RunTests(req models.Request, resp models.Response) (int, string) {
	lines := strings.Split(strings.TrimSpace(req.TestsScript), "\n")
	asserts := 0
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "//") {
			continue
		}
		asserts++
		if msg := execAssert(line, resp); msg != "" {
			return asserts, msg
		}
	}
	return asserts, ""
}

var assertRe = regexp.MustCompile(`^(\w+)\((.*)\)$`)

// execAssert 执行单条断言，失败返回原因，成功返回空串
func execAssert(line string, resp models.Response) string {
	m := assertRe.FindStringSubmatch(line)
	if m == nil {
		return "语法错误: " + line
	}
	fn := m[1]
	args := parseArgs(m[2])

	switch fn {
	case "assertStatus":
		if len(args) < 1 {
			return "assertStatus 参数不足"
		}
		code, _ := strconv.Atoi(args[0])
		if resp.StatusCode != code {
			return "状态码期望 " + args[0] + "，实际 " + strconv.Itoa(resp.StatusCode)
		}
		return ""

	case "assertHeader":
		if len(args) < 2 {
			return "assertHeader 参数不足"
		}
		got := ""
		for k, v := range resp.Headers {
			if strings.EqualFold(k, args[0]) {
				got = v
				break
			}
		}
		if !strings.Contains(got, args[1]) {
			return "响应头 " + args[0] + " 期望包含 " + args[1] + "，实际 " + got
		}
		return ""

	case "assertBodyContains":
		if len(args) < 1 {
			return "assertBodyContains 参数不足"
		}
		if !strings.Contains(resp.Body, args[0]) {
			return "响应体未包含: " + args[0]
		}
		return ""

	case "assertJsonPath":
		if len(args) < 2 {
			return "assertJsonPath 参数不足"
		}
		got, ok := readJsonPath(resp.Body, args[0])
		if !ok {
			return "JSON 路径不存在: " + args[0]
		}
		if !valuesEqual(got, args[1]) {
			return "路径 " + args[0] + " 期望 " + args[1] + "，实际 " + got
		}
		return ""

	default:
		return "未知断言: " + fn
	}
}

// parseArgs 简单解析函数参数（支持双引号字符串与数字）
func parseArgs(s string) []string {
	var args []string
	var cur strings.Builder
	inStr := false
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c == '"' {
			inStr = !inStr
			continue
		}
		if c == ',' && !inStr {
			args = append(args, strings.TrimSpace(cur.String()))
			cur.Reset()
			continue
		}
		cur.WriteByte(c)
	}
	if cur.Len() > 0 || len(args) > 0 {
		args = append(args, strings.TrimSpace(cur.String()))
	}
	return args
}

// readJsonPath 按点分路径读取 JSON 值（字符串形式返回）
func readJsonPath(body, path string) (string, bool) {
	var data interface{}
	if err := json.Unmarshal([]byte(body), &data); err != nil {
		return "", false
	}
	cur := data
	for _, seg := range strings.Split(path, ".") {
		seg = strings.TrimSpace(seg)
		switch v := cur.(type) {
		case map[string]interface{}:
			val, ok := v[seg]
			if !ok {
				return "", false
			}
			cur = val
		default:
			return "", false
		}
	}
	return stringify(cur), true
}

func stringify(v interface{}) string {
	switch t := v.(type) {
	case string:
		return t
	case float64:
		return strconv.FormatFloat(t, 'f', -1, 64)
	case bool:
		return strconv.FormatBool(t)
	case nil:
		return "null"
	default:
		b, _ := json.Marshal(v)
		return string(b)
	}
}

func valuesEqual(got, expected string) bool {
	if got == expected {
		return true
	}
	// 数字比较：去除可能的浮点差异
	if gf, err1 := strconv.ParseFloat(got, 64); err1 == nil {
		if ef, err2 := strconv.ParseFloat(expected, 64); err2 == nil {
			return gf == ef
		}
	}
	return false
}

// RunCollection 批量运行某集合下的全部请求并返回测试结果
func RunCollection(reqs []models.Request, vars []models.Variable) []models.TestResult {
	results := make([]models.TestResult, 0, len(reqs))
	for _, r := range reqs {
		resp := Send(r, vars)
		tr := models.TestResult{
			RequestID:  r.ID,
			Name:       r.Name,
			StatusCode: resp.StatusCode,
			TimeMs:     resp.TimeMs,
		}
		if resp.Error != "" {
			tr.Error = resp.Error
			tr.Passed = false
		} else if strings.TrimSpace(r.TestsScript) == "" {
			tr.Passed = true // 无测试脚本则仅校验请求成功
		} else {
			asserts, failedMsg := RunTests(r, resp)
			tr.Asserts = asserts
			tr.FailedMsg = failedMsg
			tr.Passed = failedMsg == ""
		}
		results = append(results, tr)
	}
	return results
}
