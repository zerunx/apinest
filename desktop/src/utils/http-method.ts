export const PROTOCOLS = ['HTTP', 'HTTPS', 'WS', 'TCP', 'UDP', 'GRPC'] as const

export type Protocol = (typeof PROTOCOLS)[number]

export const HTTP_METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'] as const

const METHOD_SHORT: Record<string, string> = {
  DELETE: 'DEL',
  OPTIONS: 'OPT',
  GRPC: 'gRPC',
}

/** Protocol groups that share request-param shape and can switch after create. */
export type ProtocolFamily = 'http' | 'ws' | 'tcp' | 'grpc'

export function protocolFamily(protocol: string): ProtocolFamily {
  const p = protocol.toUpperCase()
  if (p === 'HTTPS' || p === 'HTTP') return 'http'
  // Legacy "WSS" stored in older interfaces maps to the ws family.
  if (p === 'WS' || p === 'WSS') return 'ws'
  if (p === 'TCP' || p === 'UDP') return 'tcp'
  // Legacy "RPC" (with HSF/DUBBO/…) maps to gRPC.
  if (p === 'GRPC' || p === 'RPC') return 'grpc'
  return 'http'
}

export function protocolLabel(protocol: string): string {
  const p = protocol.toUpperCase()
  if (p === 'GRPC' || p === 'RPC') return 'gRPC'
  return p === 'WSS' ? 'WS' : p
}

/** Same family (HTTP↔HTTPS, TCP↔UDP) can switch after create. */
export function switchableProtocols(current: Protocol, isCreate: boolean): Protocol[] {
  if (isCreate) return [...PROTOCOLS]
  const family = protocolFamily(current)
  if (family === 'http') return ['HTTP', 'HTTPS']
  if (family === 'ws') return ['WS']
  if (family === 'tcp') return ['TCP', 'UDP']
  if (family === 'grpc') return ['GRPC']
  return [current]
}

/** WS / TCP / UDP / gRPC use one select (no separate method box). */
export function isProtocolOnlySelect(protocol: string): boolean {
  const family = protocolFamily(protocol)
  return family === 'ws' || family === 'tcp' || family === 'grpc'
}

export function methodsForProtocol(protocol: string): string[] {
  const p = protocol.toUpperCase()
  if (p === 'GRPC' || p === 'RPC') return ['GRPC']
  if (p === 'HTTP' || p === 'HTTPS') return [...HTTP_METHODS]
  if (p === 'WS' || p === 'WSS' || p === 'TCP' || p === 'UDP') {
    // Normalize legacy WSS method label to WS.
    return [p === 'WSS' ? 'WS' : p]
  }
  return [...HTTP_METHODS]
}

export function shortMethodLabel(method: string): string {
  const upper = method.toUpperCase()
  return METHOD_SHORT[upper] ?? upper
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
