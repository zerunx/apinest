import {
  base64UrlEncode,
  bytesToBase64,
  bytesToHex,
  hmac,
  hmacBase64,
  md5Hex,
  parseUrlParts,
  pemToArrayBuffer,
  percentEncode,
  randomAlnum,
  randomHex,
  shaHex,
  utf8,
} from './crypto'
import type { AuthApplyContext, AuthArtifacts, AuthConfig } from './types'

function pushHeader(out: AuthArtifacts, key: string, value: string) {
  if (!key.trim() || !value) return
  out.headers.push({ key: key.trim(), value })
}

function pushQuery(out: AuthArtifacts, key: string, value: string) {
  if (!key.trim()) return
  out.query.push({ key: key.trim(), value })
}

function basicValue(user: string, password: string): string | null {
  try {
    return `Basic ${btoa(`${user}:${password}`)}`
  } catch {
    return null
  }
}

async function digestHash(algo: AuthConfig['digestAlgorithm'], data: string): Promise<string> {
  if (algo.startsWith('MD5')) return md5Hex(data)
  if (algo.startsWith('SHA-512-256')) {
    const full = await shaHex('SHA-512', data)
    return full.slice(0, 64)
  }
  return shaHex('SHA-256', data)
}

async function applyDigest(cfg: AuthConfig, ctx: AuthApplyContext, out: AuthArtifacts) {
  const username = cfg.digestUsername
  const password = cfg.digestPassword
  if (!username && !password) return
  const realm = cfg.digestRealm
  const nonce = cfg.digestNonce
  if (!realm || !nonce) return

  const method = (ctx.method || 'GET').toUpperCase()
  const parts = parseUrlParts(ctx.url)
  const uri = parts.path + (parts.query ? `?${parts.query}` : '')
  const algo = cfg.digestAlgorithm
  const qop = cfg.digestQop
  const nc = cfg.digestNonceCount || '00000001'
  const cnonce = cfg.digestClientNonce || randomHex(8)
  const opaque = cfg.digestOpaque

  let ha1 = await digestHash(algo, `${username}:${realm}:${password}`)
  if (algo.endsWith('-sess')) {
    ha1 = await digestHash(algo, `${ha1}:${nonce}:${cnonce}`)
  }

  let ha2: string
  if (qop === 'auth-int') {
    const bodyHash = await digestHash(algo, ctx.body || '')
    ha2 = await digestHash(algo, `${method}:${uri}:${bodyHash}`)
  } else {
    ha2 = await digestHash(algo, `${method}:${uri}`)
  }

  let response: string
  if (qop === 'auth' || qop === 'auth-int') {
    response = await digestHash(algo, `${ha1}:${nonce}:${nc}:${cnonce}:${qop}:${ha2}`)
  } else {
    response = await digestHash(algo, `${ha1}:${nonce}:${ha2}`)
  }

  const fields = [
    `username="${username}"`,
    `realm="${realm}"`,
    `nonce="${nonce}"`,
    `uri="${uri}"`,
    `algorithm=${algo}`,
    `response="${response}"`,
  ]
  if (qop) {
    fields.push(`qop=${qop}`, `nc=${nc}`, `cnonce="${cnonce}"`)
  }
  if (opaque) fields.push(`opaque="${opaque}"`)
  pushHeader(out, 'Authorization', `Digest ${fields.join(', ')}`)
}

