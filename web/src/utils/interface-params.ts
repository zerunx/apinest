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
  const result = rows.length ? rows.map((row) => ({ ...row })) : [emptyParamRow()]
  const last = result[result.length - 1]
  if (hasParamContent(last)) {
    result.push(emptyParamRow())
  }
  return result
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
