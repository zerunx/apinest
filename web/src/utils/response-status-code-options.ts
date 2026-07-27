/** HTTP status codes for quick selection in response results. */
export const RESPONSE_STATUS_CODE_OPTIONS = [200, 401, 403, 404, 422, 500, 502] as const

export function isValidHttpStatusCode(value: number): boolean {
  return Number.isInteger(value) && value >= 100 && value <= 599
}

export function parseHttpStatusCodeInput(value: string): number | null {
  const trimmed = value.trim()
  if (!trimmed) return null
  const code = Number.parseInt(trimmed, 10)
  if (!Number.isFinite(code) || String(code) !== trimmed) return null
  return isValidHttpStatusCode(code) ? code : null
}

export function searchResponseStatusCodes(query: string) {
  const keyword = query.trim()
  return RESPONSE_STATUS_CODE_OPTIONS
    .filter((code) => !keyword || String(code).includes(keyword))
    .map((value) => ({ value: String(value) }))
}
