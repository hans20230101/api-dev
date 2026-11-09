<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useCollectionStore } from '@/stores/collection'
import { useTabsStore } from '@/stores/tabs'
import { methodTagType } from '@/utils'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ExportAll, ExportCollection, WriteExportFile } from '@/api/wails'
import { save } from '@tauri-apps/plugin-dialog'
import { Upload, CopyDocument } from '@element-plus/icons-vue'

interface TreeNode {
  key: string
  name: string
  kind: 'collection' | 'request' | 'orphan'
  id: number
  parentId: number | null
  method?: string
  children?: TreeNode[]
}

const store = useCollectionStore()
const tabsStore = useTabsStore()

const treeData = computed<TreeNode[]>(() => {
  const map = new Map<number, TreeNode>()
  store.collections.forEach(c =>
    map.set(c.id, { key: `c-${c.id}`, name: c.name, kind: 'collection', id: c.id, parentId: c.parent_id ?? null, children: [] }),
  )
  const roots: TreeNode[] = []
  const orphanNode: TreeNode = { key: 'orphan', name: '未归类请求', kind: 'orphan', id: 0, parentId: null, children: [] }
  store.collections.forEach(c => {
    const node = map.get(c.id)!
    if (c.parent_id != null && map.has(c.parent_id)) {
      map.get(c.parent_id)!.children!.push(node)
    } else {
      roots.push(node)
    }
  })
  store.requests.forEach(r => {
    const parent = r.collection_id == null ? undefined : map.get(r.collection_id)
    const node: TreeNode = {
      key: `r-${r.id}`,
      name: r.name,
      kind: 'request',
      id: r.id,
      parentId: r.collection_id ?? null,
      method: r.method,
    }
    if (parent) {
      parent.children!.push(node)
    } else {
      orphanNode.children!.push(node)
    }
  })
  if (orphanNode.children!.length) roots.push(orphanNode)
  return roots
})

onMounted(() => store.loadAll())

function onNodeClick(node: TreeNode) {
  if (node.kind === 'collection') {
    store.selectCollection(node.id)
  } else if (node.kind === 'request') {
    store.selectRequest(node.id)
  }
}

async function addCollection(parentId: number | null) {
  const { value } = await ElMessageBox.prompt(
    parentId == null ? '请输入集合名称' : '请输入子集合名称',
    parentId == null ? '新建集合' : '新建子集合',
    { confirmButtonText: '确定', cancelButtonText: '取消' },
  )
  if (value) {
    await store.addCollection(value, parentId)
    ElMessage.success('已创建')
  }
}

async function addRequest(collectionId: number | null) {
  if (collectionId == null) {
    ElMessage.warning('请先选择一个集合')
    return
  }
  try {
    await store.addRequest(collectionId)
    ElMessage.success('已创建')
  } catch (e) {
    console.error('创建请求失败:', e)
    ElMessage.error(`创建失败: ${e}`)
  }
}

async function duplicateRequest(node: TreeNode) {
  try {
    await store.duplicateRequest(node.id)
    ElMessage.success('已复制')
  } catch (e) {
    console.error('复制请求失败:', e)
    ElMessage.error(`复制失败: ${e}`)
  }
}

async function renameCollection(node: TreeNode) {
  const { value } = await ElMessageBox.prompt('请输入新名称', '重命名集合', {
    inputValue: node.name, confirmButtonText: '确定', cancelButtonText: '取消',
  })
  if (value) {
    await store.renameCollection(node.id, value)
    ElMessage.success('已重命名')
  }
}

async function renameRequest(node: TreeNode) {
  const { value } = await ElMessageBox.prompt('请输入新名称', '重命名请求', {
    inputValue: node.name, confirmButtonText: '确定', cancelButtonText: '取消',
  })
  if (value) {
    await store.renameRequest(node.id, value)
    ElMessage.success('已重命名')
  }
}

async function deleteNode(node: TreeNode) {
  if (node.kind === 'collection') {
    await ElMessageBox.confirm('确认删除该集合？其下所有子集合和请求都会被一并删除。', '提示', { type: 'warning' })
    await store.removeCollection(node.id)
  } else if (node.kind === 'request') {
    await ElMessageBox.confirm('确认删除该请求？', '提示', { type: 'warning' })
    await store.removeRequest(node.id)
  }
}

async function exportNode(node: TreeNode) {
  try {
    const json = await ExportCollection(node.id)
    const filePath = await save({
      title: '导出集合',
      defaultPath: `${node.name}-export.json`,
      filters: [{ name: 'JSON 文件', extensions: ['json'] }],
    })
    if (!filePath) return
    await WriteExportFile(filePath, json)
    ElMessage.success('已导出')
  } catch (e) {
    ElMessage.error(`导出失败: ${e}`)
  }
}

