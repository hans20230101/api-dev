<script setup lang="ts">
import { onMounted, ref } from 'vue'
import EnvironmentBar from '@/components/EnvironmentBar.vue'
import SidebarCollection from '@/components/SidebarCollection.vue'
import RequestTabs from '@/components/RequestTabs.vue'
import RequestEditor from '@/components/RequestEditor.vue'
import ResponseViewer from '@/components/ResponseViewer.vue'
import { useCollectionStore } from '@/stores/collection'
import { useEnvironmentStore } from '@/stores/environment'
import { ImportAll } from '@/api/wails'
import { ElMessage } from 'element-plus'

const colStore = useCollectionStore()
const envStore = useEnvironmentStore()

onMounted(() => {
  colStore.loadAll()
  envStore.load()
})

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
</script>

<template>
  <div class="layout">
    <input ref="importInput" type="file" accept=".json" style="display: none" @change="onFile" />
    <EnvironmentBar @import="onImport" />
    <div class="body-area">
      <SidebarCollection />
      <div class="workspace">
        <RequestTabs />
        <RequestEditor />
        <ResponseViewer />
      </div>
    </div>
  </div>
</template>
