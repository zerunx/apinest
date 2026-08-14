export function readJsonStringEnd(text: string, start: number): number {
  let cursor = start + 1
  while (cursor < text.length) {
    const char = text[cursor]
    if (char === '\\') {
      cursor += 2
      continue
    }
    if (char === '"') return cursor + 1
    cursor += 1
  }
  return text.length
}

export type JsonToken = {
  kind: 'string' | 'number' | 'literal' | 'punct'
  text: string
  start: number
}

function skipWs(text: string, index: number) {
  let i = index
  while (i < text.length && /\s/.test(text[i])) i += 1
  return i
}

export function removeTrailingCommas(text: string): string {
  let result = ''
  let i = 0
  while (i < text.length) {
    if (text[i] === '"') {
      const end = readJsonStringEnd(text, i)
      result += text.slice(i, end)
      i = end
      continue
    }
    if (text[i] === ',') {
      const next = skipWs(text, i + 1)
      if (text[next] === ']' || text[next] === '}') {
        i += 1
        continue
      }
    }
    result += text[i]
    i += 1
  }
  return result
}

export function tokenizeJson(text: string): JsonToken[] {
  const tokens: JsonToken[] = []
  let i = 0

  while (i < text.length) {
    const char = text[i]
    if (/\s/.test(char)) {
      i += 1
      continue
    }

    if (char === '"') {
      const end = readJsonStringEnd(text, i)
      tokens.push({ kind: 'string', text: text.slice(i, end), start: i })
      i = end
      continue
    }

    if (/[-0-9]/.test(char)) {
      const match = text.slice(i).match(/^-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?/)
      if (match) {
        tokens.push({ kind: 'number', text: match[0], start: i })
        i += match[0].length
        continue
      }
    }

    if (text.startsWith('true', i)) {
      tokens.push({ kind: 'literal', text: 'true', start: i })
      i += 4
      continue
    }
    if (text.startsWith('false', i)) {
      tokens.push({ kind: 'literal', text: 'false', start: i })
      i += 5
      continue
    }
    if (text.startsWith('null', i)) {
      tokens.push({ kind: 'literal', text: 'null', start: i })
      i += 4
      continue
    }

    if ('{}[],:'.includes(char)) {
      tokens.push({ kind: 'punct', text: char, start: i })
      i += 1
      continue
    }

    i += 1
  }

  return tokens
}

function decodeJsonString(token: string): string {
  try {
    return JSON.parse(token) as string
  } catch {
    return token
  }
}

function isKeyToken(tokens: JsonToken[], index: number) {
  const token = tokens[index]
  const next = tokens[index + 1]
  return token?.kind === 'string' && next?.kind === 'punct' && next.text === ':'
}

export function findDuplicateKeyStarts(text: string): Set<number> {
  const tokens = tokenizeJson(text)
  const dupes = new Set<number>()
  const keySets: Set<string>[] = []

  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i]
    if (token.kind === 'punct' && token.text === '{') {
      keySets.push(new Set())
      continue
    }
    if (token.kind === 'punct' && token.text === '}') {
      keySets.pop()
      continue
    }
    if (token.kind === 'string' && keySets.length && isKeyToken(tokens, i)) {
      const keyName = decodeJsonString(token.text)
      const scope = keySets[keySets.length - 1]
      if (scope.has(keyName)) dupes.add(token.start)
      else scope.add(keyName)
    }
  }

  return dupes
}

export const JSON_INDENT_UNIT = '    '

function indentLine(level: number) {
  return `\n${JSON_INDENT_UNIT.repeat(level)}`
}

export function formatJsonLenient(text: string): string | null {
  const trimmed = text.trim()
  if (!trimmed) return null

  const normalized = removeTrailingCommas(trimmed)
  const tokens = tokenizeJson(normalized)
  if (!tokens.length) return null

  let out = ''
  let indent = 0

  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i]
    const next = tokens[i + 1]

    if (token.kind === 'punct' && token.text === '{') {
      out += '{'
      if (next?.kind === 'punct' && next.text === '}') continue
      indent += 1
      out += indentLine(indent)
      continue
    }

    if (token.kind === 'punct' && token.text === '[') {
      out += '['
      if (next?.kind === 'punct' && next.text === ']') continue
      indent += 1
      out += indentLine(indent)
      continue
    }

    if (token.kind === 'punct' && token.text === '}') {
      const prev = tokens[i - 1]
      // Keep empty objects as {} on one line.
      if (prev?.kind === 'punct' && prev.text === '{') {
        out += '}'
        continue
      }
      indent = Math.max(0, indent - 1)
      out += indentLine(indent)
      out += '}'
      continue
    }

    if (token.kind === 'punct' && token.text === ']') {
      const prev = tokens[i - 1]
      // Keep empty arrays as [] on one line (e.g. "agent_id": []).
      if (prev?.kind === 'punct' && prev.text === '[') {
        out += ']'
        continue
      }
      indent = Math.max(0, indent - 1)
      out += indentLine(indent)
      out += ']'
      continue
    }

    if (token.kind === 'punct' && token.text === ',') {
      out += ','
      out += indentLine(indent)
      continue
    }

    if (token.kind === 'punct' && token.text === ':') {
      out += ': '
      continue
    }

    out += token.text
  }

  return out.endsWith('\n') ? out : `${out}\n`
}

