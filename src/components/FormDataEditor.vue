<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { models } from '@/api/wails'

const props = defineProps<{ items: models.KV[] }>()

function add() {
  props.items.push({ key: '', value: '', enable: true })
}
function remove(i: number) {
  props.items.splice(i, 1)
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
      <el-checkbox v-model="kv.enable" />
      <el-input v-model="kv.key" size="small" placeholder="参数名" style="width: 180px;" />
      <el-input
        v-if="kv.file"
        v-model="kv.value"
        size="small"
        placeholder="文件路径（点击右侧按钮选择）"
        style="flex: 1;"
        class="mono"
      />
      <el-input
        v-else
        v-model="kv.value"
        size="small"
        placeholder="文本值，支持 {{var}}"
        style="flex: 1;"
      />
      <el-tooltip content="文本/文件切换" placement="top">
        <el-switch
          v-model="kv.file"
          active-text="文件"
          inactive-text="文本"
          style="--el-switch-on-color: #67c23a;"
        />
      </el-tooltip>
      <el-button size="small" :disabled="!kv.file" @click="pickFile(i)">选择文件</el-button>
      <el-button size="small" link @click="remove(i)">×</el-button>
    </div>
    <el-button size="small" link @click="add">+ 添加一行</el-button>
  </div>
</template>
