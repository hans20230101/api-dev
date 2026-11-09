<script setup lang="ts">
import { computed } from 'vue'
import { useCollectionStore } from '@/stores/collection'
import { useRuntimeStore } from '@/stores/runtime'
import { ElMessage } from 'element-plus'
import KVEditor from './KVEditor.vue'
import TestRunner from './TestRunner.vue'

const colStore = useCollectionStore()
const runtime = useRuntimeStore()

const req = computed(() => colStore.currentRequest)

const methods = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD']
const bodyTypes = [
  { label: 'none', value: 'none' },
  { label: 'json', value: 'json' },
  { label: 'form', value: 'form' },
  { label: 'raw', value: 'raw' },
]

async function send() {
  if (!req.value) return
  if (!req.value.url) {
    ElMessage.warning('请填写 URL')
    return
  }
  await runtime.send(req.value)
}

async function save() {
  if (!req.value) return
  await colStore.saveCurrent()
  ElMessage.success('已保存')
}
</script>

<template>
  <div class="request-area">
    <div v-if="!req" class="empty">请选择或新建一个请求</div>
    <div v-else style="padding: 10px;">
      <div style="display: flex; gap: 8px; margin-bottom: 8px; align-items: center;">
        <el-input v-model="req.name" size="small" placeholder="请求名称" style="width: 220px;" />
        <span style="flex: 1;"></span>
        <TestRunner />
      </div>
      <div style="display: flex; gap: 8px; margin-bottom: 8px;">
        <el-select v-model="req.method" size="small" style="width: 110px;">
          <el-option v-for="m in methods" :key="m" :label="m" :value="m" />
        </el-select>
        <el-input v-model="req.url" size="small" placeholder="请求 URL，支持 {{var}} 变量替换" style="flex: 1;" />
        <el-button type="primary" size="small" :loading="runtime.sending" @click="send">发送</el-button>
        <el-button size="small" @click="save">保存</el-button>
      </div>
      <el-tabs>
        <el-tab-pane label="Headers">
          <KVEditor :items="req.headers" />
        </el-tab-pane>
        <el-tab-pane label="Params">
          <KVEditor :items="req.params" />
        </el-tab-pane>
        <el-tab-pane label="Body">
          <el-radio-group v-model="req.body_type" size="small" style="margin-bottom: 8px;">
            <el-radio-button v-for="b in bodyTypes" :key="b.value" :value="b.value">{{ b.label }}</el-radio-button>
          </el-radio-group>
          <template v-if="req.body_type === 'json' || req.body_type === 'raw'">
            <el-input
              v-model="req.body"
              type="textarea"
              :rows="8"
              placeholder="请求体内容，支持 {{var}}"
              class="mono"
            />
          </template>
          <template v-else-if="req.body_type === 'form'">
            <div style="color: var(--muted); margin-bottom: 6px;">
              form 模式：在下方填写 JSON 数组，例如 <code>[{"key":"a","value":"b","enable":true}]</code>
            </div>
            <el-input v-model="req.body" type="textarea" :rows="6" class="mono" />
          </template>
          <div v-else style="color: var(--muted);">该请求没有请求体</div>
        </el-tab-pane>
        <el-tab-pane label="Tests">
          <div style="color: var(--muted); margin-bottom: 6px;">
            每行一条断言，支持：assertStatus(200)、assertHeader("Content-Type","json")、
            assertBodyContains("ok")、assertJsonPath("data.id", 1)
          </div>
          <el-input
            v-model="req.tests_script"
            type="textarea"
            :rows="8"
            placeholder="assertStatus(200)"
            class="mono"
          />
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<style scoped>
.empty { color: #909399; text-align: center; padding: 40px 0; }
</style>
