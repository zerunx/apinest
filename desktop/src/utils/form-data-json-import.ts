export type FormDataImportRow = {
  name: string
  type: string
  example: string
  description: string
  required: boolean
  enabled: boolean
  fileData?: string
  fileName?: string
}

export type FormDataImportMode = 'replace' | 'append' | 'merge'

type ExistingFormDataRow = {
  name: string
  type?: string
  example?: string
  description?: string
  enabled?: boolean
  fileData?: string
  fileName?: string
}

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
    type: 'string',
    example: formatFormDataValue(value),
    description: '',
    required: false,
    enabled: true,
  }))
}

function toImportRow(row: ExistingFormDataRow): FormDataImportRow {
  const type = row.type?.trim() || 'string'
  return {
    name: row.name,
    type,
    example: row.example ?? '',
    description: row.description ?? '',
    required: false,
    enabled: row.enabled !== false,
    fileData: type === 'file' ? row.fileData : undefined,
    fileName: type === 'file' ? row.fileName : undefined,
  }
}

export function applyFormDataJsonImport(
  existing: ExistingFormDataRow[],
  jsonText: string,
  mode: FormDataImportMode,
): FormDataImportRow[] {
  const imported = parseJsonToFormDataRows(jsonText)
  const current = existing.filter((row) => row.name.trim()).map(toImportRow)

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
    const prev = result[index]
    // Keep file fields intact when JSON only carries text values.
    if (prev.type === 'file') {
      result[index] = { ...prev, enabled: true }
      continue
    }
    result[index] = {
      ...prev,
      example: imp.example,
      enabled: true,
    }
  }
  return result
}
