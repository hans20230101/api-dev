<script setup lang="ts">
import { ref } from 'vue'
import { useCollectionStore } from '@/stores/collection'
import { useRuntimeStore } from '@/stores/runtime'
import { statusTagType } from '@/utils'

const colStore = useCollectionStore()
const runtime = useRuntimeStore()
const visible = ref(false)

async function run() {
  if (colStore.selectedCollectionId == null) return
  await runtime.runCollection(colStore.selectedCollectionId)
}

const passedCount = () => runtime.testResults.filter(r => r.passed).length
</script>

<template>
  <el-button size="small" @click="visible = true">
    <el-icon><CaretRight /></el-icon>&nbsp;批量测试
  </el-button>
  <el-dialog v-model="visible" title="集合批量测试" width="820px">
    <div style="margin-bottom: 10px; display: flex; align-items: center; gap: 12px;">
      <el-button type="primary" size="small" :loading="runtime.running" @click="run">运行当前集合</el-button>
      <span v-if="colStore.selectedCollectionId == null" style="color: var(--muted);">请先在左侧选择集合</span>
      <span v-else-if="runtime.testResults.length" style="color: var(--muted);">
        共 {{ runtime.testResults.length }} 条，通过 {{ passedCount() }} 条
      </span>
    </div>
    <el-table :data="runtime.testResults" size="small" border max-height="420">
      <el-table-column label="结果" width="80">
        <template #default="{ row }">
          <el-tag :type="row.passed ? 'success' : 'danger'" size="small">{{ row.passed ? '通过' : '失败' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="name" label="请求" min-width="140" />
      <el-table-column label="状态码" width="80">
        <template #default="{ row }">
          <el-tag v-if="row.status_code" :type="statusTagType(row.status_code)" size="small" effect="plain">{{ row.status_code }}</el-tag>
          <span v-else>-</span>
        </template>
      </el-table-column>
      <el-table-column prop="time_ms" label="耗时(ms)" width="90" />
      <el-table-column prop="asserts" label="断言数" width="80" />
      <el-table-column prop="failed_msg" label="失败原因" min-width="160" />
      <el-table-column prop="error" label="错误" min-width="160" />
    </el-table>
  </el-dialog>
</template>