async function applyHawk(cfg: AuthConfig, ctx: AuthApplyContext, out: AuthArtifacts) {
  const id = cfg.hawkAuthId.trim()
  const key = cfg.hawkAuthKey
  if (!id || !key) return

  const ts = cfg.hawkTimestamp.trim() || String(Math.floor(Date.now() / 1000))
  const nonce = cfg.hawkNonce.trim() || randomAlnum(6)
  const parts = parseUrlParts(ctx.url)
  const port =
    parts.port ||
    (parts.protocol === 'https' || parts.protocol === 'wss' ? '443' : '80')
  const resource = parts.path + (parts.query ? `?${parts.query}` : '')
  const ext = cfg.hawkExt
  const app = cfg.hawkApp
  const dlg = cfg.hawkDlg

  let hashLine = ''
  if (cfg.hawkIncludePayloadHash && ctx.body) {
    const hashAlgo = cfg.hawkAlgorithm === 'sha1' ? 'SHA-1' : 'SHA-256'
    const normalized = `hawk.1.payload\n${ctx.contentType || 'text/plain'}\n${ctx.body}\n`
    const digestBuf = await crypto.subtle.digest(hashAlgo, utf8(normalized))
    hashLine = bytesToBase64(digestBuf)
  }

  const header = [
    'hawk.1.header',
    ts,
    nonce,
    (ctx.method || 'GET').toUpperCase(),
    resource,
    parts.host.toLowerCase(),
    port,
    hashLine,
    ext,
  ].join('\n')

  const macAlgo = cfg.hawkAlgorithm === 'sha1' ? 'SHA-1' : 'SHA-256'
  const mac = await hmacBase64(macAlgo, key, `${header}\n`)

  const attrs = [`id="${id}"`, `ts="${ts}"`, `nonce="${nonce}"`, `mac="${mac}"`]
  if (hashLine) attrs.push(`hash="${hashLine}"`)
  if (ext) attrs.push(`ext="${ext}"`)
  if (app) attrs.push(`app="${app}"`)
  if (dlg) attrs.push(`dlg="${dlg}"`)
  pushHeader(out, 'Authorization', `Hawk ${attrs.join(', ')}`)
}

async function applyJwt(cfg: AuthConfig, out: AuthArtifacts) {
  const secret = cfg.jwtSecret
  if (!secret) return
  let payload: Record<string, unknown>
  try {
    payload = JSON.parse(cfg.jwtPayload || '{}') as Record<string, unknown>
  } catch {
    return
  }
  let extraHeaders: Record<string, unknown> = {}
  if (cfg.jwtHeaders.trim()) {
    try {
      extraHeaders = JSON.parse(cfg.jwtHeaders) as Record<string, unknown>
    } catch {
      return
    }
  }
  const alg = cfg.jwtAlgorithm
  const header = { alg, typ: 'JWT', ...extraHeaders }
  const encodedHeader = base64UrlEncode(JSON.stringify(header))
  const encodedPayload = base64UrlEncode(JSON.stringify(payload))
  const signingInput = `${encodedHeader}.${encodedPayload}`
  const hash =
    alg === 'HS384' ? 'SHA-384' : alg === 'HS512' ? 'SHA-512' : 'SHA-256'
  let keyBytes: Uint8Array
  try {
    keyBytes = cfg.jwtSecretBase64 ? Uint8Array.from(atob(secret), (c) => c.charCodeAt(0)) : utf8(secret)
  } catch {
    return
  }
  const sig = await hmac(hash, keyBytes, signingInput)
  const token = `${signingInput}.${base64UrlEncode(sig)}`
  const prefix = (cfg.jwtHeaderPrefix || 'Bearer').trim()
  pushHeader(out, 'Authorization', prefix ? `${prefix} ${token}` : token)
}