export function isJsonKeyAt(text: string, index: number): boolean {
  if (text[index] !== '"') return false
  const end = readJsonStringEnd(text, index)
  return text[skipWs(text, end)] === ':'
}

/** Selectable JSON token (key / value) range under caret for double-click. */
export function findJsonTokenRangeAt(
  text: string,
  index: number,
): { start: number; end: number } | null {
  if (!text) return null
  const caret = Math.max(0, Math.min(index, text.length))
  const tokens = tokenizeJson(text)

  for (const token of tokens) {
    if (token.kind === 'punct') continue
    const end = token.start + token.text.length
    if (caret < token.start || caret > end) continue

    // String keys/values: select inner content only (exclude surrounding quotes).
    if (
      token.kind === 'string' &&
      token.text.length >= 2 &&
      token.text.startsWith('"') &&
      token.text.endsWith('"')
    ) {
      return { start: token.start + 1, end: end - 1 }
    }
    return { start: token.start, end }
  }

  // Fallback: bare word (partial / invalid JSON)
  const isWord = (ch: string) => /[A-Za-z0-9_\u4e00-\u9fff$]/.test(ch)
  let i = Math.min(caret, Math.max(0, text.length - 1))
  if (i < 0) return null
  if (!isWord(text[i]) && i > 0 && isWord(text[i - 1])) i -= 1
  if (!isWord(text[i])) return null
  let start = i
  let end = i + 1
  while (start > 0 && isWord(text[start - 1])) start -= 1
  while (end < text.length && isWord(text[end])) end += 1
  return { start, end }
}

export type JsonSyntaxError = {
  index: number
  line: number
  column: number
}

export function indexToLineColumn(text: string, index: number): { line: number; column: number } {
  let line = 1
  let column = 1
  const end = Math.min(Math.max(0, index), text.length)
  for (let i = 0; i < end; i += 1) {
    if (text[i] === '\n') {
      line += 1
      column = 1
    } else {
      column += 1
    }
  }
  return { line, column }
}

/** First structural / unexpected-token index, or null if the document is valid. Trailing commas are allowed. */
function locateJsonSyntaxErrorIndex(text: string): number | null {
  let i = 0
  const atEnd = () => i >= text.length
  const skip = () => {
    i = skipWs(text, i)
  }
  const failAt = () => (atEnd() ? Math.max(0, text.length - 1) : i)

  const parseString = (): boolean => {
    if (text[i] !== '"') return false
    i += 1
    while (i < text.length) {
      const char = text[i]
      if (char === '\\') {
        i += 2
        continue
      }
      if (char === '"') {
        i += 1
        return true
      }
      i += 1
    }
    return false
  }

  const parseNumber = (): boolean => {
    const start = i
    if (text[i] === '-') i += 1
    if (i >= text.length || !/\d/.test(text[i])) {
      i = start
      return false
    }
    if (text[i] === '0') {
      i += 1
      if (/\d/.test(text[i] ?? '')) {
        i = start
        return false
      }
    } else {
      while (i < text.length && /\d/.test(text[i])) i += 1
    }
    if (text[i] === '.') {
      i += 1
      if (!/\d/.test(text[i] ?? '')) {
        i = start
        return false
      }
      while (i < text.length && /\d/.test(text[i])) i += 1
    }
    if (text[i] === 'e' || text[i] === 'E') {
      i += 1
      if (text[i] === '+' || text[i] === '-') i += 1
      if (!/\d/.test(text[i] ?? '')) {
        i = start
        return false
      }
      while (i < text.length && /\d/.test(text[i])) i += 1
    }
    return true
  }

  const parseLiteral = (word: string): boolean => {
    if (!text.startsWith(word, i)) return false
    i += word.length
    return true
  }

  const parseValue = (): boolean => {
    skip()
    if (atEnd()) return false
    const char = text[i]
    if (char === '"') return parseString()
    if (char === '{') return parseObject()
    if (char === '[') return parseArray()
    if (char === 't') return parseLiteral('true')
    if (char === 'f') return parseLiteral('false')
    if (char === 'n') return parseLiteral('null')
    if (char === '-' || (char >= '0' && char <= '9')) return parseNumber()
    return false
  }

  const parseObject = (): boolean => {
    i += 1
    skip()
    if (text[i] === '}') {
      i += 1
      return true
    }
    while (true) {
      skip()
      if (text[i] !== '"') return false
      if (!parseString()) return false
      skip()
      if (text[i] !== ':') return false
      i += 1
      if (!parseValue()) return false
      skip()
      if (text[i] === ',') {
        i += 1
        skip()
        if (text[i] === '}') {
          i += 1
          return true
        }
        continue
      }
      if (text[i] === '}') {
        i += 1
        return true
      }
      return false
    }
  }

  const parseArray = (): boolean => {
    i += 1
    skip()
    if (text[i] === ']') {
      i += 1
      return true
    }
    while (true) {
      if (!parseValue()) return false
      skip()
      if (text[i] === ',') {
        i += 1
        skip()
        if (text[i] === ']') {
          i += 1
          return true
        }
        continue
      }
      if (text[i] === ']') {
        i += 1
        return true
      }
      return false
    }
  }

  if (!parseValue()) return failAt()
  skip()
  if (!atEnd()) return i
  return null
}

