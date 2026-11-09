<script setup lang="ts">
import { useTabsStore } from '@/stores/tabs'

const tabs = useTabsStore()

function onTabChange(name: number | string) {
  tabs.activeTabId = Number(name)
}

function onTabRemove(name: number | string) {
  tabs.closeTab(Number(name))
}
</script>

<template>
  <div v-if="tabs.tabs.length" class="tabs-bar">
    <el-tabs
      :model-value="tabs.activeTabId"
      type="card"
      closable
      @tab-change="onTabChange"
      @tab-remove="onTabRemove"
    >
      <el-tab-pane v-for="t in tabs.tabs" :key="t.id" :name="t.id">
        <template #label>
          <span class="tab-label">
            <span v-if="tabs.isDirty(t.id)" class="dirty-dot" title="有未保存的修改"></span>
            <span class="tab-name">{{ t.name }}</span>
          </span>
        </template>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<style scoped>
.tabs-bar { background: #fff; border-bottom: 1px solid var(--border); }
/* 选项卡内容区由外部的请求编辑器/响应面板承担，隐藏组件自带的内容区 */
.tabs-bar :deep(.el-tabs__content) { display: none; }
.tabs-bar :deep(.el-tabs__header) { margin: 0; }
.tab-label { display: inline-flex; align-items: center; gap: 4px; max-width: 180px; }
.tab-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dirty-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #e6a23c;
  flex-shrink: 0;
}
</style>
