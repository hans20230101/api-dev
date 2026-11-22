<script setup lang="ts">
import { reactive, watch } from 'vue'
import { Check, Edit } from '@element-plus/icons-vue'
import { models } from '@/api/wails'

const props = defineProps<{ items: models.KV[]; placeholder?: boolean }>()

// 新建请求（placeholder）时空列表默认显示一行空白行，方便直接开始填写；
// 该行 key 为空不会参与发送。修改已有请求时不显示，空列表只有「+ 添加一行」
const unlocked = reactive(new Set<models.KV>())
watch(
  () => [props.items, props.placeholder] as const,
  ([items, ph]) => {
    if (items.length === 0 && ph) items.push({ key: '', value: '', enable: true })
    // 清理已不存在的行对象，防止整个列表被外部替换后残留引用
    for (const kv of [...unlocked]) if (!items.includes(kv)) unlocked.delete(kv)
  },
  { immediate: true },
)

// 行级编辑锁：已有数据（key 非空）的行默认只读（保存状态），
// 点击行内「修改」图标解锁编辑，再点「完成」恢复只读
function hasData(kv: models.KV): boolean {
  return kv.key.trim() !== ''
}
function isLocked(kv: models.KV): boolean {
  return hasData(kv) && !unlocked.has(kv)
}
function toggleEdit(kv: models.KV) {
  if (unlocked.has(kv)) unlocked.delete(kv)
  else unlocked.add(kv)
}

function add() {
  props.items.push({ key: '', value: '', enable: true })
}
function remove(i: number) {
  unlocked.delete(props.items[i])
  props.items.splice(i, 1)
}
</script>

<template>
  <div>
    <div v-for="(kv, i) in items" :key="i" class="kv-row">
      <el-checkbox v-model="kv.enable" class="kv-enable" :disabled="isLocked(kv)" />
      <el-input v-model="kv.key" size="small" placeholder="key" class="kv-key" :disabled="isLocked(kv)" />
      <el-input v-model="kv.value" size="small" placeholder="value" class="kv-value" :disabled="isLocked(kv)" />
      <el-button
        size="small"
        link
        class="kv-edit"
        :title="isLocked(kv) ? '修改' : '完成'"
        @click="toggleEdit(kv)"
      >
        <el-icon><Check v-if="!isLocked(kv)" /><Edit v-else /></el-icon>
      </el-button>
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
.kv-edit {
  flex: 0 0 auto;
  margin-left: auto;
}
.kv-remove {
  flex: 0 0 auto;
}
</style>
