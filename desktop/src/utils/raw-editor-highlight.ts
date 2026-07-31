import {
  findDuplicateKeyStarts,
  findJsonSyntaxError,
  isJsonKeyAt,
  readJsonStringEnd,
  type JsonSyntaxError,
} from './raw-json-utils'

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

/** Mark spaces/tabs (not newlines) so selection can show · dots. */
function renderText(text: string, baseIndex: number): string {
  let out = ''
  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i]
    const abs = baseIndex + i
    if (ch === ' ' || ch === '\t') {
      out += `<span class="raw-hl-ws" data-i="${abs}">${ch === ' ' ? ' ' : '\t'}</span>`
    } else {
      out += escapeHtml(ch)
    }
  }
  return out
}

function wrap(className: string, value: string, baseIndex: number): string {
  return `<span class="${className}">${renderText(value, baseIndex)}</span>`
}

function tokenClass(base: string, start: number, end: number, errorIndex: number | null): string {
  if (errorIndex != null && errorIndex >= start && errorIndex < end) {
    return `${base} raw-hl-error`
  }
  return base
}

function highlightJson(text: string, errorIndex: number | null = null, indexOffset = 0): string {
  const duplicateKeys = findDuplicateKeyStarts(text)
  const parts: string[] = []
  let index = 0

  while (index < text.length) {
    const char = text[index]
    const abs = index + indexOffset

    if (/\s/.test(char)) {
      let next = index + 1
      while (next < text.length && /\s/.test(text[next])) next += 1
      if (errorIndex != null && errorIndex >= index && errorIndex < next) {
        const before = text.slice(index, errorIndex)
        const bad = text.slice(errorIndex, errorIndex + 1) || ' '
        const after = text.slice(errorIndex + 1, next)
        parts.push(renderText(before, abs))
        parts.push(wrap('raw-hl-error', bad === '\n' ? ' ' : bad, errorIndex + indexOffset))
        parts.push(renderText(after, errorIndex + 1 + indexOffset))
      } else {
        parts.push(renderText(text.slice(index, next), abs))
      }
      index = next
      continue
    }

    if (char === '"') {
      const end = readJsonStringEnd(text, index)
      const token = text.slice(index, end)
      if (isJsonKeyAt(text, index)) {
        const base = duplicateKeys.has(index) ? 'raw-hl-dup-key' : 'raw-hl-key'
        parts.push(wrap(tokenClass(base, index, end, errorIndex), token, abs))
      } else {
        parts.push(wrap(tokenClass('raw-hl-string', index, end, errorIndex), token, abs))
      }
      index = end
      continue
    }

    if (/[-0-9]/.test(char)) {
      const match = text.slice(index).match(/^-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?/)
      if (match) {
        const end = index + match[0].length
        parts.push(wrap(tokenClass('raw-hl-number', index, end, errorIndex), match[0], abs))
        index = end
        continue
      }
    }

    if (text.startsWith('true', index)) {
      parts.push(wrap(tokenClass('raw-hl-bool', index, index + 4, errorIndex), 'true', abs))
      index += 4
      continue
    }
    if (text.startsWith('false', index)) {
      parts.push(wrap(tokenClass('raw-hl-bool', index, index + 5, errorIndex), 'false', abs))
      index += 5
      continue
    }
    if (text.startsWith('null', index)) {
      parts.push(wrap(tokenClass('raw-hl-null', index, index + 4, errorIndex), 'null', abs))
      index += 4
      continue
    }

    if ('{}[],:'.includes(char)) {
      parts.push(wrap(tokenClass('raw-hl-punct', index, index + 1, errorIndex), char, abs))
      index += 1
      continue
    }

    if (errorIndex != null && errorIndex === index) {
      parts.push(wrap('raw-hl-error', char, abs))
    } else {
      parts.push(escapeHtml(char))
    }
    index += 1
  }

  if (errorIndex != null && errorIndex >= text.length) {
    parts.push(wrap('raw-hl-error raw-hl-error--eof', ' ', text.length + indexOffset))
  }

  return parts.join('')
}

