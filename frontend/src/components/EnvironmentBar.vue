<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useEnvironmentStore } from '@/stores/environment'
import { ElMessage, ElMessageBox } from 'element-plus'
import { models } from '@/api/wails'

const store = useEnvironmentStore()
const mgrVisible = ref(false)
const newVarKey = ref('')
const newVarValue = ref('')

onMounted(() => store.load())

async function addEnv() {
  const { value } = await ElMessageBox.prompt('请输入环境名称', '新建环境', {
    confirmButtonText: '确定', cancelButtonText: '取消',
  })
  if (value) {
    await store.add(value)
    ElMessage.success('已创建')
  }
}

async function renameEnv(id: number, oldName: string) {
  const { value } = await ElMessageBox.prompt('请输入新名称', '重命名环境', {
    inputValue: oldName, confirmButtonText: '确定', cancelButtonText: '取消',
  })
  if (value) await store.rename(id, value)
}

async function addVar() {
  if (!store.activeEnvId) {
    ElMessage.warning('请先选择环境')
    return
  }
  if (!newVarKey.value) return
  await store.saveVariable({
    environment_id: store.activeEnvId,
    key: newVarKey.value,
    value: newVarValue.value,
    type: 'string',
  } as models.Variable)
  newVarKey.value = ''
  newVarValue.value = ''
}

defineEmits<{ (e: 'export'): void; (e: 'import'): void }>()
</script>

<template>
  <div class="topbar">
    <img src="/app-icon.png" alt="API 教研接口测试工具" class="brand-icon" />
    <span style="font-weight: 600; font-size: 14px;">API 教研接口测试工具</span>
    <span style="flex: 1;"></span>
    <span style="color: var(--muted);">环境：</span>
    <el-select
      :model-value="store.activeEnvId"
      placeholder="无"
      size="small"
      style="width: 150px;"
      @change="(v: any) => store.setActive(v)"
    >
      <el-option v-for="e in store.environments" :key="e.id" :label="e.name" :value="e.id" />
    </el-select>
    <el-button size="small" @click="mgrVisible = true">环境管理</el-button>
    <el-button size="small" @click="$emit('import')">导入</el-button>
    <el-button size="small" @click="$emit('export')">导出</el-button>

    <el-dialog v-model="mgrVisible" title="环境与变量管理" width="680px">
      <div style="display: flex; gap: 16px;">
        <div style="width: 220px; border-right: 1px solid var(--border); padding-right: 12px;">
          <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
            <strong>环境</strong>
            <el-button size="small" link @click="addEnv">+ 新建</el-button>
          </div>
          <div v-for="e in store.environments" :key="e.id" class="env-item" :class="{ active: store.activeEnvId === e.id }">
            <span @click="store.setActive(e.id)">{{ e.name }}</span>
            <span class="ops">
              <el-button size="small" link @click="renameEnv(e.id, e.name)">改</el-button>
              <el-button size="small" link @click="store.remove(e.id)">删</el-button>
            </span>
          </div>
          <div v-if="!store.environments.length" class="empty">暂无环境</div>
        </div>
        <div style="flex: 1;">
          <div v-if="!store.activeEnvId" class="empty">请选择一个环境</div>
          <template v-else>
            <div style="display: flex; gap: 6px; margin-bottom: 8px;">
              <el-input v-model="newVarKey" size="small" placeholder="变量名" style="width: 140px;" />
              <el-input v-model="newVarValue" size="small" placeholder="变量值" style="width: 200px;" />
              <el-button size="small" type="primary" @click="addVar">添加</el-button>
            </div>
            <div v-for="v in store.variables" :key="v.id" class="kv-row">
              <el-input v-model="v.key" size="small" style="width: 140px;" />
              <el-input v-model="v.value" size="small" style="width: 220px;" />
              <el-button size="small" link @click="store.saveVariable(v)">存</el-button>
              <el-button size="small" link @click="store.removeVariable(v.id)">删</el-button>
            </div>
            <div v-if="!store.variables.length" class="empty">暂无变量，使用 <code v-pre>{{key}}</code> 在 URL/头/体中引用</div>
          </template>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<style scoped>
.brand-icon { width: 24px; height: 24px; border-radius: 6px; }
.env-item { display: flex; justify-content: space-between; align-items: center; padding: 6px 4px; cursor: pointer; }
.env-item:hover { background: #f5f7fa; }
.env-item.active { background: #ecf5ff; }
.env-item .ops { visibility: hidden; }
.env-item:hover .ops { visibility: visible; }
.empty { color: #909399; text-align: center; padding: 20px 0; font-size: 12px; }
</style>
