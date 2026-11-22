<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { useCollectionStore } from '@/stores/collection'
import { useRuntimeStore } from '@/stores/runtime'
import { useTabsStore } from '@/stores/tabs'
import { ElMessage } from 'element-plus'
import { CopyDocument, MagicStick, Edit, Check } from '@element-plus/icons-vue'
import KVEditor from './KVEditor.vue'
import FormDataEditor from './FormDataEditor.vue'
import type { models } from '@/api/models'

const colStore = useCollectionStore()
const runtime = useRuntimeStore()
const tabs = useTabsStore()

// 当前激活选项卡中的请求（各选项卡编辑内容互不影响）
const req = computed(() => tabs.activeTab)

// 是否新建请求（尚未保存过）：新建时 KV 编辑器默认显示一行空白行，修改时不显示
const isNewReq = computed(() => !!req.value && tabs.newIds.includes(req.value.id))

// ---------- 外层页签自动定位 ----------
// 打开/切换请求或改变方法时按方法落位：
// GET/DELETE/PATCH/HEAD → Params，PUT/POST → Body。
// 之后用户手动切换页签不受影响（watcher 只在请求 id 或方法变化时触发）

const activePane = ref('params')
watch(
  () => {
    const r = req.value
    return r ? `${r.id}|${r.method}` : null
  },
  () => {
    const r = req.value
    if (!r) return
    activePane.value = r.method === 'PUT' || r.method === 'POST' ? 'body' : 'params'
  },
  { immediate: true },
)

/** 当前类型对应的编辑数据源（raw 有行数与工具栏操作） */
const activeBody = computed(() => {
  const r = req.value
  if (!r) return ''
  if (r.body_type === 'raw') return r.body_raw ?? ''
  return r.body
})
const bodyLineCount = computed(() => activeBody.value.split('\n').length)

const methods = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD']
const rawContentTypes = [
  { label: 'Text', value: 'text' },
  { label: 'JSON', value: 'json' },
  { label: 'XML', value: 'xml' },
  { label: 'HTML', value: 'html' },
  { label: 'JavaScript', value: 'javascript' },
]
const bodyTypes = [
  { label: 'none', value: 'none' },
  { label: 'form', value: 'form' },
  { label: 'raw', value: 'raw' },
  { label: 'form-data', value: 'form-data' },
]

// ---------- Auth 行级编辑锁 ----------
// 与 KVEditor 一致：有值的字段默认锁定（防误触泄露凭证），点「修改」解锁，改完点 ✓ 重新锁定
const authUnlocked = reactive(new Set<string>())
function authFieldValue(f: keyof models.Auth): string {
  return (req.value?.auth?.[f] as string) ?? ''
}
function isAuthLocked(f: keyof models.Auth): boolean {
  return authFieldValue(f).trim() !== '' && !authUnlocked.has(f)
}
function toggleAuthEdit(f: keyof models.Auth) {
  if (authUnlocked.has(f)) authUnlocked.delete(f)
  else authUnlocked.add(f)
}

/**
 * 切换 Body 类型（零搬运）：各类型数据永久驻留各自字段
 * （form→body_form、raw→body_raw、form-data/none→body），
 * 切换只改 body_type，编辑内容随字段驻留，来回切换互不覆盖。
 */
function switchBodyType(type: string) {
  const r = req.value
  if (!r || type === r.body_type) return
  // 兼容旧数据：body_type 为 json 的请求统一迁入 raw（内容搬进 body_raw，Content-Type 选 json）
  if (r.body_type === 'json' && !(r.body_raw ?? '').trim() && (r.body_json ?? '').trim()) {
    r.body_raw = r.body_json
    r.raw_content_type = 'json'
  }
  r.body_type = type
}

async function send() {
  if (!req.value) return
  if (!req.value.url) {
    ElMessage.warning('请填写 URL')
    return
  }
  await runtime.send(req.value)
}

async function save() {
  if (!req.value) return
  await colStore.saveRequest(req.value)
  ElMessage.success('已保存')
}

