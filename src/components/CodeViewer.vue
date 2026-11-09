<script setup lang="ts">
import { computed } from 'vue'
import { ElMessage } from 'element-plus'
import { CopyDocument } from '@element-plus/icons-vue'

const props = defineProps<{
  content: string
  maxHeight?: string
}>()

const lines = computed(() => props.content.split('\n'))
const lineNumbers = computed(() => lines.value.map((_, i) => i + 1))

async function copyContent() {
  try {
    await navigator.clipboard.writeText(props.content)
    ElMessage.success('已复制到剪贴板')
  } catch (e) {
    ElMessage.error(`复制失败: ${e}`)
  }
}
</script>

<template>
  <div class="code-viewer">
    <div class="code-toolbar">
      <span class="line-count">{{ lineNumbers.length }} 行</span>
      <el-button size="small" link @click="copyContent">
        <el-icon><CopyDocument /></el-icon>&nbsp;复制
      </el-button>
    </div>
    <div class="code-body" :style="{ maxHeight: maxHeight || '320px' }">
      <div class="line-numbers">
        <div v-for="num in lineNumbers" :key="num" class="line-number">{{ num }}</div>
      </div>
      <pre class="code-content">{{ content }}</pre>
    </div>
  </div>
</template>

<style scoped>
.code-viewer {
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  overflow: hidden;
  background: #fafafa;
}

.code-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 10px;
  background: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
}

.line-count {
  font-size: 12px;
  color: #909399;
}

.code-body {
  display: flex;
  overflow: auto;
  font-family: 'Courier New', Courier, monospace;
  font-size: 13px;
  line-height: 1.5;
}

.line-numbers {
  flex-shrink: 0;
  padding: 10px 8px;
  background: #f5f7fa;
  border-right: 1px solid #e4e7ed;
  color: #909399;
  text-align: right;
  user-select: none;
  min-width: 40px;
}

.line-number {
  height: 1.5em;
}

.code-content {
  flex: 1;
  margin: 0;
  padding: 10px;
  white-space: pre;
  word-wrap: break-word;
}
</style>
