import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { ElMessage } from 'element-plus'

// 通过系统「另存为」对话框导出 JSON 文本，避免 WebView 默认下载目录的写入权限问题
export async function exportJsonFile(fileName: string, json: string) {
  const path = await save({
    title: '选择保存位置',
    defaultPath: fileName,
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  if (!path) return // 用户取消
  await writeTextFile(path, json)
  ElMessage.success('已导出')
}
