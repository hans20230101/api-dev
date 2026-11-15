<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useRequestSettingsStore } from '@/stores/requestSettings'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

const settings = useRequestSettingsStore()
const timeoutSecs = ref(settings.timeoutSecs)

watch(
  () => props.modelValue,
  open => {
    if (open) {
      settings.load()
      timeoutSecs.value = settings.timeoutSecs
    }
  },
)

async function onSave() {
  await settings.save(timeoutSecs.value)
  ElMessage.success(`已保存：请求超时 ${settings.timeoutSecs} 秒`)
  emit('update:modelValue', false)
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    title="请求设置"
    width="420px"
    :close-on-click-modal="false"
    @update:model-value="(v: boolean) => emit('update:modelValue', v)"
  >
    <el-form label-width="110px" @submit.prevent>
      <el-form-item label="请求超时时间">
        <el-input-number
          v-model="timeoutSecs"
          :min="1"
          :max="3600"
          :step="5"
          controls-position="right"
          style="width: 160px"
        />
        <span class="unit">秒（1–3600）</span>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="emit('update:modelValue', false)">取消</el-button>
      <el-button type="primary" @click="onSave">保存</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.unit {
  margin-left: 8px;
  font-size: 12px;
  color: #909399;
}
</style>
