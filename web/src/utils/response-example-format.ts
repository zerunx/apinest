const FORMAT_OPTIONS = ['JSON', 'XML', 'Raw'] as const
const DATA_TYPE_OPTIONS = ['Object', 'Array', 'String', 'Number', 'Boolean'] as const

export type ResponseExampleFormat = (typeof FORMAT_OPTIONS)[number]
export type ResponseExampleDataType = (typeof DATA_TYPE_OPTIONS)[number]

export function exampleFormatToContentType(
  format: string,
  dataType: string,
): string {
  if (format === 'XML') return 'application/xml'
  if (format === 'Raw') {
    if (dataType === 'HTML') return 'text/html'
    if (dataType === 'XML') return 'application/xml'
    return 'text/plain'
  }
  return 'application/json'
}

export function contentTypeToExampleFormat(contentType: string): {
  format: ResponseExampleFormat
  data_type: ResponseExampleDataType
} {
  const normalized = contentType.trim().toLowerCase()
  if (normalized.includes('xml')) {
    return { format: 'XML', data_type: 'Object' }
  }
  if (normalized.includes('html')) {
    return { format: 'Raw', data_type: 'String' }
  }
  if (normalized.includes('plain')) {
    return { format: 'Raw', data_type: 'String' }
  }
  return { format: 'JSON', data_type: 'Object' }
}

export function exampleFormatToHighlightType(format: string, dataType: string): string {
  if (format === 'JSON') return 'JSON'
  if (format === 'XML') return 'XML'
  if (dataType === 'String') return 'Text'
  if (dataType === 'Number' || dataType === 'Boolean') return 'Text'
  return 'JSON'
}

export function enrichResponseExample<T extends { content_type: string; format?: string; data_type?: string }>(
  example: T,
): T {
  const derived = contentTypeToExampleFormat(example.content_type)
  return {
    ...example,
    format: example.format || derived.format,
    data_type: example.data_type || derived.data_type,
  }
}

export function normalizeResponseExampleForSave<
  T extends { content_type: string; format?: string; data_type?: string },
>(example: T): T {
  const format = example.format || 'JSON'
  const dataType = example.data_type || 'Object'
  return {
    ...example,
    format,
    data_type: dataType,
    content_type: exampleFormatToContentType(format, dataType),
  }
}
