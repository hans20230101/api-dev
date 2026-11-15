<script setup lang="ts">
import { computed } from 'vue'
import { ElMessage } from 'element-plus'
import { save } from '@tauri-apps/plugin-dialog'
import { SaveResponseBody, models } from '@/api/wails'
import { useRuntimeStore } from '@/stores/runtime'
import { useTabsStore } from '@/stores/tabs'
import { statusTagType } from '@/utils'
import CodeViewer from './CodeViewer.vue'

const runtime = useRuntimeStore()
const tabs = useTabsStore()

// 响应与发送状态跟随当前激活的请求选项卡
const response = computed(() => {
  const id = tabs.activeTab?.id
  return id == null ? null : runtime.responses[id] ?? null
})
const sending = computed(() => {
  const id = tabs.activeTab?.id
  return id != null && runtime.sending(id)
})

// 是否属于「下载型」接口：响应为二进制、带 Content-Disposition 文件名、
// 或请求名称含「下载」—— 只有这类接口才显示「保存文件」按钮
const isDownload = computed(() => {
  const resp = response.value
  if (!resp) return false
  if (resp.is_binary || resp.filename) return true
  return (tabs.activeTab?.name ?? '').includes('下载')
})

// Content-Type 对应的默认扩展名
const MIME_EXT: Record<string, string> = {
  'application/json': 'json',
  'application/pdf': 'pdf',
  'application/xml': 'xml',
  'application/zip': 'zip',
  'application/gzip': 'gz',
  'application/octet-stream': 'bin',
  'image/png': 'png',
  'image/jpeg': 'jpg',
  'image/gif': 'gif',
  'image/svg+xml': 'svg',
  'image/webp': 'webp',
  'audio/mpeg': 'mp3',
  'video/mp4': 'mp4',
}

function guessExtension(ct?: string): string {
  if (!ct) return 'bin'
  const mime = ct.split(';')[0].trim().toLowerCase()
  if (MIME_EXT[mime]) return MIME_EXT[mime]
  if (mime.startsWith('text/')) return 'txt'
  return 'bin'
}

// 清洗文件名：去掉 Windows 非法字符与路径分隔符，避免保存对话框拒绝
function sanitizeName(name: string): string {
  const cleaned = name
    .replace(/[<>:"/\\|?*\x00-\x1f]/g, '')
    .replace(/^[.\s]+|[.\s]+$/g, '')
  return cleaned || ''
}

// 保存时的默认文件名：Content-Disposition → URL 末段 → content-type 推断
function defaultFilename(resp: models.Response): string {
  const name = resp.filename?.trim()
  if (name) return sanitizeName(name) || `response.${guessExtension(resp.content_type)}`
  const url = tabs.activeTab?.url
  if (url) {
    try {
      const path = new URL(url).pathname.split('/').filter(Boolean).pop()
      if (path) {
        const decoded = decodeURIComponent(path)
        if (/\.[A-Za-z0-9]+$/.test(decoded)) return sanitizeName(decoded)
      }
    } catch {
      // URL 无法解析时忽略，继续走扩展名推断
    }
  }
  return `response.${guessExtension(resp.content_type)}`
}

async function saveFile() {
  const resp = response.value
  if (!resp || resp.body_id == null) return
  let filePath: string | null
  try {
    filePath = await save({
      title: '保存响应',
      defaultPath: defaultFilename(resp),
    })
  } catch (e) {
    // 弹框阶段失败：多为默认目录不可写或文件名非法
    ElMessage.error(`打开保存对话框失败: ${e}`)
    return
  }
  if (!filePath) return
  try {
    await SaveResponseBody(resp.body_id, filePath)
    ElMessage.success('已保存')
  } catch (e) {
    // 写盘阶段失败：后端会返回「写入文件失败: ...」
    ElMessage.error(`保存失败: ${e}`)
  }
}
</script>

<template>
  <div class="response-area">
    <div v-if="sending" class="empty">请求发送中...</div>
    <div v-else-if="!response" class="empty">点击「发送」查看响应</div>
    <div v-else style="padding: 10px;">
      <div v-if="response.error" style="color: #f56c6c;">
        请求失败：{{ response.error }}
      </div>
      <template v-else>
        <div style="display: flex; gap: 16px; margin-bottom: 8px; align-items: center;">
          <span>响应</span>
          <el-tag :type="statusTagType(response.status_code)">状态 {{ response.status_code }}</el-tag>
          <span>耗时 {{ response.time_ms }} ms</span>
          <span>响应大小 {{ response.resp_size }} B</span>
          <span>请求大小 {{ response.req_size }} B</span>
          <el-button
            v-if="isDownload"
            size="small"
            type="primary"
            plain
            style="margin-left: auto;"
            @click="saveFile"
          >
            保存文件
          </el-button>
        </div>
        <el-tabs>
          <el-tab-pane label="Body">
            <div v-if="response.is_binary" class="binary-hint">
              <p>二进制内容（{{ response.resp_size }} B），无法以文本显示。</p>
              <el-button type="primary" @click="saveFile">保存文件</el-button>
            </div>
            <CodeViewer v-else :content="response.body" fill />
          </el-tab-pane>
          <el-tab-pane label="Headers">
            <div v-for="(v, k) in response.headers" :key="k" class="kv-row mono">
              <strong style="min-width: 180px;">{{ k }}:</strong>
              <span>{{ v }}</span>
            </div>
          </el-tab-pane>
        </el-tabs>
      </template>
    </div>
  </div>
</template>

<style scoped>
/* 响应区自适应分割面板高度：内容纵向弹性布局，Body 查看器随面板拉伸填满 */
.response-area {
  display: flex;
  flex-direction: column;
}
.response-area > div {
  display: flex;
  flex-direction: column;
  flex: 1 1 auto;
  min-height: 0;
}
.response-area :deep(.el-tabs) {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.response-area :deep(.el-tabs__content) {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}
/* tab 面板本身也弹性撑满，Body 查看器边框才能紧贴分割面板边界伸缩 */
.response-area :deep(.el-tabs__content > .el-tab-pane) {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* 压缩响应区内部 tabs（Body/Headers）标题尺寸：高度由 --el-tabs-header-height 变量控制，覆盖变量才生效 */
.response-area :deep(.el-tabs) { --el-tabs-header-height: 28px; }
.response-area :deep(.el-tabs__header) { margin: 0 0 6px; }
.response-area :deep(.el-tabs__item) {
  padding: 0 12px;
  font-size: 11px;
}

.empty { color: #909399; text-align: center; padding: 30px 0; }
.binary-hint { color: #909399; text-align: center; padding: 40px 0; }
.binary-hint p { margin-bottom: 12px; }
</style>
