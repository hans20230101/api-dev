package models

import "time"

// HTTPMethod HTTP 方法
type HTTPMethod string

const (
	GET    HTTPMethod = "GET"
	POST   HTTPMethod = "POST"
	PUT    HTTPMethod = "PUT"
	DELETE HTTPMethod = "DELETE"
	PATCH  HTTPMethod = "PATCH"
	HEAD   HTTPMethod = "HEAD"
)

// KV 键值对（请求头、参数等），支持单条启用/禁用
type KV struct {
	Key    string `json:"key"`
	Value  string `json:"value"`
	Enable bool   `json:"enable"`
}

// Request 单个接口请求定义
type Request struct {
	ID           int64      `json:"id"`
	CollectionID *int64     `json:"collection_id"`
	Name         string     `json:"name"`
	Method       HTTPMethod `json:"method"`
	URL          string     `json:"url"`
	Headers      []KV       `json:"headers"`
	Params       []KV       `json:"params"`
	BodyType     string     `json:"body_type"` // none | json | form | raw
	Body         string     `json:"body"`
	TestsScript  string     `json:"tests_script"`
	CreatedAt    time.Time  `json:"created_at"`
}

// Collection 集合（支持树形嵌套）
type Collection struct {
	ID        int64     `json:"id"`
	ParentID  *int64    `json:"parent_id"`
	Name      string    `json:"name"`
	CreatedAt time.Time `json:"created_at"`
}

// Environment 环境
type Environment struct {
	ID        int64     `json:"id"`
	Name      string    `json:"name"`
	IsActive  bool      `json:"is_active"`
	CreatedAt time.Time `json:"created_at"`
}

// Variable 环境变量
type Variable struct {
	ID            int64  `json:"id"`
	EnvironmentID int64  `json:"environment_id"`
	Key           string `json:"key"`
	Value         string `json:"value"`
	Type          string `json:"type"` // string | number | boolean
}

// Response 请求响应
type Response struct {
	StatusCode int               `json:"status_code"`
	Status     string            `json:"status"`
	TimeMs     int64             `json:"time_ms"`
	ReqSize    int               `json:"req_size"`
	RespSize   int               `json:"resp_size"`
	Headers    map[string]string `json:"headers"`
	Body       string            `json:"body"`
	Error      string            `json:"error,omitempty"`
}

// TestResult 批量运行单个请求的测试结果
type TestResult struct {
	RequestID  int64  `json:"request_id"`
	Name       string `json:"name"`
	Passed     bool   `json:"passed"`
	StatusCode int    `json:"status_code"`
	TimeMs     int64  `json:"time_ms"`
	Asserts    int    `json:"asserts"`
	FailedMsg  string `json:"failed_msg,omitempty"`
	Error      string `json:"error,omitempty"`
}