/** Locate first JSON syntax error. Trailing commas are allowed. Empty input is valid. */
export function findJsonSyntaxError(text: string): JsonSyntaxError | null {
  if (!text.trim()) return null
  const index = locateJsonSyntaxErrorIndex(text)
  if (index == null) return null
  const { line, column } = indexToLineColumn(text, index)
  return { index, line, column }
}

/** True when caret is inside a JSON string literal (not on the quotes). */
export function isInsideJsonString(text: string, index: number): boolean {
  let i = 0
  while (i < text.length && i < index) {
    if (text[i] === '"') {
      const end = readJsonStringEnd(text, i)
      if (index < end) return index > i
      i = end
      continue
    }
    i += 1
  }
  return false
}

function lineIndentAt(text: string, index: number): string {
  const lineStart = text.lastIndexOf('\n', index - 1) + 1
  const line = text.slice(lineStart, index)
  const match = line.match(/^[ \t]*/)
  return match?.[0] ?? ''
}

function lastNonWs(text: string): { char: string; index: number } | null {
  for (let i = text.length - 1; i >= 0; i -= 1) {
    if (!/\s/.test(text[i])) return { char: text[i], index: i }
  }
  return null
}

function firstNonWs(text: string): { char: string; index: number } | null {
  for (let i = 0; i < text.length; i += 1) {
    if (!/\s/.test(text[i])) return { char: text[i], index: i }
  }
  return null
}

/**
 * Smart Enter for JSON editing:
 * - `{|}` / `[|]` (optional spaces) → expand with an indented blank line
 * - after `{` / `[` → next line indented one level deeper
 * - otherwise → keep current line indent
 */
export function applyJsonEnter(
  text: string,
  selectionStart: number,
  selectionEnd: number = selectionStart,
): { text: string; cursor: number } | null {
  if (isInsideJsonString(text, selectionStart)) return null

  const before = text.slice(0, selectionStart)
  const after = text.slice(selectionEnd)
  const baseIndent = lineIndentAt(text, selectionStart)
  const innerIndent = baseIndent + JSON_INDENT_UNIT

  const left = lastNonWs(before)
  const right = firstNonWs(after)
  if (
    left &&
    right &&
    ((left.char === '{' && right.char === '}') || (left.char === '[' && right.char === ']'))
  ) {
    const between = before.slice(left.index + 1) + after.slice(0, right.index)
    // Same-line empty pair (only spaces/tabs between) → expand to multiline block.
    if (/^[ \t]*$/.test(between)) {
      const openIndent = lineIndentAt(text, left.index + 1)
      const childIndent = openIndent + JSON_INDENT_UNIT
      const head = before.slice(0, left.index + 1)
      const tail = after.slice(right.index)
      const next = `${head}\n${childIndent}\n${openIndent}${tail}`
      return { text: next, cursor: head.length + 1 + childIndent.length }
    }
  }

  const lineBeforeCursor = before.slice(before.lastIndexOf('\n') + 1)
  const deeper = /[{\[]\s*$/.test(lineBeforeCursor)
  const indent = deeper ? innerIndent : baseIndent
  const next = `${before}\n${indent}${after}`
  return { text: next, cursor: before.length + 1 + indent.length }
}
