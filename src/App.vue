<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { CaretLeft, CaretRight } from '@element-plus/icons-vue'
import EnvironmentBar from '@/components/EnvironmentBar.vue'
import SidebarCollection from '@/components/SidebarCollection.vue'
import RequestTabs from '@/components/RequestTabs.vue'
import RequestEditor from '@/components/RequestEditor.vue'
import ResponseViewer from '@/components/ResponseViewer.vue'
import { useCollectionStore } from '@/stores/collection'
import { useEnvironmentStore } from '@/stores/environment'
import { useThemeStore } from '@/stores/theme'
import { ImportAll, SyncThemeChecks } from '@/api/wails'
import { ElMessage, ElMessageBox } from 'element-plus'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const colStore = useCollectionStore()
const envStore = useEnvironmentStore()
const theme = useThemeStore()

onMounted(() => {
  colStore.loadAll()
  envStore.load()
  // 挂载后把当前主题同步到原生菜单勾选项
  SyncThemeChecks(theme.mode).catch(() => {})
})

const sidebarCollapsed = ref(false)
const importInput = ref<HTMLInputElement>()
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
    'API 教研接口测试工具<br/>版本 0.1.0<br/>支持集合管理、环境变量、请求调试与自动化测试',
    '关于',
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
        <RequestEditor />
        <ResponseViewer />
      </div>
    </div>
  </div>
</template>
