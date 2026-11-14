# API 教研接口测试工具

基于 **Tauri v2 + Vue 3 + Element Plus + Pinia** 开发的桌面接口测试工具，用于接口调试、集合管理、环境变量管理与基础自动化测试场景。

- 应用名称：`API 教研接口测试工具`
- 当前版本：`0.2.0`
- 应用标识：`com.ham.api-dev`

## 功能特性

### 集合与请求管理
- 树状展示集合、子集合与请求
- 支持新建/重命名/删除集合与请求
- 支持请求复制、请求多标签页打开与关闭
- 支持集合导入与导出（JSON）
- 左侧集合区域支持折叠

### 请求编辑与发送
- URL 与 Query Params 双向同步（地址栏带参数自动解析到 Params；Params 修改可回写到 URL）
- 支持常见请求配置：Params、Headers、Body、鉴权等（按界面提供的能力）
- Body 支持多种数据类型/格式编辑与切换，切换时保留已输入内容（按类型隔离）
- 请求名称默认只读，点击“修改”后可编辑
- `Ctrl+S` 保存当前请求页
- 支持发送请求与**取消正在发送的请求**
- 支持保存响应内容/下载响应文件（按接口场景）

### 环境与变量
- 环境与变量管理弹窗：新建/编辑/删除环境，管理环境变量
- 支持切换当前激活环境
- 请求中可结合环境变量使用（项目内置变量替换能力）

### 界面与交互
- 顶层原生菜单栏：
  - **文件**：新建集合、导入集合、退出
  - **设置**：主题（白天 / 黑夜 / 随系统，互斥勾选）、调试（打开/关闭前端 DevTools）
  - **帮助**：关于
- 主题切换：白天 / 黑夜 / 随系统
- 系统托盘：支持从托盘显示主窗口/退出，关闭窗口默认隐藏到托盘
- 默认禁用页面右键菜单

### 数据存储
- 使用本地 SQLite（`rusqlite` bundled）持久化集合、请求、环境与变量等数据
- WebView2 用户数据目录与应用数据目录会优先使用系统应用数据目录；若不可写则回退到可写位置（与数据库目录策略一致）

## 技术栈

### 前端
- [Vue 3](https://vuejs.org/) + `<script setup>` + TypeScript
- [Vite](https://vite.dev/)
- [Element Plus](https://element-plus.org/)
- [Pinia](https://pinia.vuejs.org/)
- Tauri 前端 API：`@tauri-apps/api`、`@tauri-apps/plugin-dialog`

### 后端（Tauri / Rust）
- [Tauri 2](https://tauri.app/)
- `tauri-plugin-dialog`、`tauri-plugin-single-instance`
- `reqwest`（HTTP 客户端，支持 gzip/brotli/deflate/multipart/json）
- `rusqlite`（bundled SQLite）
- `tokio-util`（请求取消令牌）
- `serde` / `serde_json`、`url`、`chrono`

## 项目结构

```text
.
├─ src/                      # 前端源码（Vue 3）
│  ├─ api/                   # 前端调用后端命令封装（wails.ts / models.ts）
│  ├─ assets/                # 静态资源
│  ├─ components/            # UI 组件
│  │  ├─ SidebarCollection.vue
│  │  ├─ RequestTabs.vue
│  │  ├─ RequestEditor.vue
│  │  ├─ ResponseViewer.vue
│  │  ├─ EnvironmentBar.vue
│  │  ├─ KVEditor.vue
│  │  ├─ FormDataEditor.vue
│  │  └─ CodeViewer.vue
│  ├─ stores/                # Pinia 状态管理
│  │  ├─ collection.ts
│  │  ├─ environment.ts
│  │  ├─ runtime.ts
│  │  ├─ tabs.ts
│  │  └─ theme.ts
│  ├─ App.vue
│  ├─ main.ts
│  └─ style.css
├─ src-tauri/                # Tauri / Rust 后端
│  ├─ src/
│  │  ├─ commands.rs         # Tauri 命令
│  │  ├─ db.rs               # SQLite 初始化与访问
│  │  ├─ httpclient/         # HTTP 请求与测试执行
│  │  ├─ repository/         # 数据访问层
│  │  ├─ models.rs           # 数据模型
│  │  ├─ lib.rs              # 应用启动、窗口/菜单/托盘
│  │  └─ main.rs
│  ├─ icons/                 # 应用图标
│  ├─ Cargo.toml
│  └─ tauri.conf.json
├─ index.html
├─ package.json
├─ vite.config.ts
├─ tsconfig.json
└─ README.md
```

## 开发环境准备

### 前置依赖
- [Node.js](https://nodejs.org/)（建议 LTS）
- 包管理器：`npm` / `pnpm`（仓库中同时存在 `package-lock.json` 与 `pnpm-lock.yaml`）
- [Rust](https://www.rust-lang.org/tools/install)（stable）
- Tauri 2 桌面开发依赖（Windows 需 WebView2 Runtime；详见 [Tauri Prerequisites](https://tauri.app/start/prerequisites/)）

### 安装依赖

```bash
# npm
npm install

# 或 pnpm
pnpm install
```

## 本地开发与构建

### 前端开发（Vite）

```bash
npm run dev
```

> `package.json` 中的脚本通过 `cross-env` 将 `TMP/TEMP/TMPDIR` 指向项目内 `.build-tmp` 目录。

### Tauri 桌面应用开发

在已安装 Tauri CLI 与 Rust 环境后，可使用 Tauri 开发模式启动（会按 `tauri.conf.json` 的 `beforeDevCommand`/`devUrl` 加载前端）：

```bash
npx tauri dev
# 或（若已全局/本地安装 tauri CLI）
cargo tauri dev
```

### 构建

前端构建：

```bash
npm run build
```

Tauri 安装包构建（Windows NSIS，见 `tauri.conf.json`）：

```bash
npm run tauri:build
# 或
npx tauri build
```

构建产物与安装包位置遵循 Tauri 默认输出（通常在 `src-tauri/target/` 下）。

## 使用说明（简要）

1. **集合/请求**：在左侧集合树中创建集合与请求，点击请求在右侧标签页中打开编辑。
2. **编辑请求**：填写 URL、方法、参数、Headers、Body 等；URL 与 Params 会按规则双向同步；修改后使用 `Ctrl+S` 保存。
3. **发送/取消**：点击发送发起请求，请求进行中可点击取消中止。
4. **环境变量**：通过“环境与变量管理”配置环境与变量，并选择激活环境用于请求。
5. **导入导出**：可通过顶层菜单“文件 -> 导入集合”导入，集合支持导出为 JSON 文件。
6. **主题与调试**：通过“设置 -> 主题”切换外观；“设置 -> 调试”可打开/关闭前端调试工具（DevTools）。

## 说明与已知事项

- `vite build` 在部分受限运行环境下可能因沙箱限制 `esbuild` 派生进程而失败；这是环境限制而非项目代码问题。开发/验证时可优先使用 `vue-tsc --noEmit` 与 `cargo check` 进行类型与编译检查。
- 本项目从 Wails 迁移而来，前端与后端交互封装位于 [`src/api/wails.ts`](src/api/wails.ts)（统一通过 Tauri `invoke` 调用命令）。
- `my.txt` 中记录了部分需求与迭代项，可作为功能演进参考。

## 许可证

未在仓库中声明许可证；如需对外分发请补充相应的 `LICENSE` 文件与说明。
