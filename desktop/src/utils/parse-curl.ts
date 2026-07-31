export type ParsedCurlHeader = {
  key: string
  value: string
}

export type ParsedCurlFormField = {
  name: string
  value: string
  type: 'string' | 'file'
}

export type ParsedCurl = {
  method: string
  url: string
  headers: ParsedCurlHeader[]
  body: string | null
  formData: ParsedCurlFormField[] | null
}

const BOOL_FLAGS = new Set([
  '-L',
  '--location',
  '-i',
  '--include',
  '-I',
  '--head',
  '-s',
  '--silent',
  '-S',
  '--show-error',
  '-v',
  '--verbose',
  '-k',
  '--insecure',
  '-f',
  '--fail',
  '-g',
  '--globoff',
  '-G',
  '--get',
  '-N',
  '--no-buffer',
  '-n',
  '--netrc',
  '-#',
  '--progress-bar',
  '-4',
  '-6',
  '--compressed',
  '--http1.0',
  '--http1.1',
  '--http2',
  '--http2-prior-knowledge',
  '--raw',
  '--remote-name',
  '--remote-header-name',
  '--fail-with-body',
  '--location-trusted',
  '--no-keepalive',
  '--path-as-is',
])

const ARG_FLAGS = new Set([
  '-X',
  '--request',
  '-H',
  '--header',
  '-d',
  '--data',
  '--data-raw',
  '--data-binary',
  '--data-urlencode',
  '--json',
  '-F',
  '--form',
  '--form-string',
  '-u',
  '--user',
  '-A',
  '--user-agent',
  '-b',
  '--cookie',
  '-e',
  '--referer',
  '--url',
  '-o',
  '--output',
  '-O',
  '-w',
  '--write-out',
  '-m',
  '--max-time',
  '--connect-timeout',
  '-x',
  '--proxy',
  '--cacert',
  '--capath',
  '--cert',
  '--key',
  '-E',
  '--passwd',
  '--proxy-header',
  '-T',
  '--upload-file',
])