async function applyOauth1(cfg: AuthConfig, ctx: AuthApplyContext, out: AuthArtifacts) {
  const consumerKey = cfg.oauth1ConsumerKey.trim()
  if (!consumerKey) return

  const timestamp = cfg.oauth1Timestamp.trim() || String(Math.floor(Date.now() / 1000))
  const nonce = cfg.oauth1Nonce.trim() || randomAlnum(11)
  const version = cfg.oauth1Version.trim() || '1.0'
  const method = cfg.oauth1SignatureMethod

  const oauthParams: Record<string, string> = {
    oauth_consumer_key: consumerKey,
    oauth_nonce: nonce,
    oauth_signature_method: method,
    oauth_timestamp: timestamp,
    oauth_version: version,
  }
  if (cfg.oauth1Token.trim()) oauthParams.oauth_token = cfg.oauth1Token.trim()
  if (cfg.oauth1Callback.trim()) oauthParams.oauth_callback = cfg.oauth1Callback.trim()
  if (cfg.oauth1Verifier.trim()) oauthParams.oauth_verifier = cfg.oauth1Verifier.trim()

  const parts = parseUrlParts(ctx.url)
  const baseUrl = `${parts.protocol}://${parts.host}${parts.port ? `:${parts.port}` : ''}${parts.path}`

  const urlParams: Array<[string, string]> = []
  if (parts.query) {
    for (const pair of parts.query.split('&')) {
      if (!pair) continue
      const eq = pair.indexOf('=')
      const k = eq >= 0 ? pair.slice(0, eq) : pair
      const v = eq >= 0 ? pair.slice(eq + 1) : ''
      urlParams.push([decodeURIComponent(k), decodeURIComponent(v)])
    }
  }

  const allParams = [
    ...urlParams,
    ...Object.entries(oauthParams),
  ].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))

  const paramString = allParams
    .map(([k, v]) => `${percentEncode(k)}=${percentEncode(v)}`)
    .join('&')
  const baseString = [
    (ctx.method || 'GET').toUpperCase(),
    percentEncode(baseUrl),
    percentEncode(paramString),
  ].join('&')

  const signingKey = `${percentEncode(cfg.oauth1ConsumerSecret)}&${percentEncode(cfg.oauth1TokenSecret)}`
  let signature: string
  if (method === 'PLAINTEXT') {
    signature = signingKey
  } else {
    const hash = method === 'HMAC-SHA256' ? 'SHA-256' : 'SHA-1'
    signature = await hmacBase64(hash, signingKey, baseString)
  }
  oauthParams.oauth_signature = signature

  if (cfg.oauth1AddParamsTo === 'query' || cfg.oauth1AddParamsTo === 'body') {
    // Body injection requires form-urlencoded rewrite; query is the practical fallback.
    for (const [k, v] of Object.entries(oauthParams)) pushQuery(out, k, v)
    return
  }

  const realm = cfg.oauth1Realm.trim()
  const headerParams = Object.entries(oauthParams)
    .map(([k, v]) => `${percentEncode(k)}="${percentEncode(v)}"`)
    .join(', ')
  pushHeader(
    out,
    'Authorization',
    realm ? `OAuth realm="${percentEncode(realm)}", ${headerParams}` : `OAuth ${headerParams}`,
  )
}

async function applyEdgeGrid(cfg: AuthConfig, ctx: AuthApplyContext, out: AuthArtifacts) {
  const clientToken = cfg.edgeClientToken.trim()
  const accessToken = cfg.edgeAccessToken.trim()
  const clientSecret = cfg.edgeClientSecret
  if (!clientToken || !accessToken || !clientSecret) return

  const timestamp =
    cfg.edgeTimestamp.trim() ||
    new Date().toISOString().replace(/[-:]/g, '').replace(/\.\d{3}/, '')
  const nonce = cfg.edgeNonce.trim() || crypto.randomUUID()
  const parts = parseUrlParts(ctx.url)
  const method = (ctx.method || 'GET').toUpperCase()
  const path = parts.path + (parts.query ? `?${parts.query}` : '')

  const headersToSign = cfg.edgeHeadersToSign
    .split(',')
    .map((h) => h.trim().toLowerCase())
    .filter(Boolean)
  // Without full request header map, only path/method/body are signed (Postman-compatible core).
  void headersToSign

  let bodyHash = ''
  if (ctx.body && (method === 'POST' || method === 'PUT' || method === 'PATCH')) {
    bodyHash = bytesToBase64(await crypto.subtle.digest('SHA-256', utf8(ctx.body))).slice(0, 16)
  }

  const dataToSign = [
    method,
    'https', // EdgeGrid typically over HTTPS
    parts.host,
    path,
    '', // canonical headers
    bodyHash,
    `EG1-HMAC-SHA256;${timestamp};${nonce};`,
  ].join('\t')

  const signingKey = await hmac('SHA-256', clientSecret, `EG1-HMAC-SHA256;${timestamp};`)
  const signature = await hmacBase64('SHA-256', new Uint8Array(signingKey), dataToSign)
  const auth = [
    `client_token=${clientToken}`,
    `access_token=${accessToken}`,
    `timestamp=${timestamp}`,
    `nonce=${nonce}`,
    `signature=${signature}`,
  ].join(';')
  pushHeader(out, 'Authorization', `EG1-HMAC-SHA256 ${auth}`)
}