// 请求名称：默认只读展示，点击「修改」进入编辑，失焦或回车退出编辑
const nameEditing = ref(false)
const nameInputRef = ref<InstanceType<typeof import('element-plus')['ElInput']>>()

function startRename() {
  nameEditing.value = true
  nextTick(() => nameInputRef.value?.focus())
}

function finishRename() {
  nameEditing.value = false
}

/** 各类型数据的当前值（用于工具栏操作读写） */
function activeBodyText(): string {
  const r = req.value
  if (!r) return ''
  if (r.body_type === 'raw') return r.body_raw ?? ''
  return r.body
}

async function copyBody() {
  if (!req.value) return
  try {
    await navigator.clipboard.writeText(activeBodyText())
    ElMessage.success('已复制到剪贴板')
  } catch (e) {
    ElMessage.error(`复制失败: ${e}`)
  }
}

/** 格式化 JSON 请求体（非法 JSON 给出错误提示且不改动内容） */
function formatJson() {
  const r = req.value
  if (!r) return
  const text = activeBodyText()
  if (!text.trim()) {
    r.body_raw = '{\n  \n}'
    return
  }
  try {
    const pretty = JSON.stringify(JSON.parse(text), null, 2)
    r.body_raw = pretty
    ElMessage.success('已格式化')
  } catch (e) {
    ElMessage.error(`JSON 格式错误: ${e}`)
  }
}

/** 把 body 里存的 JSON 数组解析为 KV 列表，缺省字段补齐保证旧数据可编辑 */
function parseKvBody(body: string): models.KV[] {
  try {
    const parsed = JSON.parse(body)
    if (Array.isArray(parsed)) {
      const defaults = { key: '', value: '', enable: true }
      return parsed.map(kv => ({ ...defaults, ...kv }))
    }
  } catch {
    // 非法 JSON 视为空表单
  }
  return []
}

/**
 * form 与 form-data 各用独立的本地编辑数组，双向同步到各自的存储字段：
 * form → req.body_form，form-data → req.body。
 * 每种类型的数据永久驻留，切换类型不需要任何搬运，编辑不丢。
 */
const formItems = ref<models.KV[]>([])
const fdItems = ref<models.KV[]>([])
watch(
  () => {
    const r = req.value
    return r && r.body_type === 'form' ? `${r.id}|${r.body_form ?? ''}` : null
  },
  sig => {
    if (sig === null || !req.value) return
    const parsed = parseKvBody(req.value.body_form ?? '')
    if (JSON.stringify(parsed) !== JSON.stringify(formItems.value)) {
      formItems.value = parsed
    }
  },
)
watch(
  formItems,
  items => {
    const r = req.value
    if (!r || r.body_type !== 'form') return
    const json = JSON.stringify(items)
    if ((r.body_form ?? '') !== json) r.body_form = json
  },
  { deep: true },
)
watch(
  () => {
    const r = req.value
    return r && r.body_type === 'form-data' ? `${r.id}|${r.body}` : null
  },
  sig => {
    if (sig === null || !req.value) return
    const parsed = parseKvBody(req.value.body)
    if (JSON.stringify(parsed) !== JSON.stringify(fdItems.value)) {
      fdItems.value = parsed
    }
  },
)
watch(
  fdItems,
  items => {
    const r = req.value
    if (!r || r.body_type !== 'form-data') return
    const json = JSON.stringify(items)
    if (r.body !== json) r.body = json
  },
  { deep: true },
)

/** XML/HTML 缩进格式化：压紧空白后按标签层级缩进 */
function prettyMarkup(src: string): string {
  const compact = src.replace(/>\s+</g, '><').trim()
  const tokens = compact.replace(/></g, '>\n<').split('\n')
  let indent = 0
  return tokens
    .map(raw => {
      const t = raw.trim()
      if (!t) return ''
      // 闭合标签降低层级
      if (/^<\/[\w:.-]+>$/.test(t)) indent = Math.max(0, indent - 1)
      const line = '  '.repeat(indent) + t
      // 非自闭合、非声明/注释的开标签提升层级
      const isOpenTag = /^<[a-zA-Z][^>]*>$/.test(t) && !/\/>$/.test(t)
      if (isOpenTag) indent++
      return line
    })
    .filter(Boolean)
    .join('\n')
}

