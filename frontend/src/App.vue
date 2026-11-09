<script setup lang="ts">
import { onMounted, ref } from 'vue'
import EnvironmentBar from '@/components/EnvironmentBar.vue'
import SidebarCollection from '@/components/SidebarCollection.vue'
import RequestEditor from '@/components/RequestEditor.vue'
import ResponseViewer from '@/components/ResponseViewer.vue'
import { useCollectionStore } from '@/stores/collection'
import { useEnvironmentStore } from '@/stores/environment'
import { ExportAll, ImportAll } from '@/api/wails'
import { ElMessage } from 'element-plus'

const colStore = useCollectionStore()
const envStore = useEnvironmentStore()

onMounted(() => {
  colStore.loadCollections()
  envStore.load()
})

async function onExport() {
  const json = await ExportAll()
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'api-dev-export.json'
  a.click()
  URL.revokeObjectURL(url)
  ElMessage.success('已导出')
}

const importInput = ref<HTMLInputElement>()
async function onImport() {
  importInput.value?.click()
}
async function onFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  const text = await file.text()
  await ImportAll(text)
  await colStore.loadCollections()
  await envStore.load()
  ElMessage.success('导入完成')
  ;(e.target as HTMLInputElement).value = ''
}
</script>

<template>
  <div class="layout">
    <input ref="importInput" type="file" accept=".json" style="display: none" @change="onFile" />
    <EnvironmentBar @export="onExport" @import="onImport" />
    <div class="body-area">
      <SidebarCollection />
      <div class="workspace">
        <RequestEditor />
        <ResponseViewer />
      </div>
    </div>
  </div>
</template>