async function applyAsap(cfg: AuthConfig, out: AuthArtifacts) {
  const issuer = cfg.asapIssuer.trim()
  const audience = cfg.asapAudience.trim()
  const kid = cfg.asapKeyId.trim()
  const privateKeyPem = cfg.asapPrivateKey.trim()
  if (!issuer || !audience || !kid || !privateKeyPem) return

  const now = Math.floor(Date.now() / 1000)
  const exp = now + (Number(cfg.asapExpiresIn) || 3600)
  const claims: Record<string, unknown> = {
    iss: issuer,
    aud: audience,
    iat: now,
    exp,
    jti: randomHex(16),
  }
  if (cfg.asapSubject.trim()) claims.sub = cfg.asapSubject.trim()
  if (cfg.asapClaims.trim()) {
    try {
      Object.assign(claims, JSON.parse(cfg.asapClaims) as Record<string, unknown>)
    } catch {
      /* ignore */
    }
  }

  const header = { alg: 'RS256', typ: 'JWT', kid }
  const encodedHeader = base64UrlEncode(JSON.stringify(header))
  const encodedPayload = base64UrlEncode(JSON.stringify(claims))
  const signingInput = `${encodedHeader}.${encodedPayload}`

  try {
    const key = await crypto.subtle.importKey(
      'pkcs8',
      pemToArrayBuffer(privateKeyPem),
      { name: 'RSASSA-PKCS1-v1_5', hash: 'SHA-256' },
      false,
      ['sign'],
    )
    const sig = await crypto.subtle.sign('RSASSA-PKCS1-v1_5', key, utf8(signingInput))
    const token = `${signingInput}.${base64UrlEncode(sig)}`
    pushHeader(out, 'Authorization', `Bearer ${token}`)
  } catch {
    /* invalid key */
  }
}

