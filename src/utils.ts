// HTTP 方法对应的标签颜色
export function methodTagType(m: string) {
  switch (m) {
    case 'GET': return 'success'
    case 'POST': return 'warning'
    case 'PUT': return 'primary'
    case 'DELETE': return 'danger'
    case 'PATCH': return 'info'
    default: return 'info'
  }
}

// 状态码对应的标签颜色
export function statusTagType(code: number) {
  if (code >= 200 && code < 300) return 'success'
  if (code >= 300 && code < 400) return 'info'
  if (code >= 400 && code < 500) return 'warning'
  if (code >= 500) return 'danger'
  return 'info'
}