/** True when text looks like a curl command (e.g. paste into URL bar). */
export function looksLikeCurl(text: string): boolean {
  const trimmed = text.trim()
  if (!trimmed) return false
  if (/^curl(?:\.exe)?(?:\s|$)/i.test(trimmed)) return true
  // Multiline paste where first line may be blank / comment
  return /^[\s#]*curl(?:\.exe)?\b/im.test(trimmed)
}

function unescapeDoubleQuoted(value: string): string {
  let out = ''
  for (let i = 0; i < value.length; i += 1) {
    const ch = value[i]
    if (ch !== '\\' || i + 1 >= value.length) {
      out += ch
      continue
    }
    const next = value[i + 1]
    if (next === 'n') {
      out += '\n'
      i += 1
    } else if (next === 'r') {
      out += '\r'
      i += 1
    } else if (next === 't') {
      out += '\t'
      i += 1
    } else if (next === '"' || next === '\\' || next === '$' || next === '`') {
      out += next
      i += 1
    } else {
      out += next
      i += 1
    }
  }
  return out
}

/** Join bash/cmd line continuations, then tokenize with quote awareness. */
export function tokenizeCurl(input: string): string[] {
  const joined = input
    .replace(/\r\n/g, '\n')
    .replace(/\\\n/g, ' ')
    .replace(/\^\n/g, ' ')
    .trim()

  const tokens: string[] = []
  let i = 0
  while (i < joined.length) {
    while (i < joined.length && /\s/.test(joined[i])) i += 1
    if (i >= joined.length) break

    const ch = joined[i]
    if (ch === "'" ) {
      i += 1
      let value = ''
      while (i < joined.length && joined[i] !== "'") {
        value += joined[i]
        i += 1
      }
      if (i < joined.length) i += 1
      tokens.push(value)
      continue
    }

    if (ch === '"') {
      i += 1
      let raw = ''
      while (i < joined.length) {
        if (joined[i] === '\\' && i + 1 < joined.length) {
          raw += joined[i] + joined[i + 1]
          i += 2
          continue
        }
        if (joined[i] === '"') break
        raw += joined[i]
        i += 1
      }
      if (i < joined.length) i += 1
      tokens.push(unescapeDoubleQuoted(raw))
      continue
    }

    if (ch === '$' && joined[i + 1] === "'") {
      i += 2
      let value = ''
      while (i < joined.length && joined[i] !== "'") {
        if (joined[i] === '\\' && i + 1 < joined.length) {
          const next = joined[i + 1]
          if (next === 'n') value += '\n'
          else if (next === 'r') value += '\r'
          else if (next === 't') value += '\t'
          else value += next
          i += 2
          continue
        }
        value += joined[i]
        i += 1
      }
      if (i < joined.length) i += 1
      tokens.push(value)
      continue
    }

    let value = ''
    while (i < joined.length && !/\s/.test(joined[i])) {
      value += joined[i]
      i += 1
    }
    tokens.push(value)
  }
  return tokens
}

function splitFlag(token: string): { flag: string; inline?: string } {
  if (token.startsWith('--')) {
    const eq = token.indexOf('=')
    if (eq > 2) return { flag: token.slice(0, eq), inline: token.slice(eq + 1) }
    return { flag: token }
  }
  // -XPOST / -HContent-Type:… (short flag glued to value)
  if (/^-[A-Za-z]$/.test(token)) return { flag: token }
  if (/^-[A-Za-z].+/.test(token) && !token.startsWith('--')) {
    return { flag: token.slice(0, 2), inline: token.slice(2) }
  }
  return { flag: token }
}

function parseHeaderLine(raw: string): ParsedCurlHeader | null {
  const idx = raw.indexOf(':')
  if (idx <= 0) return null
  const key = raw.slice(0, idx).trim()
  const value = raw.slice(idx + 1).trim()
  if (!key) return null
  return { key, value }
}

function parseFormField(raw: string): ParsedCurlFormField {
  const idx = raw.indexOf('=')
  if (idx < 0) return { name: raw, value: '', type: 'string' }
  const name = raw.slice(0, idx)
  let value = raw.slice(idx + 1)
  let type: 'string' | 'file' = 'string'
  if (value.startsWith('@')) {
    type = 'file'
    value = value.slice(1)
  }
  // Strip optional type suffix like `;type=image/png`
  const semi = value.indexOf(';')
  if (semi >= 0) value = value.slice(0, semi)
  return { name, value, type }
}

function takeValue(
  tokens: string[],
  index: number,
  inline?: string,
): { value: string; next: number } {
  if (inline !== undefined) return { value: inline, next: index }
  return { value: tokens[index] ?? '', next: index + 1 }
}

/**
 * Parse a curl command into method / url / headers / body.
 * Returns null if the text is not a recognizable curl invocation.
 */
export function parseCurl(text: string): ParsedCurl | null {
  if (!looksLikeCurl(text)) return null

  const tokens = tokenizeCurl(text)
  if (!tokens.length) return null

  let start = 0
  while (start < tokens.length && !/^curl(?:\.exe)?$/i.test(tokens[start])) start += 1
  if (start >= tokens.length) return null
  start += 1

  let method: string | null = null
  let url = ''
  const headers: ParsedCurlHeader[] = []
  const dataParts: string[] = []
  const formData: ParsedCurlFormField[] = []
  let forceGet = false
  let hasJsonFlag = false
  let user: string | null = null

  let i = start
  while (i < tokens.length) {
    const token = tokens[i]
    i += 1

    if (!token.startsWith('-')) {
      if (!url) url = token
      continue
    }

    const { flag, inline } = splitFlag(token)

    if (flag === '-G' || flag === '--get') {
      forceGet = true
      continue
    }

    if (BOOL_FLAGS.has(flag)) continue

    if (flag === '-X' || flag === '--request') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      if (value) method = value.toUpperCase()
      continue
    }

    if (flag === '--url') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      if (value) url = value
      continue
    }

    if (flag === '-H' || flag === '--header' || flag === '--proxy-header') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      const header = parseHeaderLine(value)
      if (header) headers.push(header)
      continue
    }

    if (
      flag === '-d' ||
      flag === '--data' ||
      flag === '--data-raw' ||
      flag === '--data-binary' ||
      flag === '--data-urlencode'
    ) {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      dataParts.push(value)
      continue
    }

    if (flag === '--json') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      dataParts.push(value)
      hasJsonFlag = true
      continue
    }

    if (flag === '-F' || flag === '--form' || flag === '--form-string') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      formData.push(parseFormField(value))
      continue
    }

    if (flag === '-u' || flag === '--user') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      user = value
      continue
    }

    if (flag === '-A' || flag === '--user-agent') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      headers.push({ key: 'User-Agent', value })
      continue
    }

    if (flag === '-b' || flag === '--cookie') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      if (!value.startsWith('@')) headers.push({ key: 'Cookie', value })
      continue
    }

    if (flag === '-e' || flag === '--referer') {
      const { value, next } = takeValue(tokens, i, inline)
      i = next
      headers.push({ key: 'Referer', value })
      continue
    }

    if (ARG_FLAGS.has(flag)) {
      // Known but unused here — consume the argument so it is not treated as URL.
      const { next } = takeValue(tokens, i, inline)
      i = next
      continue
    }

    // Unknown long option with = already handled; unknown --flag may take a value.
    if (flag.startsWith('--') && inline === undefined) {
      const nextTok = tokens[i]
      if (nextTok && !nextTok.startsWith('-')) i += 1
      continue
    }
  }

  if (hasJsonFlag && !headers.some((h) => h.key.toLowerCase() === 'content-type')) {
    headers.push({ key: 'Content-Type', value: 'application/json' })
  }

  if (user && !headers.some((h) => h.key.toLowerCase() === 'authorization')) {
    try {
      headers.push({
        key: 'Authorization',
        value: `Basic ${btoa(user)}`,
      })
    } catch {
      headers.push({ key: 'Authorization', value: `Basic ${user}` })
    }
  }

  const body =
    formData.length > 0 ? null : dataParts.length > 0 ? dataParts.join('&') : null

  if (!method) {
    if (forceGet) method = 'GET'
    else if (body != null || formData.length > 0) method = 'POST'
    else method = 'GET'
  }

  return {
    method,
    url,
    headers,
    body,
    formData: formData.length > 0 ? formData : null,
  }
}
