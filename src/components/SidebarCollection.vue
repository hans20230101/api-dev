<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { ElMessage, ElMessageBox, type TreeInstance } from 'element-plus'
import { useCollectionStore } from '@/stores/collection'
import { useTabsStore } from '@/stores/tabs'
import { methodTagType } from '@/utils'
import { ExportAll, ExportCollection, WriteExportFile } from '@/api/wails'
import { save } from '@tauri-apps/plugin-dialog'
import { Upload, CopyDocument } from '@element-plus/icons-vue'

const emit = defineEmits<{ (e: 'import'): void }>()

interface TreeNode {
  key: string
  name: string
  kind: 'collection' | 'request' | 'orphan'
  id: number
  parentId: number | null
  method?: string
  /** 顶层集合 id：集合节点为其所属顶层集合；请求节点为其父集合的顶层集合；未归类为 null */
  topId: number | null
  children?: TreeNode[]
}

const store = useCollectionStore()
const tabsStore = useTabsStore()
const treeRef = ref<TreeInstance>()
// 用户主动折叠的集合 id 集合。默认所有集合都是展开的（首次加载 / 新建 / 导入均展开），
// 只有用户手动折叠过的集合保持折叠。
const collapsedIds = ref(new Set<number>())
function onNodeExpand(data: TreeNode) {
  if (data.kind === 'collection') {
    collapsedIds.value.delete(data.id)
  }
}

function onNodeCollapse(data: TreeNode) {
  if (data.kind === 'collection') {
    collapsedIds.value.add(data.id)
  }
}

// 应展开的节点 key：除被用户折叠的集合外全部展开；未归类请求有子项时也展开。
const expandedKeys = computed(() => {
  const keys: string[] = []
  if (store.requests.some(r => r.collection_id == null)) keys.push('orphan')
  store.collections.forEach(c => {
    if (!collapsedIds.value.has(c.id)) keys.push(`c-${c.id}`)
  })
  return keys
})

watch(() => store.collections.map(c => c.id), (ids: number[]) => {
  // 清理已删除集合的折叠记录，避免编号复用后误折叠
  const current = new Set(ids)
  const stale = Array.from(collapsedIds.value).filter(id => !current.has(id))
  if (stale.length) {
    const next = new Set(collapsedIds.value)
    stale.forEach(id => next.delete(id))
    collapsedIds.value = next
  }
}, { immediate: true })