/** JavaScript 简易缩进：按行首尾的花括号/方括号调整层级 */
function prettyScript(src: string): string {
  let indent = 0
  return src
    .replace(/\r/g, '')
    .split('\n')
    .map(raw => {
      const t = raw.trim()
      if (!t) return ''
      if (/^[}\])]/.test(t)) indent = Math.max(0, indent - 1)
      const line = '  '.repeat(indent) + t
      if (/[{\[]$/.test(t)) indent++
      return line
    })
    .filter(Boolean)
    .join('\n')
}

/** 纯文本整理：去行尾空白、压缩连续空行 */
function prettyText(src: string): string {
  return src
    .replace(/\r/g, '')
    .split('\n')
    .map(l => l.replace(/\s+$/, ''))
    .join('\n')
    .replace(/\n{3,}/g, '\n\n')
}

/** raw 类型按所选内容格式分发格式化（数据源为 body_raw 隔离字段） */
function formatRaw() {
  const r = req.value
  if (!r) return
  const src = r.body_raw ?? ''
  if (!src.trim()) {
    ElMessage.warning('内容为空')
    return
  }
  switch (r.raw_content_type || 'text') {
    case 'json': {
      try {
        r.body_raw = JSON.stringify(JSON.parse(src), null, 2)
        ElMessage.success('已格式化')
      } catch (e) {
        ElMessage.error(`JSON 格式错误: ${e}`)
      }
      return
    }
    case 'xml':
    case 'html':
      r.body_raw = prettyMarkup(src)
      break
    case 'javascript':
      r.body_raw = prettyScript(src)
      break
    default:
      r.body_raw = prettyText(src)
  }
  ElMessage.success('已格式化')
}

/** 行号栏与文本域滚动同步 */
const gutterRef = ref<HTMLElement>()
function syncGutter(e: Event) {
  if (gutterRef.value) gutterRef.value.scrollTop = (e.target as HTMLElement).scrollTop
}

// ---------- URL 查询参数 与 Params 表格双向同步 ----------
// URL 变化 → 解析 ?key=value 到 Params；Params 编辑 → 重写 URL 查询串。
// syncing 标志避免两个 watch 互相触发形成回环。
let syncing = false

function splitUrl(url: string): { base: string; query: string } {
  const i = url.indexOf('?')
  return i === -1
    ? { base: url, query: '' }
    : { base: url.slice(0, i), query: url.slice(i + 1) }
}

/**
 * 查询串片段编码：保留中文与 {{var}} 变量占位符的可读性，
 * 仅转义会破坏 query 结构的字符（& = + # 和空格）。
 * 编码结果可被 URLSearchParams 解码回原值，往返稳定。
 */