async function applyAwsSigV4(cfg: AuthConfig, ctx: AuthApplyContext, out: AuthArtifacts) {
  const accessKey = cfg.awsAccessKey.trim()
  const secretKey = cfg.awsSecretKey
  const region = cfg.awsRegion.trim()
  const service = cfg.awsService.trim()
  if (!accessKey || !secretKey || !region || !service) return

  const parts = parseUrlParts(ctx.url)
  const method = (ctx.method || 'GET').toUpperCase()
  const amzDate = new Date()
    .toISOString()
    .replace(/[-:]/g, '')
    .replace(/\.\d{3}/, '')
  const dateStamp = amzDate.slice(0, 8)
  const payloadHash = await shaHex('SHA-256', ctx.body || '')

  const headersMap: Record<string, string> = {
    host: parts.host + (parts.port ? `:${parts.port}` : ''),
    'x-amz-date': amzDate,
  }
  if (cfg.awsSessionToken.trim()) headersMap['x-amz-security-token'] = cfg.awsSessionToken.trim()

  const signedHeaderKeys = Object.keys(headersMap).sort()
  const canonicalHeaders = signedHeaderKeys.map((k) => `${k}:${headersMap[k]}\n`).join('')
  const signedHeaders = signedHeaderKeys.join(';')

  const canonicalQuery = parts.query
    ? parts.query
        .split('&')
        .filter(Boolean)
        .map((pair) => {
          const eq = pair.indexOf('=')
          const k = eq >= 0 ? pair.slice(0, eq) : pair
          const v = eq >= 0 ? pair.slice(eq + 1) : ''
          return [decodeURIComponent(k), decodeURIComponent(v)] as const
        })
        .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
        .map(([k, v]) => `${percentEncode(k)}=${percentEncode(v)}`)
        .join('&')
    : ''

  const canonicalRequest = [
    method,
    parts.path || '/',
    canonicalQuery,
    canonicalHeaders,
    signedHeaders,
    payloadHash,
  ].join('\n')

  const credentialScope = `${dateStamp}/${region}/${service}/aws4_request`
  const stringToSign = [
    'AWS4-HMAC-SHA256',
    amzDate,
    credentialScope,
    await shaHex('SHA-256', canonicalRequest),
  ].join('\n')

  const kDate = await hmac('SHA-256', `AWS4${secretKey}`, dateStamp)
  const kRegion = await hmac('SHA-256', new Uint8Array(kDate), region)
  const kService = await hmac('SHA-256', new Uint8Array(kRegion), service)
  const kSigning = await hmac('SHA-256', new Uint8Array(kService), 'aws4_request')
  const signature = bytesToHex(await hmac('SHA-256', new Uint8Array(kSigning), stringToSign))

  if (cfg.awsAddAuthDataTo === 'query') {
    pushQuery(out, 'X-Amz-Algorithm', 'AWS4-HMAC-SHA256')
    pushQuery(out, 'X-Amz-Credential', `${accessKey}/${credentialScope}`)
    pushQuery(out, 'X-Amz-Date', amzDate)
    pushQuery(out, 'X-Amz-SignedHeaders', signedHeaders)
    pushQuery(out, 'X-Amz-Signature', signature)
    if (cfg.awsSessionToken.trim()) pushQuery(out, 'X-Amz-Security-Token', cfg.awsSessionToken.trim())
    return
  }

  pushHeader(out, 'X-Amz-Date', amzDate)
  if (cfg.awsSessionToken.trim()) {
    pushHeader(out, 'X-Amz-Security-Token', cfg.awsSessionToken.trim())
  }
  pushHeader(
    out,
    'Authorization',
    `AWS4-HMAC-SHA256 Credential=${accessKey}/${credentialScope}, SignedHeaders=${signedHeaders}, Signature=${signature}`,
  )
}

function applyNtlm(cfg: AuthConfig, out: AuthArtifacts) {
  // Type 1 (negotiate) message — full handshake needs 401 challenge round-trips.
  const domain = cfg.ntlmDomain
  const workstation = cfg.ntlmWorkstation
  const domainBytes = utf8(domain)
  const wsBytes = utf8(workstation)
  const bodyLen = 32
  const domainOffset = bodyLen
  const wsOffset = domainOffset + domainBytes.length
  const msg = new Uint8Array(wsOffset + wsBytes.length)
  const view = new DataView(msg.buffer)
  // NTLMSSP\0
  const sig = utf8('NTLMSSP\0')
  msg.set(sig, 0)
  view.setUint32(8, 1, true) // Type 1
  // flags: Negotiate Unicode | OEM | Request Target | NTLM | Always Sign | Extended Session Security
  view.setUint32(12, 0x00088207, true)
  view.setUint16(16, domainBytes.length, true)
  view.setUint16(18, domainBytes.length, true)
  view.setUint32(20, domainOffset, true)
  view.setUint16(24, wsBytes.length, true)
  view.setUint16(26, wsBytes.length, true)
  view.setUint32(28, wsOffset, true)
  msg.set(domainBytes, domainOffset)
  msg.set(wsBytes, wsOffset)
  pushHeader(out, 'Authorization', `NTLM ${bytesToBase64(msg)}`)
  void cfg.ntlmUsername
  void cfg.ntlmPassword
}

