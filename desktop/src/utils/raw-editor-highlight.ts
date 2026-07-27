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

function highlightJson(text: string, errorIndex: number | null = null): string {
  const duplicateKeys = findDuplicateKeyStarts(text)
  const parts: string[] = []
  let index = 0

  while (index < text.length) {
    const char = text[index]

    if (/\s/.test(char)) {
      let next = index + 1
      while (next < text.length && /\s/.test(text[next])) next += 1
      if (errorIndex != null && errorIndex >= index && errorIndex < next) {
        const before = text.slice(index, errorIndex)
        const bad = text.slice(errorIndex, errorIndex + 1) || ' '
        const after = text.slice(errorIndex + 1, next)
        parts.push(renderText(before, index))
        parts.push(wrap('raw-hl-error', bad === '\n' ? ' ' : bad, errorIndex))
        parts.push(renderText(after, errorIndex + 1))
      } else {
        parts.push(renderText(text.slice(index, next), index))
      }
      index = next
      continue
    }

    if (char === '"') {
      const end = readJsonStringEnd(text, index)
      const token = text.slice(index, end)
      if (isJsonKeyAt(text, index)) {
        const base = duplicateKeys.has(index) ? 'raw-hl-dup-key' : 'raw-hl-key'
        parts.push(wrap(tokenClass(base, index, end, errorIndex), token, index))
      } else {
        parts.push(wrap(tokenClass('raw-hl-string', index, end, errorIndex), token, index))
      }
      index = end
      continue
    }

    if (/[-0-9]/.test(char)) {
      const match = text.slice(index).match(/^-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?/)
      if (match) {
        const end = index + match[0].length
        parts.push(wrap(tokenClass('raw-hl-number', index, end, errorIndex), match[0], index))
        index = end
        continue
      }
    }

    if (text.startsWith('true', index)) {
      parts.push(wrap(tokenClass('raw-hl-bool', index, index + 4, errorIndex), 'true', index))
      index += 4
      continue
    }
    if (text.startsWith('false', index)) {
      parts.push(wrap(tokenClass('raw-hl-bool', index, index + 5, errorIndex), 'false', index))
      index += 5
      continue
    }
    if (text.startsWith('null', index)) {
      parts.push(wrap(tokenClass('raw-hl-null', index, index + 4, errorIndex), 'null', index))
      index += 4
      continue
    }

    if ('{}[],:'.includes(char)) {
      parts.push(wrap(tokenClass('raw-hl-punct', index, index + 1, errorIndex), char, index))
      index += 1
      continue
    }

    if (errorIndex != null && errorIndex === index) {
      parts.push(wrap('raw-hl-error', char, index))
    } else {
      parts.push(escapeHtml(char))
    }
    index += 1
  }

  if (errorIndex != null && errorIndex >= text.length) {
    parts.push(wrap('raw-hl-error raw-hl-error--eof', ' ', errorIndex))
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

export function resolveRawHighlightType(contentType: string, format?: string): string {
  if (['JSON', 'Text', 'XML', 'HTML'].includes(contentType)) return contentType
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
    default:
      highlighted = escapeHtml(text)
  }

  return highlighted.endsWith('\n') ? highlighted : `${highlighted}\n`
}