function encodeQueryPart(s: string): string {
  return s
    .replace(/&/g, '%26')
    .replace(/=/g, '%3D')
    .replace(/\+/g, '%2B')
    .replace(/#/g, '%23')
    .replace(/ /g, '%20')
}

/** URL 编辑 → Params 补全：解析查询串为 KV 行（未在 Params 中的参数自动补上） */
watch(
  () => {
    const r = req.value
    return r ? `${r.id}|${r.url}` : null
  },
  sig => {
    if (sig === null || !req.value || syncing) return
    const r = req.value
    const { query } = splitUrl(r.url)
    const pairs = Array.from(new URLSearchParams(query) as unknown as Iterable<[string, string]>)
      .filter(([k]) => k !== '')
    // 就地合并：地址里的参数与表格同名启用行复用同一行对象（仅值有差异才更新），
    // 避免切回选项卡触发同步时整表重建成编辑器形状的行（丢失 file 等字段），
    // 导致内容与保存快照不一致而误标「未保存」
    const pool = r.params.filter(kv => kv.enable)
    const parsed: models.KV[] = []
    for (const [key, value] of pairs) {
      const idx = pool.findIndex(kv => kv.key === key)
      if (idx >= 0) {
        const kv = pool.splice(idx, 1)[0]
        if (kv.value !== value) kv.value = value // 地址为准
        parsed.push(kv)
      } else {
        parsed.push({ key, value, enable: true })
      }
    }
    // 保留 URL 中无法体现的行：禁用行不拼进地址；空 key 占位行（KVEditor 为空表
    // 自动补的行）同样不在地址里，若丢弃会在切回选项卡时被 KVEditor 重新补回，
    // 行对象重建导致与保存快照不一致而误标「未保存」，故与禁用行一并原样保留；
    // 在地址里删掉的启用参数，表格行同步删除，避免下次编辑又“复活”回地址
    const known = new Set(pairs.map(([k]) => k))
    for (const kv of r.params) {
      if ((!kv.key || !kv.enable) && !known.has(kv.key)) parsed.push({ ...kv })
    }
    if (JSON.stringify(parsed) !== JSON.stringify(r.params)) {
      syncing = true
      r.params = parsed
      nextTick(() => (syncing = false))
    }
  },
)

/** Params 编辑 → URL 重写：查询串由启用的参数行重建 */
watch(
  () => req.value?.params,
  items => {
    const r = req.value
    if (!r || syncing) return
    const parts: string[] = []
    for (const kv of items ?? []) {
      if (kv.enable && kv.key) parts.push(`${encodeQueryPart(kv.key)}=${encodeQueryPart(kv.value)}`)
    }
    const qs = parts.join('&')
    const next = qs ? `${splitUrl(r.url).base}?${qs}` : splitUrl(r.url).base
    if (next !== r.url) {
      syncing = true
      r.url = next
      nextTick(() => (syncing = false))
    }
  },
  { deep: true },
)

// Ctrl+S 保存当前请求选项卡（拦截 WebView 自带的“保存网页”默认行为）
function onKeydown(e: KeyboardEvent) {
  if (e.ctrlKey && !e.altKey && !e.shiftKey && e.key.toLowerCase() === 's') {
    e.preventDefault()
    save()
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="request-area">
    <div v-if="!req" class="empty">点击左侧请求以选项卡打开，或点击集合旁的“+ 请求”新建</div>
    <div v-else style="padding: 10px;">
      <div style="display: flex; gap: 8px; margin-bottom: 8px; align-items: center;">
        <el-input
          v-if="nameEditing"
          ref="nameInputRef"
          v-model="req.name"
          size="small"
          placeholder="请求名称"
          style="width: 220px;"
          @blur="finishRename"
          @keyup.enter="finishRename"
        />
        <template v-else>
          <span class="req-name" :title="'点击修改请求名称'">{{ req.name || '未命名' }}</span>
          <el-button size="small" text :icon="Edit" @click="startRename">修改</el-button>
        </template>
        <span style="flex: 1;"></span>
      </div>
      <div style="display: flex; gap: 8px; margin-bottom: 8px;">
        <el-select v-model="req.method" size="small" style="width: 110px;">
          <el-option v-for="m in methods" :key="m" :label="m" :value="m" />
        </el-select>
        <el-input v-model="req.url" size="small" placeholder="请求 URL，支持 {{var}} 变量替换" style="flex: 1;" />
        <el-button v-if="runtime.sending(req.id)" type="danger" size="small" @click="runtime.cancel(req.id)">取消</el-button>
        <el-button v-else type="primary" size="small" @click="send">发送</el-button>
        <el-button size="small" @click="save">保存</el-button>
      </div>
      <el-tabs v-model="activePane">
        <el-tab-pane label="Params" name="params">
          <KVEditor :items="req.params" :placeholder="isNewReq" />
        </el-tab-pane>
        <el-tab-pane label="Auth" name="auth">
          <div class="auth-form">
            <el-select v-model="req.auth.auth_type" size="small" class="auth-type">
              <el-option label="No Auth" value="none" />
              <el-option label="Bearer Token" value="bearer" />
              <el-option label="Basic Auth" value="basic" />
              <el-option label="API Key" value="api_key" />
            </el-select>
            <template v-if="req.auth.auth_type === 'bearer'">
              <div class="auth-row">
                <el-input
                  v-model="req.auth.token"
                  size="small"
                  placeholder="Token，支持变量替换"
                  class="auth-field"
                  :disabled="isAuthLocked('token')"
                />
                <el-icon class="auth-edit" @click="toggleAuthEdit('token')">
                  <Check v-if="!isAuthLocked('token')" />
                  <Edit v-else />
                </el-icon>
              </div>
            </template>
            <template v-else-if="req.auth.auth_type === 'basic'">
              <div class="auth-row">
                <el-input
                  v-model="req.auth.username"
                  size="small"
                  placeholder="Username，支持变量替换"
                  class="auth-field"
                  :disabled="isAuthLocked('username')"
                />
                <el-icon class="auth-edit" @click="toggleAuthEdit('username')">
                  <Check v-if="!isAuthLocked('username')" />
                  <Edit v-else />
                </el-icon>
              </div>
              <div class="auth-row">
                <el-input
                  v-model="req.auth.password"
                  size="small"
                  show-password
                  placeholder="Password，支持变量替换"
                  class="auth-field"
                  :disabled="isAuthLocked('password')"
                />
                <el-icon class="auth-edit" @click="toggleAuthEdit('password')">
                  <Check v-if="!isAuthLocked('password')" />
                  <Edit v-else />
                </el-icon>
              </div>
            </template>
            <template v-else-if="req.auth.auth_type === 'api_key'">
              <div class="auth-row">
                <el-input
                  v-model="req.auth.key"
                  size="small"
                  placeholder="Key（如 x-api-key），支持变量替换"
                  class="auth-field"
                  :disabled="isAuthLocked('key')"
                />
                <el-icon class="auth-edit" @click="toggleAuthEdit('key')">
                  <Check v-if="!isAuthLocked('key')" />
                  <Edit v-else />
                </el-icon>
              </div>
              <div class="auth-row">
                <el-input
                  v-model="req.auth.value"
                  size="small"
                  placeholder="Value，支持变量替换"
                  class="auth-field"
                  :disabled="isAuthLocked('value')"
                />
                <el-icon class="auth-edit" @click="toggleAuthEdit('value')">
                  <Check v-if="!isAuthLocked('value')" />
                  <Edit v-else />
                </el-icon>
              </div>
              <el-radio-group v-model="req.auth.location" size="small">
                <el-radio-button value="header">Header</el-radio-button>
                <el-radio-button value="query">Query</el-radio-button>
              </el-radio-group>
            </template>
            <div v-else class="auth-hint">此请求不使用认证</div>
          </div>
        </el-tab-pane>
        <el-tab-pane label="Headers" name="headers">
          <KVEditor :items="req.headers" :placeholder="isNewReq" />
        </el-tab-pane>
        <el-tab-pane label="Body" name="body">
          <el-radio-group
            :model-value="req.body_type"
            size="small"
            style="margin-bottom: 8px;"
            @update:model-value="switchBodyType"
          >
            <el-radio-button v-for="b in bodyTypes" :key="b.value" :value="b.value">{{ b.label }}</el-radio-button>
          </el-radio-group>
          <template v-if="req.body_type === 'raw'">
            <div class="body-toolbar">
              <span class="raw-ct-label">类型</span>
              <el-select
                :model-value="req.raw_content_type || 'text'"
                size="small"
                style="width: 110px;"
                @update:model-value="req.raw_content_type = $event"
              >
                <el-option v-for="t in rawContentTypes" :key="t.value" :label="t.label" :value="t.value" />
              </el-select>
              <span style="flex: 1;"></span>
              <el-button size="small" link @click="formatRaw">
                <el-icon><MagicStick /></el-icon>&nbsp;格式化
              </el-button>
              <el-button size="small" link @click="copyBody">
                <el-icon><CopyDocument /></el-icon>&nbsp;复制
              </el-button>
            </div>
            <div class="code-editor">
              <div ref="gutterRef" class="code-gutter mono">
                <div v-for="n in bodyLineCount" :key="n" class="code-ln">{{ n }}</div>
              </div>
              <textarea
                v-model="req.body_raw"
                class="code-textarea mono"
                spellcheck="false"
                placeholder="请求体内容，支持 {{var}}"
                @scroll="syncGutter"
              ></textarea>
            </div>
          </template>
          <template v-else-if="req.body_type === 'form'">
            <div style="color: var(--muted); margin-bottom: 6px;">
              form 模式：以表单形式填写键值对，发送时以 application/x-www-form-urlencoded 提交
            </div>
            <KVEditor :items="formItems" :placeholder="isNewReq" />
          </template>
          <template v-else-if="req.body_type === 'form-data'">
            <div style="color: var(--muted); margin-bottom: 6px;">
              form-data 模式：每一行可切换「文本 / 文件」，文件行发送时以 multipart/form-data 上传
            </div>
            <FormDataEditor :items="fdItems" :placeholder="isNewReq" />
          </template>
          <div v-else style="color: var(--muted);">该请求没有请求体</div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<style scoped>
/* 请求区自适应分割面板高度：纵向弹性布局，让编辑器随面板拉伸填满 */
.request-area {
  display: flex;
  flex-direction: column;
}
.request-area > div {
  display: flex;
  flex-direction: column;
  flex: 1 1 auto;
  min-height: 0;
}
.request-area > div > .el-tabs {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.request-area :deep(.el-tabs__content) {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}
/* tab 面板本身也弹性撑满，json 编辑框边框才能紧贴分割面板边界伸缩 */
.request-area :deep(.el-tabs__content > .el-tab-pane) {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.empty { color: #909399; text-align: center; padding: 40px 0; }

/* Auth 页签：类型下拉 + 条件字段，纵向窄栏排布 */
.auth-form {
  padding: 8px 0;
  max-width: 480px;
}
.auth-type {
  width: 160px;
  margin-bottom: 8px;
  display: block;
}
.auth-field {
  margin-bottom: 8px;
}
/* Auth 字段行：输入框 + 修改锁图标（有值默认锁定，与 KVEditor 一致） */
.auth-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}
.auth-row .auth-field {
  margin-bottom: 0;
  flex: 1 1 auto;
}
.auth-edit {
  margin-left: 6px;
  color: #909399;
  cursor: pointer;
  flex: 0 0 auto;
}
.auth-edit:hover {
  color: var(--el-color-primary);
}
.auth-hint {
  color: #909399;
  font-size: 12px;
}

/* 压缩请求区内部 tabs（Params/Headers/Body）标题尺寸：高度由 --el-tabs-header-height 变量控制，覆盖变量才生效 */
.request-area :deep(.el-tabs) { --el-tabs-header-height: 28px; }
.request-area :deep(.el-tabs__header) { margin: 0 0 6px; }
.request-area :deep(.el-tabs__item) {
  padding: 0 12px;
  font-size: 11px;
}

/* 只读请求名称：与 small 输入框视觉对齐 */
.req-name {
  font-size: 13px;
  color: #303133;
  padding: 0 8px;
  line-height: 24px;
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: inline-block;
  cursor: default;
}
.body-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 10px;
  background: #f5f7fa;
  border: 1px solid #dcdfe6;
  border-bottom: none;
  border-radius: 4px 4px 0 0;
  margin-bottom: -1px;
}
.body-toolbar .line-count {
  font-size: 12px;
  color: #909399;
}
.raw-ct-label {
  font-size: 12px;
  color: #909399;
}
/* JSON 编辑器：左侧行号栏 + 等宽文本域，高度自适应分割面板（弹性拉伸，最小 160px） */
.code-editor {
  display: flex;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  flex: 1 1 auto;
  min-height: 160px;
  overflow: hidden;
}
.code-gutter {
  flex: 0 0 40px;
  padding: 8px 6px 8px 0;
  text-align: right;
  background: #f5f7fa;
  border-right: 1px solid var(--border);
  color: #909399;
  overflow: hidden;
  font-size: 12px;
  line-height: 20px;
  user-select: none;
}
.code-textarea {
  flex: 1;
  border: none;
  outline: none;
  resize: none;
  padding: 8px;
  font-size: 12px;
  line-height: 20px;
}
</style>
