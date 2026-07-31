export const PROTOCOLS = ['HTTP', 'HTTPS', 'WS', 'WSS', 'TCP', 'UDP', 'RPC'] as const

export type Protocol = (typeof PROTOCOLS)[number]

export const HTTP_METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'] as const

export const RPC_METHODS = ['HSF', 'DUBBO', 'GRPC', 'SOFA', 'TR'] as const

const METHOD_SHORT: Record<string, string> = {
  DELETE: 'DEL',
  OPTIONS: 'OPT',
}

/** Protocol groups that share request-param shape and can switch after create. */
export type ProtocolFamily = 'http' | 'ws' | 'tcp' | 'rpc'

export function protocolFamily(protocol: string): ProtocolFamily {
  const p = protocol.toUpperCase()
  if (p === 'HTTPS' || p === 'HTTP') return 'http'
  if (p === 'WS' || p === 'WSS') return 'ws'
  if (p === 'TCP' || p === 'UDP') return 'tcp'
  if (p === 'RPC') return 'rpc'
  return 'http'
}

/** Same family (HTTP↔HTTPS, WS↔WSS, TCP↔UDP) can switch after create. */
export function switchableProtocols(current: Protocol, isCreate: boolean): Protocol[] {
  if (isCreate) return [...PROTOCOLS]
  const family = protocolFamily(current)
  if (family === 'http') return ['HTTP', 'HTTPS']
  if (family === 'ws') return ['WS', 'WSS']
  if (family === 'tcp') return ['TCP', 'UDP']
  if (family === 'rpc') return ['RPC']
  return [current]
}

/** WS / WSS / TCP / UDP use one select (no separate method box). */
export function isProtocolOnlySelect(protocol: string): boolean {
  const family = protocolFamily(protocol)
  return family === 'ws' || family === 'tcp'
}

export function methodsForProtocol(protocol: string): string[] {
  const p = protocol.toUpperCase()
  if (p === 'RPC') return [...RPC_METHODS]
  if (p === 'HTTP' || p === 'HTTPS') return [...HTTP_METHODS]
  if (p === 'WS' || p === 'WSS' || p === 'TCP' || p === 'UDP') return [p]
  return [...HTTP_METHODS]
}

export function shortMethodLabel(method: string): string {
  const upper = method.toUpperCase()
  return METHOD_SHORT[upper] ?? upper
}

export function isKnownProtocol(value: unknown): value is Protocol {
  return typeof value === 'string' && (PROTOCOLS as readonly string[]).includes(value.toUpperCase())
}

export function normalizeProtocol(value: unknown, fallback: Protocol = 'HTTP'): Protocol {
  if (typeof value !== 'string') return fallback
  const upper = value.toUpperCase()
  return (PROTOCOLS as readonly string[]).includes(upper) ? (upper as Protocol) : fallback
}
