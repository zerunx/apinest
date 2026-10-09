export const DEFAULT_USER_AGENT = 'ApiNest/0.3.4'

export type AutoHeader = {
  key: string
  value: string
  /** Include in the outgoing request. Host / Content-Length are display-only. */
  send: boolean
}

export type AutoHeaderContext = {
  url: string
  bodyFormat: 'none' | 'form-data' | 'raw'
  rawBody: string
  authType?: 'none' | 'bearer' | 'basic'
  authBearerToken?: string
  authBasicUser?: string
  authBasicPassword?: string
  userHeaders: Array<{ key: string; enabled?: boolean }>
  /** Lowercase header names the user has unchecked in the hidden-headers UI. */
  disabledAutoHeaders?: Iterable<string>
}

function disabledAutoHeaderKeys(ctx: AutoHeaderContext): Set<string> {
  const keys = new Set<string>()
  for (const key of ctx.disabledAutoHeaders ?? []) {
    const normalized = key.trim().toLowerCase()
    if (normalized) keys.add(normalized)
  }
  return keys
}

function takenKeys(headers: Array<{ key: string; enabled?: boolean }>): Set<string> {
  const keys = new Set<string>()
  for (const h of headers) {
    if (h.enabled === false) continue
    const key = h.key.trim().toLowerCase()
    if (key) keys.add(key)
  }
  return keys
}

export function hostFromUrl(url: string): string {
  const raw = url.trim()
  if (!raw) return ''
  try {
    const withScheme = /^[a-z][a-z0-9+.-]*:\/\//i.test(raw) ? raw : `http://${raw}`
    return new URL(withScheme).host
  } catch {
    return ''
  }
}

function byteLength(text: string): string {
  return String(new TextEncoder().encode(text).length)
}

function basicAuthValue(user: string, password: string): string | null {
  try {
    return `Basic ${btoa(`${user}:${password}`)}`
  } catch {
    return null
  }
}

export function buildAutoHeaders(ctx: AutoHeaderContext): AutoHeader[] {
  const taken = takenKeys(ctx.userHeaders)
  const headers: AutoHeader[] = []

  const add = (key: string, value: string, send: boolean) => {
    if (!value) return
    if (taken.has(key.toLowerCase())) return
    headers.push({ key, value, send })
  }

  add('User-Agent', DEFAULT_USER_AGENT, true)
  add('Accept', '*/*', true)
  add('Accept-Encoding', 'gzip, deflate', false)
  add('Connection', 'keep-alive', true)
  add('Host', hostFromUrl(ctx.url), false)

  if (ctx.bodyFormat === 'raw') {
    add('Content-Type', 'application/json', true)
    add('Content-Length', byteLength(ctx.rawBody ?? ''), false)
  }

  if (ctx.authType === 'bearer' && ctx.authBearerToken?.trim()) {
    add('Authorization', `Bearer ${ctx.authBearerToken.trim()}`, true)
  } else if (ctx.authType === 'basic' && (ctx.authBasicUser || ctx.authBasicPassword)) {
    const value = basicAuthValue(ctx.authBasicUser ?? '', ctx.authBasicPassword ?? '')
    if (value) add('Authorization', value, true)
  }

  return headers
}

export function sendableAutoHeaders(ctx: AutoHeaderContext): Array<{ key: string; value: string }> {
  const disabled = disabledAutoHeaderKeys(ctx)
  return buildAutoHeaders(ctx)
    .filter((h) => h.send && !disabled.has(h.key.toLowerCase()))
    .map((h) => ({ key: h.key, value: h.value }))
}

export function isAutoHeaderEnabled(ctx: AutoHeaderContext, key: string): boolean {
  return !disabledAutoHeaderKeys(ctx).has(key.trim().toLowerCase())
}
