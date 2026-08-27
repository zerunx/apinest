export type CurlHeader = {
  key: string
  value: string
}

export type CurlFormField = {
  name: string
  value: string
  type?: 'string' | 'file'
}

export type BuildCurlInput = {
  method: string
  url: string
  headers?: CurlHeader[]
  body?: string | null
  formData?: CurlFormField[] | null
}

/** Replace `{{VAR}}` / `{{ VAR }}` with environment values; unknown keys kept as-is. */
export function applyEnvVars(input: string, vars: Record<string, string> | null | undefined): string {
  if (!input || !vars || !Object.keys(vars).length) return input
  return input.replace(/\{\{\s*([^{}]+?)\s*\}\}/g, (match, key: string) => {
    const k = key.trim()
    return Object.prototype.hasOwnProperty.call(vars, k) ? vars[k] : match
  })
}

/** Bash-safe single-quoted string. */
export function shellSingleQuote(value: string): string {
  return `'${value.replace(/'/g, `'\\''`)}'`
}

function escapeHeaderValue(value: string): string {
  return value.replace(/\r?\n/g, ' ')
}

/**
 * Build a Postman-style curl command from the current request.
 */
export function buildCurl(input: BuildCurlInput): string {
  const method = (input.method || 'GET').toUpperCase()
  const url = (input.url || '').trim() || '/'
  const headers = (input.headers ?? []).filter((h) => h.key.trim())
  const formData = (input.formData ?? []).filter((f) => f.name.trim())
  const body = input.body ?? null

  const lines: string[] = []
  const hasBody = body != null && body !== ''
  const hasForm = formData.length > 0

  if (method === 'GET' && !hasBody && !hasForm) {
    lines.push(`curl --location ${shellSingleQuote(url)}`)
  } else if (method === 'POST' && (hasBody || hasForm)) {
    lines.push(`curl --location ${shellSingleQuote(url)} \\`)
  } else {
    lines.push(`curl --location --request ${method} ${shellSingleQuote(url)} \\`)
  }

  const continuation = method === 'GET' && !hasBody && !hasForm ? false : true

  // If we started without trailing \, and will add more, fix first line.
  if (!continuation && (headers.length || hasBody || hasForm)) {
    lines[0] = `${lines[0]} \\`
  }

  for (const h of headers) {
    const line = `--header ${shellSingleQuote(`${h.key.trim()}: ${escapeHeaderValue(h.value)}`)}`
    lines.push(line)
  }

  if (hasForm) {
    for (const field of formData) {
      const value =
        field.type === 'file' ? `@${field.value || 'file'}` : field.value
      lines.push(`--form ${shellSingleQuote(`${field.name}=${value}`)}`)
    }
  } else if (hasBody) {
    lines.push(`--data ${shellSingleQuote(body!)}`)
  }

  // Join with ` \` newlines except last line
  if (lines.length === 1) return lines[0]

  return lines
    .map((line, idx) => {
      const bare = line.replace(/\s\\$/, '')
      return idx < lines.length - 1 ? `${bare} \\` : bare
    })
    .join('\n')
}

type TabDetail = {
  request_headers?: Array<{ name?: string; example?: string; enabled?: boolean }>
  request_body?: {
    format?: string
    raw?: string
    fields?: Array<{
      name?: string
      example?: string
      type?: string
      enabled?: boolean
    }>
  }
  authorization?: {
    type?: string
    bearer_token?: string
    basic_username?: string
    basic_password?: string
  }
}

export type CurlTabState = {
  method?: string
  url?: string
  detail?: TabDetail | null
}

/** Build curl from InterfaceEditor.exportTabState()-shaped data. */
export function buildCurlFromTabState(
  state: CurlTabState | null | undefined,
  variables?: Record<string, string> | null,
): string {
  if (!state) return ''

  const vars = variables ?? {}
  const detail = state.detail ?? {}
  const headers: CurlHeader[] = []

  for (const h of detail.request_headers ?? []) {
    if (h.enabled === false) continue
    const key = applyEnvVars(String(h.name ?? '').trim(), vars)
    if (!key) continue
    headers.push({ key, value: applyEnvVars(String(h.example ?? ''), vars) })
  }

  const auth = detail.authorization
  const hasAuthHeader = headers.some((h) => h.key.toLowerCase() === 'authorization')
  if (!hasAuthHeader && auth) {
    if (auth.type === 'bearer' && auth.bearer_token?.trim()) {
      headers.push({
        key: 'Authorization',
        value: `Bearer ${applyEnvVars(auth.bearer_token.trim(), vars)}`,
      })
    } else if (auth.type === 'basic' && (auth.basic_username || auth.basic_password)) {
      try {
        const user = applyEnvVars(auth.basic_username ?? '', vars)
        const pass = applyEnvVars(auth.basic_password ?? '', vars)
        headers.push({
          key: 'Authorization',
          value: `Basic ${btoa(`${user}:${pass}`)}`,
        })
      } catch {
        // ignore invalid latin1 for btoa
      }
    }
  }

  const body = detail.request_body
  const format = body?.format ?? 'none'
  if (
    format === 'raw' &&
    !headers.some((h) => h.key.toLowerCase() === 'content-type')
  ) {
    headers.push({ key: 'Content-Type', value: 'application/json' })
  }
  let raw: string | null = null
  let formData: CurlFormField[] | null = null

  if (format === 'raw' && body?.raw != null && String(body.raw) !== '') {
    raw = applyEnvVars(String(body.raw), vars)
  } else if (format === 'form-data' && Array.isArray(body?.fields)) {
    formData = body!.fields!
      .filter((f) => f.enabled !== false && String(f.name ?? '').trim())
      .map((f) => ({
        name: applyEnvVars(String(f.name ?? '').trim(), vars),
        value: applyEnvVars(String(f.example ?? ''), vars),
        type: f.type === 'file' ? 'file' : 'string',
      }))
  }

  return buildCurl({
    method: state.method || 'GET',
    url: applyEnvVars(state.url || '', vars),
    headers,
    body: raw,
    formData,
  })
}
