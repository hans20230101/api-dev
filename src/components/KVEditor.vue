<script setup lang="ts">
import { watch } from 'vue'
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
</script>

<template>
  <div>
    <div v-for="(kv, i) in items" :key="i" class="kv-row">
      <el-checkbox v-model="kv.enable" class="kv-enable" />
      <el-input v-model="kv.key" size="small" placeholder="key" class="kv-key" />
      <el-input v-model="kv.value" size="small" placeholder="value" class="kv-value" />
      <el-button size="small" link class="kv-remove" @click="remove(i)">×</el-button>
    </div>
    <el-button size="small" link @click="add">+ 添加一行</el-button>
  </div>
</template>

<style scoped>
/* 行内布局：key/value 固定首选宽度，宽面板下保持紧凑不再被拉长；
   面板变窄时按比例收缩；删除按钮 margin-left:auto 保持贴右 */
.kv-enable {
  flex: 0 0 auto;
}
.kv-key {
  flex: 0 1 180px;
  min-width: 110px;
}
.kv-value {
  flex: 0 1 340px;
  min-width: 160px;
}
.kv-remove {
  flex: 0 0 auto;
  margin-left: auto;
}
</style>