function highlightMarkup(text: string): string {
  const escaped = escapeHtml(text)
  return escaped
    .replace(/(&lt;!--[\s\S]*?--&gt;)/g, '<span class="raw-hl-comment">$1</span>')
    .replace(/(&lt;\/?[\w:-]+)/g, '<span class="raw-hl-tag">$1</span>')
    .replace(/([\w:-]+)(=)/g, '<span class="raw-hl-attr">$1</span><span class="raw-hl-punct">=</span>')
    .replace(/(&quot;(?:\\.|[^&])*?&quot;)/g, '<span class="raw-hl-string">$1</span>')
    .replace(/(&gt;|&lt;)/g, '<span class="raw-hl-punct">$1</span>')
}

/** Syntax highlight for generated curl commands. */
function highlightCurl(text: string): string {
  const parts: string[] = []
  let index = 0

  while (index < text.length) {
    const char = text[index]

    if (/\s/.test(char)) {
      let next = index + 1
      while (next < text.length && /\s/.test(text[next])) next += 1
      parts.push(renderText(text.slice(index, next), index))
      index = next
      continue
    }

    if (char === '\\' && (index + 1 >= text.length || text[index + 1] === '\n' || text[index + 1] === '\r')) {
      parts.push(wrap('raw-hl-punct', '\\', index))
      index += 1
      continue
    }

    if (char === "'" || char === '"') {
      const quote = char
      let end = index + 1
      while (end < text.length) {
        if (quote === '"' && text[end] === '\\' && end + 1 < text.length) {
          end += 2
          continue
        }
        if (text[end] === quote) {
          end += 1
          break
        }
        end += 1
      }
      const closed = end > index + 1 && text[end - 1] === quote
      const inner = closed ? text.slice(index + 1, end - 1) : text.slice(index + 1, end)
      const trimmed = inner.trim()
      const looksJson =
        closed &&
        ((trimmed.startsWith('{') && trimmed.endsWith('}')) ||
          (trimmed.startsWith('[') && trimmed.endsWith(']')))
      if (looksJson) {
        parts.push(wrap('raw-hl-string', quote, index))
        parts.push(highlightJson(inner, null, index + 1))
        parts.push(wrap('raw-hl-string', quote, end - 1))
      } else {
        parts.push(wrap('raw-hl-string', text.slice(index, end), index))
      }
      index = end
      continue
    }

    if (char === '-') {
      let end = index + 1
      while (end < text.length && /[A-Za-z0-9_=-]/.test(text[end])) end += 1
      parts.push(wrap('raw-hl-attr', text.slice(index, end), index))
      index = end
      continue
    }

    if (/[A-Za-z_]/.test(char)) {
      let end = index + 1
      while (end < text.length && /[A-Za-z0-9_.+-]/.test(text[end])) end += 1
      const word = text.slice(index, end)
      const cls = word.toLowerCase() === 'curl' || word.toLowerCase() === 'curl.exe' ? 'raw-hl-bool' : 'raw-hl-plain'
      parts.push(wrap(cls, word, index))
      index = end
      continue
    }

    parts.push(wrap('raw-hl-punct', char, index))
    index += 1
  }

  return parts.join('')
}

export function resolveRawHighlightType(contentType: string, format?: string): string {
  if (['JSON', 'Text', 'XML', 'HTML', 'CURL'].includes(contentType)) return contentType
  if (format === 'raw') return 'JSON'
  return 'Text'
}

export function diagnoseRawJson(text: string): JsonSyntaxError | null {
  return findJsonSyntaxError(text)
}

export function highlightRawContent(
  text: string,
  contentType: string,
  format?: string,
  errorIndex: number | null = null,
): string {
  if (!text) return '\n'

  const type = resolveRawHighlightType(contentType, format)
  let highlighted: string

  switch (type) {
    case 'JSON':
      highlighted = highlightJson(text, errorIndex)
      break
    case 'XML':
    case 'HTML':
      highlighted = highlightMarkup(text)
      break
    case 'CURL':
      highlighted = highlightCurl(text)
      break
    default:
      highlighted = renderText(text, 0)
  }

  return highlighted.endsWith('\n') ? highlighted : `${highlighted}\n`
}
