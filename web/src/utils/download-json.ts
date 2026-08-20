export function safeDownloadName(name: string, fallback = 'apinest') {
  const trimmed = name.trim() || fallback
  return trimmed.replace(/[<>:"/\\|?*]/g, '_').slice(0, 80) || fallback
}

export function downloadJson(filename: string, data: unknown) {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename.endsWith('.json') ? filename : `${filename}.json`
  document.body.appendChild(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}
