import {
  compactFieldTree,
  emptyFieldNode,
  hasFieldContent,
  type FieldTreeNode,
} from '@/utils/interface-field-tree'

export type BodyImportMode = 'replace' | 'append' | 'merge'

function deepCloneTree(nodes: FieldTreeNode[]): FieldTreeNode[] {
  return nodes.map((node) => ({
    ...node,
    children: node.children ? deepCloneTree(node.children) : [],
  }))
}

function inferFieldType(value: unknown): string {
  if (Array.isArray(value)) return 'array'
  if (value === null || value === undefined) return 'string'
  if (typeof value === 'object') return 'object'
  if (typeof value === 'number') return 'number'
  if (typeof value === 'boolean') return 'boolean'
  return 'string'
}

function formatExample(value: unknown): string {
  if (value === null || value === undefined) return ''
  if (typeof value === 'object') return ''
  if (typeof value === 'string') return value
  return String(value)
}

function valueToField(name: string, value: unknown, parentId = 0): FieldTreeNode {
  const type = inferFieldType(value)
  const field = emptyFieldNode(parentId)
  field.name = name
  field.type = type
  field.example = formatExample(value)

  if (type === 'object' && value && typeof value === 'object' && !Array.isArray(value)) {
    field.children = valueToFields(value as Record<string, unknown>)
  } else if (type === 'array' && Array.isArray(value) && value.length > 0) {
    const first = value[0]
    if (typeof first === 'object' && first !== null && !Array.isArray(first)) {
      field.children = valueToFields(first as Record<string, unknown>)
    }
  }

  return field
}

function valueToFields(obj: Record<string, unknown>): FieldTreeNode[] {
  return Object.entries(obj).map(([name, value]) => valueToField(name, value))
}

export function parseJsonImportText(jsonText: string): unknown {
  const text = jsonText.trim()
  if (!text) {
    throw new Error('empty')
  }
  return JSON.parse(text)
}

export function parseJsonToBodyFields(jsonText: string): FieldTreeNode[] {
  const parsed = parseJsonImportText(jsonText)
  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    throw new Error('root-object')
  }
  return valueToFields(parsed as Record<string, unknown>)
}

export function mergeFieldTrees(existing: FieldTreeNode[], imported: FieldTreeNode[]): FieldTreeNode[] {
  const result = deepCloneTree(existing).filter(hasFieldContent)

  for (const imp of imported) {
    const index = result.findIndex((row) => row.name === imp.name)
    if (index === -1) {
      result.push({
        ...imp,
        children: imp.children ? deepCloneTree(imp.children) : [],
      })
      continue
    }

    const current = result[index]
    result[index] = {
      ...current,
      type: imp.type || current.type,
      example: imp.example || current.example,
      description: imp.description || current.description,
      required: imp.required,
      children: mergeFieldTrees(current.children ?? [], imp.children ?? []),
    }
  }

  return result
}

function deepMergeJson(base: Record<string, unknown>, patch: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = { ...base }

  for (const [key, value] of Object.entries(patch)) {
    const current = result[key]
    if (
      value &&
      typeof value === 'object' &&
      !Array.isArray(value) &&
      current &&
      typeof current === 'object' &&
      !Array.isArray(current)
    ) {
      result[key] = deepMergeJson(current as Record<string, unknown>, value as Record<string, unknown>)
    } else {
      result[key] = value
    }
  }

  return result
}

export function applyBodyFieldsImport(
  existing: FieldTreeNode[],
  jsonText: string,
  mode: BodyImportMode,
): FieldTreeNode[] {
  const imported = parseJsonToBodyFields(jsonText)
  const current = compactFieldTree(existing)

  if (mode === 'replace') {
    return imported
  }
  if (mode === 'append') {
    return [...current, ...imported]
  }
  return mergeFieldTrees(current, imported)
}

export function applyRawBodyImport(
  existingRaw: string,
  jsonText: string,
  mode: BodyImportMode,
): string {
  const imported = parseJsonImportText(jsonText)
  const formatted = `${JSON.stringify(imported, null, 2)}\n`

  if (mode === 'replace') {
    return formatted
  }

  const currentText = existingRaw.trim()
  if (!currentText) {
    return formatted
  }

  const current = parseJsonImportText(currentText)

  if (
    mode === 'append' &&
    typeof current === 'object' &&
    current !== null &&
    !Array.isArray(current) &&
    typeof imported === 'object' &&
    imported !== null &&
    !Array.isArray(imported)
  ) {
    return `${JSON.stringify({ ...(current as Record<string, unknown>), ...(imported as Record<string, unknown>) }, null, 2)}\n`
  }

  if (
    mode === 'merge' &&
    typeof current === 'object' &&
    current !== null &&
    !Array.isArray(current) &&
    typeof imported === 'object' &&
    imported !== null &&
    !Array.isArray(imported)
  ) {
    return `${JSON.stringify(deepMergeJson(current as Record<string, unknown>, imported as Record<string, unknown>), null, 2)}\n`
  }

  return formatted
}
