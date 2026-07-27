function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function wrap(className: string, value: string): string {
  return `<span class="${className}">${escapeHtml(value)}</span>`
}

function isJsonKey(text: string, index: number): boolean {
  let cursor = index
  while (cursor < text.length && /\s/.test(text[cursor])) cursor += 1
  return text[cursor] === ':'
}

function readJsonString(text: string, start: number): number {
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

function highlightJson(text: string): string {
  const parts: string[] = []
  let index = 0

  while (index < text.length) {
    const char = text[index]

    if (/\s/.test(char)) {
      let next = index + 1
      while (next < text.length && /\s/.test(text[next])) next += 1
      parts.push(escapeHtml(text.slice(index, next)))
      index = next
      continue
    }

    if (char === '"') {
      const end = readJsonString(text, index)
      const token = text.slice(index, end)
      parts.push(isJsonKey(text, end) ? wrap('raw-hl-key', token) : wrap('raw-hl-string', token))
      index = end
      continue
    }

    if (/[-0-9]/.test(char)) {
      const match = text.slice(index).match(/^-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?/)
      if (match) {
        parts.push(wrap('raw-hl-number', match[0]))
        index += match[0].length
        continue
      }
    }

    if (text.startsWith('true', index)) {
      parts.push(wrap('raw-hl-bool', 'true'))
      index += 4
      continue
    }
    if (text.startsWith('false', index)) {
      parts.push(wrap('raw-hl-bool', 'false'))
      index += 5
      continue
    }
    if (text.startsWith('null', index)) {
      parts.push(wrap('raw-hl-null', 'null'))
      index += 4
      continue
    }

    if ('{}[],:'.includes(char)) {
      parts.push(wrap('raw-hl-punct', char))
      index += 1
      continue
    }

    parts.push(escapeHtml(char))
    index += 1
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

export function highlightRawContent(text: string, contentType: string, format?: string): string {
  if (!text) return '\n'

  const type = resolveRawHighlightType(contentType, format)
  let highlighted: string

  switch (type) {
    case 'JSON':
      highlighted = highlightJson(text)
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
