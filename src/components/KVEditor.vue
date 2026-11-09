<script setup lang="ts">
import { models } from '@/api/wails'

const props = defineProps<{ items: models.KV[] }>()

function add() {
  props.items.push({ key: '', value: '', enable: true })
}
function remove(i: number) {
  props.items.splice(i, 1)
}
</script>

<template>
  <div>
    <div v-for="(kv, i) in items" :key="i" class="kv-row">
      <el-checkbox v-model="kv.enable" />
      <el-input v-model="kv.key" size="small" placeholder="key" style="width: 180px;" />
      <el-input v-model="kv.value" size="small" placeholder="value" style="flex: 1;" />
      <el-button size="small" link @click="remove(i)">×</el-button>
    </div>
    <el-button size="small" link @click="add">+ 添加一行</el-button>
  </div>
</template>