const treeData = computed<TreeNode[]>(() => {
  const map = new Map<number, TreeNode>()
  const roots: TreeNode[] = []
  // 先计算每个集合的顶层祖先 id，用于限制拖放范围为同一顶层集合内
  const topOf = new Map<number, number>()
  const resolveTop = (c: { id: number; parent_id?: number | null }, seen = new Set<number>()): number => {
    if (topOf.has(c.id)) return topOf.get(c.id)!
    if (c.parent_id == null || seen.has(c.id)) { topOf.set(c.id, c.id); return c.id }
    const parent = store.collections.find(p => p.id === c.parent_id)
    const top = parent ? resolveTop(parent, new Set([...seen, c.id])) : c.id
    topOf.set(c.id, top)
    return top
  }
  store.collections.forEach(c => map.set(c.id, { key: `c-${c.id}`, name: c.name, kind: 'collection', id: c.id, parentId: c.parent_id ?? null, topId: resolveTop(c), children: [] }))
  const orphanNode: TreeNode = { key: 'orphan', name: '未归类请求', kind: 'orphan', id: 0, parentId: null, topId: null, children: [] }
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
      topId: parent ? parent.topId : null,
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
    const defaultName = `${node.name} (副本)`
    const { value } = await ElMessageBox.prompt('请输入副本名称（留空使用默认名称）', '复制请求', {
      inputValue: defaultName,
      confirmButtonText: '确定',
      cancelButtonText: '取消',
    })
    // 输入为空时由 store 回落到默认名称
    await store.duplicateRequest(node.id, value)
    ElMessage.success('已复制')
  } catch (e) {
    if (e === 'cancel' || e === 'close') return
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
  else if (cmd === 'import-collection') emit('import')
}

function isActive(node: TreeNode) {
  if (node.kind === 'request') return tabsStore.activeTabId === node.id
  if (node.kind === 'collection') return store.selectedCollectionId === node.id
  return false
}

// ===== 拖放：请求节点可拖动，仅允许放到同一顶层集合下的其它集合（或未归类）中 =====
const dragOverKey = ref<string | null>(null)
// 拖动中的请求所属顶层集合 id；null 表示未归类请求，可放入任意集合
let dragTopId: number | null | undefined

function isDropTarget(node: TreeNode) {
  return node.kind === 'collection' || node.kind === 'orphan'
}

// 目标集合是否与拖动请求同属一个顶层集合
function canDropInto(node: TreeNode): boolean {
  if (node.kind === 'orphan') return true
  if (dragTopId === undefined) return false
  return dragTopId === null ? true : node.topId === dragTopId
}

function onDragStart(e: DragEvent, node: TreeNode) {
  if (node.kind !== 'request') return
  e.dataTransfer!.setData('application/x-request-id', String(node.id))
  e.dataTransfer!.effectAllowed = 'move'
  dragTopId = node.topId
}

function onDragOver(e: DragEvent, node: TreeNode) {
  if (!isDropTarget(node) || !canDropInto(node)) return
  e.preventDefault()
  e.dataTransfer!.dropEffect = 'move'
  dragOverKey.value = node.key
}

function onDragLeave(node: TreeNode) {
  if (dragOverKey.value === node.key) dragOverKey.value = null
}

async function onDrop(e: DragEvent, node: TreeNode) {
  dragOverKey.value = null
  if (!isDropTarget(node) || !canDropInto(node)) return
  dragTopId = undefined
  e.preventDefault()
  const id = Number(e.dataTransfer!.getData('application/x-request-id'))
  if (!id) return
  const targetCollectionId = node.kind === 'collection' ? node.id : null
  try {
    await store.moveRequest(id, targetCollectionId)
    ElMessage.success(`已移动到「${node.name}」`)
  } catch (err) {
    console.error('移动请求失败:', err)
    ElMessage.error(`移动失败: ${err}`)
  }
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
            <el-dropdown-item command="import-collection" divided>
              <el-icon><Upload /></el-icon>导入集合
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
    <div style="flex: 1; overflow: auto;">
      <el-tree
        v-if="store.collections.length"
        ref="treeRef"
        :data="treeData"
        node-key="key"
        :props="{ label: 'name', children: 'children' }"
        :default-expanded-keys="expandedKeys"
        :auto-expand-parent="false"
        :expand-on-click-node="false"
        highlight-current
        @node-click="onNodeClick"
        @node-expand="onNodeExpand"
        @node-collapse="onNodeCollapse"
        style="padding: 6px;"
      >
        <template #default="{ data }">
          <span
            class="node"
            :class="{ active: isActive(data), 'drop-target': dragOverKey === data.key }"
            :draggable="data.kind === 'request'"
            @dragstart="(e: DragEvent) => onDragStart(e, data)"
            @dragover="(e: DragEvent) => onDragOver(e, data)"
            @dragleave="onDragLeave(data)"
            @drop="(e: DragEvent) => onDrop(e, data)"
          >
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
/* 树节点行高：默认 26px 偏挤，统一到 30px 让请求名称呼吸感更好 */
.sidebar :deep(.el-tree-node__content) { height: 30px; }

.node { display: flex; align-items: center; gap: 6px; width: 100%; overflow: hidden; padding-right: 4px; }
.node-icon { color: #e6a23c; flex-shrink: 0; font-size: 14px; }

/* 请求行：method 标签定宽等宽字体，名称列对齐；标签字号压缩 */
.node :deep(.el-tag) {
  flex-shrink: 0;
  width: 52px;
  justify-content: center;
  font-size: 10px;
  font-family: Consolas, Monaco, monospace;
  padding: 0;
}

/* 请求名称：13px 常规灰度、hover 加深、选中主色加粗 */
.node-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  color: var(--text);
  line-height: 1.4;
}
.node:hover .node-name { color: var(--el-color-primary); }

.ops-trigger { visibility: hidden; }
.node:hover .ops-trigger { visibility: visible; }
.node.active .node-name { color: #409eff; font-weight: 600; }

/* 拖放：请求节点可拖动，集合/未归类节点悬停时高亮为放置目标 */
.node[draggable='true'] { cursor: grab; }
.node.drop-target {
  outline: 1px dashed var(--el-color-primary);
  outline-offset: -1px;
  border-radius: 4px;
  background: var(--el-color-primary-light-9);
}

.empty { color: #909399; text-align: center; padding: 20px 0; font-size: 12px; }
</style>
