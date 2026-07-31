/** Runtime check for macOS (Tauri webview / browser preview). */
export function isMacOS(): boolean {
  if (typeof navigator === 'undefined') return false
  const platform = navigator.platform || ''
  if (/^Mac/i.test(platform)) return true
  const ua = navigator.userAgent || ''
  return /Mac OS X|Macintosh|Darwin/i.test(ua)
}
