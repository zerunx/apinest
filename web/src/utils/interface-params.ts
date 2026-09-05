export interface ParamRow {
  name: string
  type: string
  required: boolean
  enabled?: boolean
  description: string
  example: string
}

export function emptyParamRow(): ParamRow {
  return {
    name: '',
    type: 'string',
    required: true,
    enabled: true,
    description: '',
    example: '',
  }
}

export function hasParamContent(row: ParamRow): boolean {
  return !!(row.name.trim() || row.description.trim() || row.example.trim())
}

export function ensureTrailingEmptyRow(rows: ParamRow[]): ParamRow[] {
  if (!rows.length) return [emptyParamRow()]
  const last = rows[rows.length - 1]
  if (!hasParamContent(last)) return rows
  return [...rows, emptyParamRow()]
}

export function compactParamRows(rows: ParamRow[]): ParamRow[] {
  return rows.filter(hasParamContent).map((row) => ({
    ...row,
    enabled: row.enabled !== false,
  }))
}

export function setAllParamsRequired(rows: ParamRow[], required: boolean): ParamRow[] {
  return rows.map((row) => ({
    ...row,
    required,
  }))
}
