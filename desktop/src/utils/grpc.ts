export type GrpcMetadataRow = {
  name: string
  example: string
  description: string
  enabled: boolean
}

export type GrpcMethod = {
  name: string
  rpc_path: string
  service_name: string
  request_type: string
  response_type: string
  client_streaming: boolean
  server_streaming: boolean
  message: string
  metadata: GrpcMetadataRow[]
}

export type GrpcService = {
  name: string
  full_name: string
  methods: GrpcMethod[]
}

export type GrpcCatalog = {
  proto_name: string
  proto_source: string
  package: string
  selected_rpc: string
  services: GrpcService[]
}

export function emptyGrpcMetadata(): GrpcMetadataRow[] {
  return [{ name: '', example: '', description: '', enabled: true }]
}

export function normalizeGrpcMetadata(rows: unknown): GrpcMetadataRow[] {
  const list = Array.isArray(rows)
    ? rows.map((row) => {
        const r = row as Record<string, unknown>
        return {
          name: String(r.name ?? r.key ?? ''),
          example: String(r.example ?? r.value ?? ''),
          description: String(r.description ?? ''),
          enabled: r.enabled !== false,
        }
      })
    : []
  if (!list.length || list[list.length - 1].name.trim() || list[list.length - 1].example.trim()) {
    list.push({ name: '', example: '', description: '', enabled: true })
  }
  return list
}

export function compactGrpcMetadata(rows: GrpcMetadataRow[]) {
  return rows
    .filter((r) => r.name.trim())
    .map((r) => ({
      name: r.name.trim(),
      example: r.example,
      description: r.description,
      enabled: r.enabled !== false,
    }))
}

export function parseGrpcCatalog(detail: Record<string, unknown> | null | undefined): GrpcCatalog | null {
  const grpc = detail?.grpc
  if (!grpc || typeof grpc !== 'object') return null
  const g = grpc as Record<string, unknown>
  const servicesRaw = Array.isArray(g.services) ? g.services : []
  const services: GrpcService[] = servicesRaw.map((svc) => {
    const s = svc as Record<string, unknown>
    const methodsRaw = Array.isArray(s.methods) ? s.methods : []
    return {
      name: String(s.name ?? ''),
      full_name: String(s.full_name ?? s.name ?? ''),
      methods: methodsRaw.map((m) => {
        const method = m as Record<string, unknown>
        return {
          name: String(method.name ?? ''),
          rpc_path: String(method.rpc_path ?? ''),
          service_name: String(method.service_name ?? s.full_name ?? ''),
          request_type: String(method.request_type ?? ''),
          response_type: String(method.response_type ?? ''),
          client_streaming: !!method.client_streaming,
          server_streaming: !!method.server_streaming,
          message: String(method.message ?? '{}'),
          metadata: normalizeGrpcMetadata(method.metadata),
        }
      }),
    }
  })
  if (!services.some((s) => s.methods.length)) return null
  return {
    proto_name: String(g.proto_name ?? ''),
    proto_source: String(g.proto_source ?? ''),
    package: String(g.package ?? ''),
    selected_rpc: String(g.selected_rpc ?? services[0]?.methods[0]?.rpc_path ?? ''),
    services,
  }
}

export function firstRpcPath(catalog: GrpcCatalog | null): string {
  return catalog?.services[0]?.methods[0]?.rpc_path ?? ''
}

export function findGrpcMethod(catalog: GrpcCatalog | null, rpcPath: string): GrpcMethod | null {
  if (!catalog || !rpcPath) return null
  for (const svc of catalog.services) {
    const found = svc.methods.find((m) => m.rpc_path === rpcPath)
    if (found) return found
  }
  return null
}

const SAMPLE_WORDS = [
  'apple',
  'river',
  'cloud',
  'maple',
  'ocean',
  'stone',
  'light',
  'cedar',
  'olive',
  'pearl',
  'quartz',
  'silk',
  'amber',
  'coral',
  'daisy',
  'ember',
  'flint',
  'grove',
  'hazel',
  'ivory',
  'jade',
  'lemon',
  'mint',
  'nova',
  'opal',
  'pine',
  'quest',
  'ridge',
  'sage',
  'tide',
  'ultra',
  'violet',
  'willow',
  'xenon',
  'yarrow',
  'zephyr',
  'north',
  'south',
  'delta',
  'echo',
]

function randomWord() {
  return SAMPLE_WORDS[Math.floor(Math.random() * SAMPLE_WORDS.length)] ?? 'sample'
}

function randomInt() {
  return Math.floor(Math.random() * 99) + 1
}

function randomFloat() {
  return Math.round((Math.random() * 99 + 1) * 10) / 10
}

function fillSampleValue(value: unknown): unknown {
  if (typeof value === 'string') return randomWord()
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) return randomInt()
    return Number.isInteger(value) ? randomInt() : randomFloat()
  }
  if (typeof value === 'boolean') return Math.random() < 0.5
  if (Array.isArray(value)) return value.map((item) => fillSampleValue(item))
  if (value && typeof value === 'object') {
    const out: Record<string, unknown> = {}
    for (const [key, nested] of Object.entries(value as Record<string, unknown>)) {
      out[key] = fillSampleValue(nested)
    }
    return out
  }
  return value
}

function parseJsonObject(raw: string): unknown | null {
  const trimmed = raw.trim()
  if (!trimmed) return null
  try {
    return JSON.parse(trimmed)
  } catch {
    return null
  }
}

function hasFillableFields(value: unknown): boolean {
  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
    return true
  }
  if (Array.isArray(value)) return value.some(hasFillableFields)
  if (value && typeof value === 'object') {
    return Object.values(value as Record<string, unknown>).some(hasFillableFields)
  }
  return false
}

/** Fill scalar fields in a Message JSON template with random sample values. */
export function fillGrpcMessageSamples(current: string, fallback = ''): string | null {
  const parsed = parseJsonObject(current) ?? parseJsonObject(fallback)
  if (parsed == null || !hasFillableFields(parsed)) return null
  const filled = fillSampleValue(parsed)
  return `${JSON.stringify(filled, null, 4)}\n`
}
