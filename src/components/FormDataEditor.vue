<script setup lang="ts">
import { watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { models } from '@/api/wails'

const props = defineProps<{ items: models.KV[] }>()

// 空列表时默认显示一行空白行，方便直接开始填写；该行 key 为空不会参与发送
watch(
  () => props.items,
  items => {
    if (items.length === 0) items.push({ key: '', value: '', enable: true })
  },
  { immediate: true },
)

function add() {
  props.items.push({ key: '', value: '', enable: true })
}
function remove(i: number) {
  props.items.splice(i, 1)
}

// 参数值类型：文本 / 文件（存储沿用 kv.file 布尔字段，默认文本）
const valueTypeOptions = [
  { label: '文本', value: 'text' },
  { label: '文件', value: 'file' },
]
function valueTypeOf(kv: models.KV): string {
  return kv.file ? 'file' : 'text'
}
function setValueType(kv: models.KV, type: string) {
  kv.file = type === 'file'
  // 切到文件时清空旧文本内容，避免把文本误当文件路径发送
  if (kv.file) kv.value = ''
}

async function pickFile(i: number) {
  const selected = await open({
    title: '选择上传文件',
    multiple: false,
  })
  if (typeof selected === 'string') {
    props.items[i].value = selected
  }
}
</script>

<template>
  <div>
    <div v-for="(kv, i) in items" :key="i" class="kv-row">
      <el-checkbox v-model="kv.enable" class="kv-enable" />
      <el-input v-model="kv.key" size="small" placeholder="参数名" class="kv-key" />
      <!-- 复合输入框：prepend 是 文本/文件 类型下拉；文本时右侧为输入框，文件时右侧为选择文件按钮 -->
      <div v-if="!kv.file" class="composite">
        <el-select
          :model-value="valueTypeOf(kv)"
          size="small"
          class="composite-type"
          @update:model-value="setValueType(kv, $event)"
        >
          <el-option v-for="t in valueTypeOptions" :key="t.value" :label="t.label" :value="t.value" />
        </el-select>
        <el-input v-model="kv.value" size="small" placeholder="文本值，支持 {{var}}" style="flex: 1;" />
      </div>
      <div v-else class="composite">
        <el-select
          :model-value="valueTypeOf(kv)"
          size="small"
          class="composite-type"
          @update:model-value="setValueType(kv, $event)"
        >
          <el-option v-for="t in valueTypeOptions" :key="t.value" :label="t.label" :value="t.value" />
        </el-select>
        <el-button size="small" style="flex: 1;" @click="pickFile(i)">选择文件</el-button>
      </div>
      <el-button size="small" link class="kv-remove" @click="remove(i)">×</el-button>
    </div>
    <el-button size="small" link @click="add">+ 添加一行</el-button>
  </div>
</template>

<style scoped>
/* 行内布局与 KVEditor 对齐：参数名/复合值区固定首选宽度，宽面板下保持紧凑；
   删除按钮 margin-left:auto 保持贴右 */
.kv-enable {
  flex: 0 0 auto;
}
.kv-key {
  flex: 0 1 180px;
  min-width: 110px;
}
.kv-remove {
  flex: 0 0 auto;
  margin-left: auto;
}
/* 复合输入框：类型下拉与值输入/文件按钮无缝连成一体 */
.composite {
  display: flex;
  flex: 0 1 340px;
  min-width: 180px;
  gap: 0;
}
.composite-type {
  width: 90px;
  flex: 0 0 auto;
}
</style>