function onCommand(cmd: string, node: TreeNode) {
  switch (cmd) {
    case 'add-collection': addCollection(node.id); break
    case 'add-request': addRequest(node.id); break
    case 'export': exportNode(node); break
    case 'duplicate': duplicateRequest(node); break
    case 'rename': node.kind === 'request' ? renameRequest(node) : renameCollection(node); break
    case 'delete': deleteNode(node); break
  }
}

async function exportSelected() {
  const id = store.selectedCollectionId
  const node = id == null ? undefined : treeData.value.find(n => n.kind === 'collection' && n.id === id)
  if (!node) {
    ElMessage.warning('请先选择一个集合')
    return
  }
  if (node.parentId != null) {
    ElMessage.warning('只有顶层集合才支持导出')
    return
  }
  await exportNode(node)
}

async function exportAll() {
  try {
    const json = await ExportAll()
    const filePath = await save({
      title: '导出全部',
      defaultPath: 'api-dev-export.json',
      filters: [{ name: 'JSON 文件', extensions: ['json'] }],
    })
    if (!filePath) return
    await WriteExportFile(filePath, json)
    ElMessage.success('已导出')
  } catch (e) {
    ElMessage.error(`导出失败: ${e}`)
  }
}

function onHeaderCommand(cmd: string) {
  if (cmd === 'add-collection') addCollection(null)
  else if (cmd === 'add-request') addRequest(store.selectedCollectionId)
  else if (cmd === 'export-all') exportAll()
  else if (cmd === 'export-selected') exportSelected()
}

function isActive(node: TreeNode) {
  if (node.kind === 'request') return tabsStore.activeTabId === node.id
  if (node.kind === 'collection') return store.selectedCollectionId === node.id
  return false
}
</script>

<template>
  <div class="sidebar">
    <div class="panel-title" style="display: flex; justify-content: space-between; align-items: center;">
      <span>集合/请求</span>
      <el-dropdown trigger="click" @command="onHeaderCommand">
        <el-button size="small" link>
          <el-icon><Plus /></el-icon>&nbsp;新建
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="add-collection">
              <el-icon><FolderAdd /></el-icon>新建集合
            </el-dropdown-item>
            <el-dropdown-item command="add-request">
              <el-icon><DocumentAdd /></el-icon>新建请求
            </el-dropdown-item>
            <el-dropdown-item command="export-selected" divided>
              <el-icon><Upload /></el-icon>导入集合
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
    <div style="flex: 1; overflow: auto;">
      <el-tree
        v-if="store.collections.length"
        :data="treeData"
        node-key="key"
        :props="{ label: 'name', children: 'children' }"
        :default-expand-all="true"
        :expand-on-click-node="false"
        highlight-current
        @node-click="onNodeClick"
        style="padding: 6px;"
      >
        <template #default="{ data }">
          <span class="node" :class="{ active: isActive(data) }">
            <el-icon v-if="data.kind === 'orphan'" class="node-icon"><Files /></el-icon>
            <el-icon v-else-if="data.kind === 'collection'" class="node-icon"><Folder /></el-icon>
            <el-tag v-else size="small" :type="methodTagType(data.method || 'GET')" effect="plain">
              {{ data.method || 'GET' }}
            </el-tag>
            <span class="node-name">{{ data.name }}</span>
            <el-dropdown
              v-if="data.kind !== 'orphan'"
              trigger="click"
              @command="(cmd: string) => onCommand(cmd, data)"
            >
              <el-button size="small" link class="ops-trigger" @click.stop>
                <el-icon><MoreFilled /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <template v-if="data.kind === 'collection'">
                    <el-dropdown-item command="add-collection">
                      <el-icon><FolderAdd /></el-icon>新建子集合
                    </el-dropdown-item>
                    <el-dropdown-item command="add-request">
                      <el-icon><DocumentAdd /></el-icon>新建请求
                    </el-dropdown-item>
                    <el-dropdown-item v-if="data.parentId == null" command="export">
                      <el-icon><Download /></el-icon>导出集合
                    </el-dropdown-item>
                  </template>
                  <el-dropdown-item v-if="data.kind === 'request'" command="duplicate">
                    <el-icon><CopyDocument /></el-icon>复制
                  </el-dropdown-item>
                  <el-dropdown-item command="rename">
                    <el-icon><EditPen /></el-icon>重命名
                  </el-dropdown-item>
                  <el-dropdown-item command="delete" divided>
                    <el-icon><Delete /></el-icon>删除
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </span>
        </template>
      </el-tree>
      <div v-if="!store.collections.length" class="empty">暂无集合</div>
    </div>
  </div>
</template>

<style scoped>
.node { display: flex; align-items: center; gap: 6px; width: 100%; overflow: hidden; }
.node-icon { color: #e6a23c; }
.node-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ops-trigger { visibility: hidden; }
.node:hover .ops-trigger { visibility: visible; }
.node.active .node-name { color: #409eff; font-weight: 600; }
.empty { color: #909399; text-align: center; padding: 20px 0; font-size: 12px; }
</style>
