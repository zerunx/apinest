export type FormDataImportRow = {
  name: string
  type: 'string'
  example: string
  description: string
  required: boolean
  enabled: boolean
}

export type FormDataImportMode = 'replace' | 'append' | 'merge'

function formatFormDataValue(value: unknown): string {
  if (value === null || value === undefined) return ''
  if (typeof value === 'string') return value
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  return JSON.stringify(value)
}

/** Parse a JSON object into flat form-data text rows. */
export function parseJsonToFormDataRows(jsonText: string): FormDataImportRow[] {
  const text = jsonText.trim()
  if (!text) {
    throw new Error('empty')
  }

  let parsed: unknown
  try {
    parsed = JSON.parse(text)
  } catch {
    throw new Error('invalid')
  }

  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    throw new Error('root-object')
  }

  return Object.entries(parsed as Record<string, unknown>).map(([name, value]) => ({
    name,
    type: 'string' as const,
    example: formatFormDataValue(value),
    description: '',
    required: false,
    enabled: true,
  }))
}

export function applyFormDataJsonImport(
  existing: Array<{ name: string; example?: string; description?: string; enabled?: boolean }>,
  jsonText: string,
  mode: FormDataImportMode,
): FormDataImportRow[] {
  const imported = parseJsonToFormDataRows(jsonText)
  const current = existing
    .filter((row) => row.name.trim())
    .map((row) => ({
      name: row.name,
      type: 'string' as const,
      example: row.example ?? '',
      description: row.description ?? '',
      required: false,
      enabled: row.enabled !== false,
    }))

  if (mode === 'replace') {
    return imported
  }
  if (mode === 'append') {
    return [...current, ...imported]
  }

  const result = current.map((row) => ({ ...row }))
  for (const imp of imported) {
    const index = result.findIndex((row) => row.name === imp.name)
    if (index === -1) {
      result.push({ ...imp })
      continue
    }
    result[index] = {
      ...result[index],
      example: imp.example,
      enabled: true,
    }
  }
  return result
}
