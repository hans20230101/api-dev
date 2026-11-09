<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useCollectionStore } from '@/stores/collection'
import { methodTagType } from '@/utils'
import { ElMessage, ElMessageBox } from 'element-plus'

interface TreeNode { id: number; name: string; children: TreeNode[] }

const store = useCollectionStore()

const treeData = computed<TreeNode[]>(() => {
  const map = new Map<number, TreeNode>()
  store.collections.forEach(c => map.set(c.id, { id: c.id, name: c.name, children: [] }))
  const roots: TreeNode[] = []
  store.collections.forEach(c => {
    const node = map.get(c.id)!
    if (c.parent_id != null && map.has(c.parent_id)) {
      map.get(c.parent_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  })
  return roots
})

onMounted(() => store.loadCollections())

async function onNodeClick(node: TreeNode) {
  await store.selectCollection(node.id)
}

async function addCollection() {
  const { value } = await ElMessageBox.prompt('请输入集合名称', '新建集合', {
    confirmButtonText: '确定', cancelButtonText: '取消',
  })
  if (value) {
    await store.addCollection(value, null)
    ElMessage.success('已创建')
  }
}

async function addRequest() {
  if (store.selectedCollectionId == null) {
    ElMessage.warning('请先选择一个集合')
    return
  }
  store.newRequest()
}

async function deleteCollection(id: number) {
  await ElMessageBox.confirm('确认删除该集合？下挂请求会被移出。', '提示', { type: 'warning' })
  await store.removeCollection(id)
}
</script>

<template>
  <div class="sidebar">
    <div class="panel-title" style="display: flex; justify-content: space-between; align-items: center;">
      <span>集合</span>
      <el-button size="small" link @click="addCollection">+ 新建</el-button>
    </div>
    <div style="overflow: auto; max-height: 45%;">
      <el-tree
        :data="treeData"
        node-key="id"
        :props="{ label: 'name' }"
        highlight-current
        @node-click="onNodeClick"
        style="padding: 6px;"
      >
        <template #default="{ data }">
          <span style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
            <span>{{ data.name }}</span>
            <el-button size="small" link @click.stop="deleteCollection(data.id)">×</el-button>
          </span>
        </template>
      </el-tree>
      <div v-if="!store.collections.length" class="empty">暂无集合</div>
    </div>

    <div class="panel-title" style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border);">
      <span>请求{{ store.selectedCollectionId == null ? '' : '' }}</span>
      <el-button size="small" link @click="addRequest">+ 新建</el-button>
    </div>
    <div style="flex: 1; overflow: auto;">
      <div
        v-for="r in store.requests"
        :key="r.id"
        class="req-item"
        :class="{ active: store.currentRequest?.id === r.id }"
        @click="store.selectRequest(r.id)"
      >
        <el-tag size="small" :type="methodTagType(r.method)" effect="plain">{{ r.method }}</el-tag>
        <span class="req-name">{{ r.name }}</span>
        <el-button size="small" link @click.stop="store.removeRequest(r.id)">×</el-button>
      </div>
      <div v-if="!store.requests.length" class="empty">暂无请求</div>
    </div>
  </div>
</template>

<style scoped>
.req-item { display: flex; align-items: center; gap: 6px; padding: 6px 10px; cursor: pointer; }
.req-item:hover { background: #f5f7fa; }
.req-item.active { background: #ecf5ff; }
.req-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.empty { color: #909399; text-align: center; padding: 20px 0; font-size: 12px; }
</style>
