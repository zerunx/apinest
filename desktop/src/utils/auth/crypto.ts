const textEncoder = new TextEncoder()

export function utf8(bytes: string): Uint8Array {
  return textEncoder.encode(bytes)
}

export function bytesToHex(bytes: ArrayBuffer | Uint8Array): string {
  const arr = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
  let out = ''
  for (let i = 0; i < arr.length; i++) {
    out += arr[i]!.toString(16).padStart(2, '0')
  }
  return out
}

export function bytesToBase64(bytes: ArrayBuffer | Uint8Array): string {
  const arr = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
  let binary = ''
  for (let i = 0; i < arr.length; i++) binary += String.fromCharCode(arr[i]!)
  return btoa(binary)
}

export function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64)
  const out = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i)
  return out
}

export function base64UrlEncode(bytes: ArrayBuffer | Uint8Array | string): string {
  const data =
    typeof bytes === 'string'
      ? utf8(bytes)
      : bytes instanceof Uint8Array
        ? bytes
        : new Uint8Array(bytes)
  return bytesToBase64(data).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '')
}

export function randomHex(bytes = 16): string {
  const buf = new Uint8Array(bytes)
  crypto.getRandomValues(buf)
  return bytesToHex(buf)
}

export function randomAlnum(len = 8): string {
  const alphabet = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  const buf = new Uint8Array(len)
  crypto.getRandomValues(buf)
  let out = ''
  for (let i = 0; i < len; i++) out += alphabet[buf[i]! % alphabet.length]
  return out
}

/** Compact MD5 (Digest Auth). */
export function md5Hex(message: string): string {
  function cmn(q: number, a: number, b: number, x: number, s: number, t: number) {
    a = (a + q + x + t) | 0
    return (((a << s) | (a >>> (32 - s))) + b) | 0
  }
  function ff(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
    return cmn((b & c) | (~b & d), a, b, x, s, t)
  }
  function gg(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
    return cmn((b & d) | (c & ~d), a, b, x, s, t)
  }
  function hh(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
    return cmn(b ^ c ^ d, a, b, x, s, t)
  }
  function ii(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
    return cmn(c ^ (b | ~d), a, b, x, s, t)
  }

  const msg = unescape(encodeURIComponent(message))
  const n = msg.length
  const words: number[] = []
  for (let i = 0; i < n; i++) {
    words[i >> 2] |= (msg.charCodeAt(i) & 0xff) << ((i % 4) * 8)
  }
  words[n >> 2] |= 0x80 << ((n % 4) * 8)
  const bitLen = n * 8
  words[(((n + 8) >> 6) << 4) + 14] = bitLen

  let a = 1732584193
  let b = -271733879
  let c = -1732584194
  let d = 271733878

  for (let i = 0; i < words.length; i += 16) {
    const oa = a
    const ob = b
    const oc = c
    const od = d
    const w = (j: number) => words[i + j] | 0

    a = ff(a, b, c, d, w(0), 7, -680876936)
    d = ff(d, a, b, c, w(1), 12, -389564586)
    c = ff(c, d, a, b, w(2), 17, 606105819)
    b = ff(b, c, d, a, w(3), 22, -1044525330)
    a = ff(a, b, c, d, w(4), 7, -176418897)
    d = ff(d, a, b, c, w(5), 12, 1200080426)
    c = ff(c, d, a, b, w(6), 17, -1473231341)
    b = ff(b, c, d, a, w(7), 22, -45705983)
    a = ff(a, b, c, d, w(8), 7, 1770035416)
    d = ff(d, a, b, c, w(9), 12, -1958414417)
    c = ff(c, d, a, b, w(10), 17, -42063)
    b = ff(b, c, d, a, w(11), 22, -1990404162)
    a = ff(a, b, c, d, w(12), 7, 1804603682)
    d = ff(d, a, b, c, w(13), 12, -40341101)
    c = ff(c, d, a, b, w(14), 17, -1502002290)
    b = ff(b, c, d, a, w(15), 22, 1236535329)

    a = gg(a, b, c, d, w(1), 5, -165796510)
    d = gg(d, a, b, c, w(6), 9, -1069501632)
    c = gg(c, d, a, b, w(11), 14, 643717713)
    b = gg(b, c, d, a, w(0), 20, -373897302)
    a = gg(a, b, c, d, w(5), 5, -701558691)
    d = gg(d, a, b, c, w(10), 9, 38016083)
    c = gg(c, d, a, b, w(15), 14, -660478335)
    b = gg(b, c, d, a, w(4), 20, -405537848)
    a = gg(a, b, c, d, w(9), 5, 568446438)
    d = gg(d, a, b, c, w(14), 9, -1019803690)
    c = gg(c, d, a, b, w(3), 14, -187363961)
    b = gg(b, c, d, a, w(8), 20, 1163531501)
    a = gg(a, b, c, d, w(13), 5, -1444681467)
    d = gg(d, a, b, c, w(2), 9, -51403784)
    c = gg(c, d, a, b, w(7), 14, 1735328473)
    b = gg(b, c, d, a, w(12), 20, -1926607734)

    a = hh(a, b, c, d, w(5), 4, -378558)
    d = hh(d, a, b, c, w(8), 11, -2022574463)
    c = hh(c, d, a, b, w(11), 16, 1839030562)
    b = hh(b, c, d, a, w(14), 23, -35309556)
    a = hh(a, b, c, d, w(1), 4, -1530992060)
    d = hh(d, a, b, c, w(4), 11, 1272893353)
    c = hh(c, d, a, b, w(7), 16, -155497632)
    b = hh(b, c, d, a, w(10), 23, -1094730640)
    a = hh(a, b, c, d, w(13), 4, 681279174)
    d = hh(d, a, b, c, w(0), 11, -358537222)
    c = hh(c, d, a, b, w(3), 16, -722521979)
    b = hh(b, c, d, a, w(6), 23, 76029189)
    a = hh(a, b, c, d, w(9), 4, -640364487)
    d = hh(d, a, b, c, w(12), 11, -421815835)
    c = hh(c, d, a, b, w(15), 16, 530742520)
    b = hh(b, c, d, a, w(2), 23, -995338651)

    a = ii(a, b, c, d, w(0), 6, -198630844)
    d = ii(d, a, b, c, w(7), 10, 1126891415)
    c = ii(c, d, a, b, w(14), 15, -1416354905)
    b = ii(b, c, d, a, w(5), 21, -57434055)
    a = ii(a, b, c, d, w(12), 6, 1700485571)
    d = ii(d, a, b, c, w(3), 10, -1894986606)
    c = ii(c, d, a, b, w(10), 15, -1051523)
    b = ii(b, c, d, a, w(1), 21, -2054922799)
    a = ii(a, b, c, d, w(8), 6, 1873313359)
    d = ii(d, a, b, c, w(15), 10, -30611744)
    c = ii(c, d, a, b, w(6), 15, -1560198380)
    b = ii(b, c, d, a, w(13), 21, 1309151649)
    a = ii(a, b, c, d, w(4), 6, -145523070)
    d = ii(d, a, b, c, w(11), 10, -1120210379)
    c = ii(c, d, a, b, w(2), 15, 718787259)
    b = ii(b, c, d, a, w(9), 21, -343485551)

    a = (a + oa) | 0
    b = (b + ob) | 0
    c = (c + oc) | 0
    d = (d + od) | 0
  }

  function toHex(n: number) {
    let s = ''
    for (let j = 0; j < 4; j++) s += ((n >> (j * 8)) & 0xff).toString(16).padStart(2, '0')
    return s
  }
  return toHex(a) + toHex(b) + toHex(c) + toHex(d)
}