export async function applyAuth(
  cfg: AuthConfig,
  ctx: AuthApplyContext,
): Promise<AuthArtifacts> {
  const out: AuthArtifacts = { headers: [], query: [] }
  switch (cfg.type) {
    case 'none':
      break
    case 'apikey': {
      const key = cfg.apiKey.trim()
      const value = cfg.apiValue
      if (!key) break
      if (cfg.apiKeyAddTo === 'query') pushQuery(out, key, value)
      else pushHeader(out, key, value)
      break
    }
    case 'basic': {
      const value = basicValue(cfg.basicUsername, cfg.basicPassword)
      if (value) pushHeader(out, 'Authorization', value)
      break
    }
    case 'bearer': {
      const token = cfg.bearerToken.trim()
      if (token) pushHeader(out, 'Authorization', `Bearer ${token}`)
      break
    }
    case 'digest':
      await applyDigest(cfg, ctx, out)
      break
    case 'hawk':
      await applyHawk(cfg, ctx, out)
      break
    case 'jwt':
      await applyJwt(cfg, out)
      break
    case 'oauth1':
      await applyOauth1(cfg, ctx, out)
      break
    case 'oauth2': {
      const token = cfg.oauth2AccessToken.trim()
      if (!token) break
      if (cfg.oauth2AddTokenTo === 'query') {
        pushQuery(out, cfg.oauth2TokenParamName.trim() || 'access_token', token)
      } else {
        const prefix = (cfg.oauth2HeaderPrefix || 'Bearer').trim()
        pushHeader(out, 'Authorization', prefix ? `${prefix} ${token}` : token)
      }
      break
    }
    case 'edgegrid':
      await applyEdgeGrid(cfg, ctx, out)
      break
    case 'asap':
      await applyAsap(cfg, out)
      break
    case 'awsv4':
      await applyAwsSigV4(cfg, ctx, out)
      break
    case 'ntlm':
      if (cfg.ntlmUsername || cfg.ntlmPassword || cfg.ntlmDomain) applyNtlm(cfg, out)
      break
  }
  return out
}

/** Sync preview for simple auth types (auto-headers panel). */
export function previewAuthSync(cfg: AuthConfig): AuthArtifacts {
  const out: AuthArtifacts = { headers: [], query: [] }
  switch (cfg.type) {
    case 'apikey': {
      const key = cfg.apiKey.trim()
      if (!key) break
      if (cfg.apiKeyAddTo === 'query') pushQuery(out, key, cfg.apiValue)
      else pushHeader(out, key, cfg.apiValue)
      break
    }
    case 'basic': {
      const value = basicValue(cfg.basicUsername, cfg.basicPassword)
      if (value) pushHeader(out, 'Authorization', value)
      break
    }
    case 'bearer': {
      const token = cfg.bearerToken.trim()
      if (token) pushHeader(out, 'Authorization', `Bearer ${token}`)
      break
    }
    case 'oauth2': {
      const token = cfg.oauth2AccessToken.trim()
      if (!token) break
      if (cfg.oauth2AddTokenTo === 'query') {
        pushQuery(out, cfg.oauth2TokenParamName.trim() || 'access_token', token)
      } else {
        const prefix = (cfg.oauth2HeaderPrefix || 'Bearer').trim()
        pushHeader(out, 'Authorization', prefix ? `${prefix} ${token}` : token)
      }
      break
    }
    default:
      break
  }
  return out
}

/** Folder auth first; interface auth wins on duplicate header/query keys (same as folder headers). */
export function mergeAuthArtifacts(
  base: AuthArtifacts,
  override: AuthArtifacts,
): AuthArtifacts {
  const headers = [...base.headers]
  for (const h of override.headers) {
    const keyLc = h.key.toLowerCase()
    const idx = headers.findIndex((x) => x.key.toLowerCase() === keyLc)
    if (idx >= 0) headers[idx] = h
    else headers.push(h)
  }
  const query = [...base.query]
  for (const q of override.query) {
    const keyLc = q.key.toLowerCase()
    const idx = query.findIndex((x) => x.key.toLowerCase() === keyLc)
    if (idx >= 0) query[idx] = q
    else query.push(q)
  }
  return { headers, query }
}

export function appendQueryToUrl(url: string, query: Array<{ key: string; value: string }>): string {
  if (!query.length) return url
  const extras = query.map((q) => `${q.key}=${q.value}`).join('&')
  if (!url) return extras
  if (url.includes('?')) {
    if (url.endsWith('?') || url.endsWith('&')) return `${url}${extras}`
    return `${url}&${extras}`
  }
  return `${url}?${extras}`
}
