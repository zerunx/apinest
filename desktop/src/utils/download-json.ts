export function safeDownloadName(name: string, fallback = 'apinest') {
  const trimmed = name.trim() || fallback
  return trimmed.replace(/[<>:"/\\|?*]/g, '_').slice(0, 80) || fallback
}