export async function shaHex(
  algo: 'SHA-1' | 'SHA-256' | 'SHA-384' | 'SHA-512',
  data: string | Uint8Array,
): Promise<string> {
  const buf = typeof data === 'string' ? utf8(data) : data
  const digest = await crypto.subtle.digest(algo, buf)
  return bytesToHex(digest)
}

export async function hmac(
  algo: 'SHA-1' | 'SHA-256' | 'SHA-384' | 'SHA-512',
  key: string | Uint8Array,
  data: string | Uint8Array,
): Promise<ArrayBuffer> {
  const keyBytes = typeof key === 'string' ? utf8(key) : key
  const dataBytes = typeof data === 'string' ? utf8(data) : data
  const cryptoKey = await crypto.subtle.importKey(
    'raw',
    keyBytes,
    { name: 'HMAC', hash: algo },
    false,
    ['sign'],
  )
  return crypto.subtle.sign('HMAC', cryptoKey, dataBytes)
}

export async function hmacHex(
  algo: 'SHA-1' | 'SHA-256' | 'SHA-384' | 'SHA-512',
  key: string | Uint8Array,
  data: string | Uint8Array,
): Promise<string> {
  return bytesToHex(await hmac(algo, key, data))
}

export async function hmacBase64(
  algo: 'SHA-1' | 'SHA-256' | 'SHA-384' | 'SHA-512',
  key: string | Uint8Array,
  data: string | Uint8Array,
): Promise<string> {
  return bytesToBase64(await hmac(algo, key, data))
}

export function percentEncode(value: string): string {
  return encodeURIComponent(value).replace(/[!'()*]/g, (c) =>
    `%${c.charCodeAt(0).toString(16).toUpperCase()}`,
  )
}

export function parseUrlParts(url: string): {
  host: string
  path: string
  query: string
  protocol: string
  port: string
} {
  try {
    const withScheme = /^[a-z][a-z0-9+.-]*:\/\//i.test(url) ? url : `http://${url}`
    const u = new URL(withScheme)
    return {
      host: u.hostname,
      path: u.pathname || '/',
      query: u.search.startsWith('?') ? u.search.slice(1) : u.search,
      protocol: u.protocol.replace(':', ''),
      port: u.port,
    }
  } catch {
    return { host: '', path: '/', query: '', protocol: 'http', port: '' }
  }
}

export function pemToArrayBuffer(pem: string): ArrayBuffer {
  const cleaned = pem
    .replace(/-----BEGIN [^-]+-----/g, '')
    .replace(/-----END [^-]+-----/g, '')
    .replace(/\s+/g, '')
  return base64ToBytes(cleaned).buffer
}
