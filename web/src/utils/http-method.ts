export const CREATE_PROTOCOLS = ['HTTP', 'HTTPS', 'WS', 'TCP', 'UDP'] as const

export const PROTOCOLS = [...CREATE_PROTOCOLS, 'GRPC'] as const

export type Protocol = (typeof PROTOCOLS)[number]

export const HTTP_METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'] as const

const METHOD_SHORT: Record<string, string> = {
  DELETE: 'DEL',
  OPTIONS: 'OPT',
  GRPC: 'gRPC',
}

export type ProtocolFamily = 'http' | 'ws' | 'tcp' | 'grpc'

export function protocolFamily(protocol: string): ProtocolFamily {
  const p = protocol.toUpperCase()
  if (p === 'HTTPS' || p === 'HTTP') return 'http'
  if (p === 'WS' || p === 'WSS') return 'ws'
  if (p === 'TCP' || p === 'UDP') return 'tcp'
  if (p === 'GRPC' || p === 'RPC') return 'grpc'
  return 'http'
}

export function protocolLabel(protocol: string): string {
  const p = protocol.toUpperCase()
  if (p === 'GRPC' || p === 'RPC') return 'gRPC'
  return p === 'WSS' ? 'WS' : p
}

export function switchableProtocols(current: Protocol, isCreate: boolean): Protocol[] {
  if (isCreate) return [...CREATE_PROTOCOLS]
  const family = protocolFamily(current)
  if (family === 'http') return ['HTTP', 'HTTPS']
  if (family === 'ws') return ['WS']
  if (family === 'tcp') return ['TCP', 'UDP']
  if (family === 'grpc') return ['GRPC']
  return [current]
}

export function isProtocolOnlySelect(protocol: string): boolean {
  const family = protocolFamily(protocol)
  return family === 'ws' || family === 'tcp' || family === 'grpc'
}

export function methodsForProtocol(protocol: string): string[] {
  const p = protocol.toUpperCase()
  if (p === 'GRPC' || p === 'RPC') return ['GRPC']
  if (p === 'HTTP' || p === 'HTTPS') return [...HTTP_METHODS]
  if (p === 'WS' || p === 'WSS' || p === 'TCP' || p === 'UDP') {
    return [p === 'WSS' ? 'WS' : p]
  }
  return [...HTTP_METHODS]
}

export function shortMethodLabel(method: string): string {
  const upper = method.toUpperCase()
  return METHOD_SHORT[upper] ?? upper
}

export function methodTone(method: string): string {
  const upper = method.toUpperCase()
  if (upper === 'DEL' || upper === 'DELETE') return 'delete'
  if (upper === 'OPT' || upper === 'OPTIONS') return 'options'
  if (upper === 'WSS' || upper === 'WS') return 'ws'
  if (upper === 'RPC' || upper === 'GRPC') return 'grpc'
  return upper.toLowerCase()
}

export function isKnownProtocol(value: unknown): boolean {
  if (typeof value !== 'string') return false
  const upper = value.toUpperCase()
  if (upper === 'WSS' || upper === 'RPC') return true
  return (PROTOCOLS as readonly string[]).includes(upper)
}

export function normalizeProtocol(value: unknown, fallback: Protocol = 'HTTP'): Protocol {
  if (typeof value !== 'string') return fallback
  const upper = value.toUpperCase()
  if (upper === 'WSS') return 'WS'
  if (upper === 'RPC') return 'GRPC'
  return (PROTOCOLS as readonly string[]).includes(upper) ? (upper as Protocol) : fallback
}

export function inferProtocol(method: string, url = ''): Protocol {
  const m = method.toUpperCase()
  if (m === 'WS' || m === 'WSS') return 'WS'
  if (m === 'TCP') return 'TCP'
  if (m === 'UDP') return 'UDP'
  if (m === 'GRPC' || m === 'RPC') return 'GRPC'
  if (/^https:\/\//i.test(url)) return 'HTTPS'
  if (/^wss:\/\//i.test(url) || /^ws:\/\//i.test(url)) return 'WS'
  if (/^tcp:\/\//i.test(url)) return 'TCP'
  if (/^udp:\/\//i.test(url)) return 'UDP'
  return 'HTTP'
}

export function applyProtocolScheme(url: string, protocol: Protocol): string {
  const schemeMap: Record<Protocol, string | null> = {
    HTTP: 'http',
    HTTPS: 'https',
    WS: 'ws',
    TCP: 'tcp',
    UDP: 'udp',
    GRPC: null,
  }
  const scheme = schemeMap[protocol]
  if (!scheme || !/^[a-z][a-z0-9+.-]*:\/\//i.test(url)) return url
  return url.replace(/^[a-z][a-z0-9+.-]*:\/\//i, `${scheme}://`)
}
