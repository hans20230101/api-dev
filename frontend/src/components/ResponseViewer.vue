<script setup lang="ts">
import { useRuntimeStore } from '@/stores/runtime'
import { statusTagType } from '@/utils'

const runtime = useRuntimeStore()
</script>

<template>
  <div class="response-area">
    <div class="panel-title">响应</div>
    <div v-if="runtime.sending" class="empty">请求发送中...</div>
    <div v-else-if="!runtime.response" class="empty">点击「发送」查看响应</div>
    <div v-else style="padding: 10px;">
      <div v-if="runtime.response.error" style="color: #f56c6c;">
        请求失败：{{ runtime.response.error }}
      </div>
      <template v-else>
        <div style="display: flex; gap: 16px; margin-bottom: 8px; align-items: center;">
          <el-tag :type="statusTagType(runtime.response.status_code)">状态 {{ runtime.response.status_code }}</el-tag>
          <span>耗时 {{ runtime.response.time_ms }} ms</span>
          <span>响应大小 {{ runtime.response.resp_size }} B</span>
          <span>请求大小 {{ runtime.response.req_size }} B</span>
        </div>
        <el-tabs>
          <el-tab-pane label="响应体">
            <pre class="mono" style="background: #fafafa; padding: 10px; border-radius: 4px; max-height: 320px; overflow: auto; margin: 0;">{{ runtime.response.body }}</pre>
          </el-tab-pane>
          <el-tab-pane label="响应头">
            <div v-for="(v, k) in runtime.response.headers" :key="k" class="kv-row mono">
              <strong style="min-width: 180px;">{{ k }}:</strong>
              <span>{{ v }}</span>
            </div>
          </el-tab-pane>
        </el-tabs>
      </template>
    </div>
  </div>
</template>

<style scoped>
.empty { color: #909399; text-align: center; padding: 30px 0; }
</style>
