<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { CaretLeft, CaretRight } from '@element-plus/icons-vue'
import EnvironmentBar from '@/components/EnvironmentBar.vue'
import SidebarCollection from '@/components/SidebarCollection.vue'
import RequestTabs from '@/components/RequestTabs.vue'
import RequestEditor from '@/components/RequestEditor.vue'
import ResponseViewer from '@/components/ResponseViewer.vue'
import RequestSettingsDialog from '@/components/RequestSettingsDialog.vue'
import { useCollectionStore } from '@/stores/collection'
import { useEnvironmentStore } from '@/stores/environment'
import { useThemeStore } from '@/stores/theme'
import { useLayoutStore } from '@/stores/layout'
import { ImportAll, SyncThemeChecks } from '@/api/wails'
import { ElMessage, ElMessageBox } from 'element-plus'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const colStore = useCollectionStore()
const envStore = useEnvironmentStore()
const theme = useThemeStore()
const layoutStore = useLayoutStore()

onMounted(() => {
  colStore.loadAll()
  envStore.load()
  // 挂载后把当前主题同步到原生菜单勾选项
  SyncThemeChecks(theme.mode).catch(() => {})
})

const sidebarCollapsed = ref(false)
const importInput = ref<HTMLInputElement>()
const requestSettingsVisible = ref(false)

// ---------- 请求/响应分割面板拖拽 ----------
const splitContainerRef = ref<HTMLDivElement>()
function onSplitMouseDown(e: MouseEvent) {
  const container = splitContainerRef.value
  if (!container) return
  layoutStore.setSplitDragging(true)
  const rect = container.getBoundingClientRect()
  const move = (ev: MouseEvent) => {
    const pct = ((ev.clientY - rect.top) / rect.height) * 100
    layoutStore.setRequestPct(pct)
  }
  const up = () => {
    layoutStore.setSplitDragging(false)
    window.removeEventListener('mousemove', move)
    window.removeEventListener('mouseup', up)
  }
  window.addEventListener('mousemove', move)
  window.addEventListener('mouseup', up)
  e.preventDefault()
}

async function onImport() {
  importInput.value?.click()
}
async function onFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  const text = await file.text()
  await ImportAll(text)
  await colStore.loadAll()
  await envStore.load()
  ElMessage.success('导入完成')
  ;(e.target as HTMLInputElement).value = ''
}

// ---------- 原生顶层菜单事件 ----------
const unlisteners: UnlistenFn[] = []

async function menuNewCollection() {
  let value: string
  try {
    ({ value } = await ElMessageBox.prompt('请输入集合名称', '新建集合', {
      confirmButtonText: '确定', cancelButtonText: '取消',
    }))
  } catch {
    return // 用户取消
  }
  if (value && value.trim()) {
    await colStore.addCollection(value.trim(), null)
    ElMessage.success('已创建')
  }
}

function menuAbout() {
  ElMessageBox.alert(
    'API 教研接口测试工具<br/>版本 0.3.0<br/>支持集合管理、环境变量、请求调试与自动化测试',
    '关于软件',
    { dangerouslyUseHTMLString: true, confirmButtonText: '确定' },
  ).catch(() => {})
}

function menuAboutAuthor() {
  ElMessageBox.alert(
    '<b>作者：ham</b><br/>个人 AI 开发者<br/>专注于 AI 辅助的桌面工具开发',
    '关于作者',
    { dangerouslyUseHTMLString: true, confirmButtonText: '确定' },
  ).catch(() => {})
}

onMounted(async () => {
  // 原生菜单动作（文件/帮助）
  unlisteners.push(await listen<string>('menu://action', event => {
    switch (event.payload) {
      case 'menu://new-collection': menuNewCollection(); break
      case 'menu://import-collection': onImport(); break
      case 'menu://about': menuAbout(); break
      case 'menu://request-settings': requestSettingsVisible.value = true; break
      case 'menu://about-author': menuAboutAuthor(); break
    }
  }))
  // 原生菜单主题选择：切换（store 内会同步勾选状态）
  unlisteners.push(await listen<string>('menu://theme', event => {
    theme.setMode(event.payload as typeof theme.mode)
  }))
})

onUnmounted(() => unlisteners.forEach(fn => fn()))
</script>

<template>
  <div class="layout">
    <input ref="importInput" type="file" accept=".json" style="display: none" @change="onFile" />
    <RequestSettingsDialog v-model="requestSettingsVisible" />
    <EnvironmentBar @import="onImport" />
    <div class="body-area">
      <div class="sidebar-wrap" :class="{ collapsed: sidebarCollapsed }">
        <button
          class="sidebar-toggle"
          :title="sidebarCollapsed ? '展开侧栏' : '折叠侧栏'"
          @click="sidebarCollapsed = !sidebarCollapsed"
        >
          <el-icon><CaretLeft v-if="!sidebarCollapsed" /><CaretRight v-else /></el-icon>
        </button>
        <SidebarCollection @import="onImport" />
      </div>
      <div class="workspace">
        <RequestTabs />
        <div
          ref="splitContainerRef"
          class="split-pane"
          :class="{ dragging: layoutStore.splitDragging }"
        >
          <div class="split-top" :style="{ height: layoutStore.requestPct + '%' }">
            <RequestEditor />
          </div>
          <div class="split-handle" @mousedown="onSplitMouseDown">
            <div class="split-handle-grip"></div>
          </div>
          <div class="split-bottom">
            <ResponseViewer />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 垂直分割面板：请求区 + 拖拽手柄 + 响应区 */
.split-pane {
  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.split-top {
  flex: 0 0 auto;
  min-height: 0;
  overflow: hidden;
}
.split-bottom {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}
.split-handle {
  flex: 0 0 6px;
  height: 6px;
  cursor: row-resize;
  background: var(--bg);
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
  position: relative;
  transition: background 0.15s ease;
}
.split-handle:hover,
.split-pane.dragging .split-handle {
  background: var(--el-color-primary);
}
.split-handle-grip {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 40px;
  height: 2px;
  border-radius: 2px;
  background: var(--muted);
  pointer-events: none;
}
.split-handle:hover .split-handle-grip,
.split-pane.dragging .split-handle-grip {
  background: #fff;
}
/* 拖拽中：避免 iframe 等元素拦截鼠标移动，并禁用文本选中 */
.split-pane.dragging {
  user-select: none;
}
</style>
