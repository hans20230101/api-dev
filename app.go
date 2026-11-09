package main

import (
	"context"
	"encoding/json"
	"errors"

	"api-dev/internal/db"
	"api-dev/internal/httpclient"
	"api-dev/internal/models"
	"api-dev/internal/repository"
)

// App 是 Wails 暴露给前端的业务绑定对象
type App struct {
	ctx context.Context
}

func NewApp() *App { return &App{} }

// startup 应用启动时初始化数据库
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
	if err := db.Init(); err != nil {
		println("db init error:", err.Error())
	}
}

// ===== 集合 =====

func (a *App) ListCollections() []models.Collection {
	c, _ := repository.ListCollections(nil)
	return c
}

func (a *App) CreateCollection(name string, parentID *int64) models.Collection {
	c, _ := repository.CreateCollection(name, parentID)
	return c
}

func (a *App) UpdateCollection(id int64, name string) error {
	return repository.UpdateCollection(id, name)
}

func (a *App) DeleteCollection(id int64) error {
	return repository.DeleteCollection(id)
}

// ===== 请求 =====

func (a *App) ListRequests(collectionID *int64) []models.Request {
	r, _ := repository.ListRequests(collectionID)
	return r
}

func (a *App) GetRequest(id int64) models.Request {
	r, _ := repository.GetRequest(id)
	return r
}

func (a *App) SaveRequest(r models.Request) models.Request {
	saved, _ := repository.SaveRequest(r)
	return saved
}

func (a *App) DeleteRequest(id int64) error {
	return repository.DeleteRequest(id)
}

// ===== 环境 =====

func (a *App) ListEnvironments() []models.Environment {
	e, _ := repository.ListEnvironments()
	return e
}

func (a *App) CreateEnvironment(name string) models.Environment {
	e, _ := repository.CreateEnvironment(name)
	return e
}

func (a *App) UpdateEnvironment(id int64, name string) error {
	return repository.UpdateEnvironment(id, name)
}

func (a *App) DeleteEnvironment(id int64) error {
	return repository.DeleteEnvironment(id)
}

func (a *App) SetActiveEnvironment(id int64) error {
	return repository.SetActiveEnvironment(id)
}

func (a *App) GetActiveEnvironment() *models.Environment {
	e, _ := repository.GetActiveEnvironment()
	return e
}

// ===== 变量 =====

func (a *App) ListVariables(envID int64) []models.Variable {
	v, _ := repository.ListVariables(envID)
	return v
}

func (a *App) SaveVariable(v models.Variable) models.Variable {
	saved, _ := repository.SaveVariable(v)
	return saved
}

func (a *App) DeleteVariable(id int64) error {
	return repository.DeleteVariable(id)
}

// ===== 发送与测试 =====

// SendRequest 发送单个请求（自动使用当前激活环境的变量）
func (a *App) SendRequest(req models.Request) models.Response {
	vars, _ := repository.GetActiveVariables()
	return httpclient.Send(req, vars)
}

// RunCollection 批量运行某集合下全部请求并返回测试结果
func (a *App) RunCollection(collectionID int64) []models.TestResult {
	reqs, err := repository.ListRequests(&collectionID)
	if err != nil || len(reqs) == 0 {
		return []models.TestResult{}
	}
	vars, _ := repository.GetActiveVariables()
	return httpclient.RunCollection(reqs, vars)
}

// ===== 导入导出 =====

// ExportData 导入导出的数据结构
type ExportData struct {
	Collections  []models.Collection  `json:"collections"`
	Requests     []models.Request     `json:"requests"`
	Environments []models.Environment `json:"environments"`
	Variables    []models.Variable    `json:"variables"`
}

// ExportAll 导出全部数据为 JSON 字符串
func (a *App) ExportAll() string {
	data := ExportData{}
	data.Collections, _ = repository.ListCollections(nil)
	data.Requests, _ = repository.ListRequests(nil)
	data.Environments, _ = repository.ListEnvironments()
	allVars := []models.Variable{}
	for _, e := range data.Environments {
		vs, _ := repository.ListVariables(e.ID)
		allVars = append(allVars, vs...)
	}
	data.Variables = allVars
	b, _ := json.MarshalIndent(data, "", "  ")
	return string(b)
}

// ImportAll 从 JSON 字符串导入数据（按现有结构重建，变量随环境重建后 ID 变化暂不迁移）
func (a *App) ImportAll(jsonStr string) error {
	var data ExportData
	if err := json.Unmarshal([]byte(jsonStr), &data); err != nil {
		return errors.New("JSON 解析失败: " + err.Error())
	}
	for _, c := range data.Collections {
		_, _ = repository.CreateCollection(c.Name, c.ParentID)
	}
	for _, r := range data.Requests {
		_, _ = repository.SaveRequest(r)
	}
	for _, e := range data.Environments {
		_, _ = repository.CreateEnvironment(e.Name)
	}
	return nil
}
