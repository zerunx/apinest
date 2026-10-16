export const AUTH_TYPES = [
  'none',
  'apikey',
  'basic',
  'bearer',
  'digest',
  'hawk',
  'jwt',
  'oauth1',
  'oauth2',
  'edgegrid',
  'asap',
  'awsv4',
  'ntlm',
] as const

export type AuthType = (typeof AUTH_TYPES)[number]

export type AuthAddTo = 'header' | 'query' | 'body'

export type AuthConfig = {
  type: AuthType
  /** API Key */
  apiKey: string
  apiValue: string
  apiKeyAddTo: 'header' | 'query'
  /** Bearer */
  bearerToken: string
  /** Basic */
  basicUsername: string
  basicPassword: string
  /** Digest */
  digestUsername: string
  digestPassword: string
  digestRealm: string
  digestNonce: string
  digestAlgorithm: 'MD5' | 'MD5-sess' | 'SHA-256' | 'SHA-256-sess' | 'SHA-512-256' | 'SHA-512-256-sess'
  digestQop: '' | 'auth' | 'auth-int'
  digestNonceCount: string
  digestClientNonce: string
  digestOpaque: string
  digestDisableRetry: boolean
  /** Hawk */
  hawkAuthId: string
  hawkAuthKey: string
  hawkAlgorithm: 'sha256' | 'sha1'
  hawkUser: string
  hawkNonce: string
  hawkExt: string
  hawkApp: string
  hawkDlg: string
  hawkTimestamp: string
  hawkIncludePayloadHash: boolean
  /** JWT Bearer */
  jwtAlgorithm: 'HS256' | 'HS384' | 'HS512'
  jwtSecret: string
  jwtSecretBase64: boolean
  jwtPayload: string
  jwtHeaders: string
  jwtAddTokenTo: 'header'
  jwtHeaderPrefix: string
  jwtQueryParamKey: string
  /** OAuth 1.0 */
  oauth1SignatureMethod: 'HMAC-SHA1' | 'HMAC-SHA256' | 'PLAINTEXT'
  oauth1ConsumerKey: string
  oauth1ConsumerSecret: string
  oauth1Token: string
  oauth1TokenSecret: string
  oauth1PrivateKey: string
  oauth1Callback: string
  oauth1Verifier: string
  oauth1Timestamp: string
  oauth1Nonce: string
  oauth1Version: string
  oauth1Realm: string
  oauth1AddParamsTo: 'header' | 'query' | 'body'
  oauth1IncludeBodyHash: boolean
  /** OAuth 2.0 */
  oauth2AccessToken: string
  oauth2HeaderPrefix: string
  oauth2AddTokenTo: 'header' | 'query'
  oauth2TokenParamName: string
  /** Akamai EdgeGrid */
  edgeAccessToken: string
  edgeClientToken: string
  edgeClientSecret: string
  edgeNonce: string
  edgeTimestamp: string
  edgeBaseUrl: string
  edgeHeadersToSign: string
  /** ASAP */
  asapIssuer: string
  asapSubject: string
  asapAudience: string
  asapKeyId: string
  asapPrivateKey: string
  asapClaims: string
  asapExpiresIn: string
  /** AWS Signature */
  awsAccessKey: string
  awsSecretKey: string
  awsRegion: string
  awsService: string
  awsSessionToken: string
  awsAddAuthDataTo: 'header' | 'query'
  /** NTLM */
  ntlmUsername: string
  ntlmPassword: string
  ntlmDomain: string
  ntlmWorkstation: string
}

export function emptyAuthConfig(type: AuthType = 'none'): AuthConfig {
  return {
    type,
    apiKey: '',
    apiValue: '',
    apiKeyAddTo: 'header',
    bearerToken: '',
    basicUsername: '',
    basicPassword: '',
    digestUsername: '',
    digestPassword: '',
    digestRealm: '',
    digestNonce: '',
    digestAlgorithm: 'MD5',
    digestQop: 'auth',
    digestNonceCount: '00000001',
    digestClientNonce: '',
    digestOpaque: '',
    digestDisableRetry: false,
    hawkAuthId: '',
    hawkAuthKey: '',
    hawkAlgorithm: 'sha256',
    hawkUser: '',
    hawkNonce: '',
    hawkExt: '',
    hawkApp: '',
    hawkDlg: '',
    hawkTimestamp: '',
    hawkIncludePayloadHash: false,
    jwtAlgorithm: 'HS256',
    jwtSecret: '',
    jwtSecretBase64: false,
    jwtPayload: '{\n  "sub": "1234567890",\n  "name": "John Doe",\n  "iat": 1516239022\n}',
    jwtHeaders: '',
    jwtAddTokenTo: 'header',
    jwtHeaderPrefix: 'Bearer',
    jwtQueryParamKey: 'token',
    oauth1SignatureMethod: 'HMAC-SHA1',
    oauth1ConsumerKey: '',
    oauth1ConsumerSecret: '',
    oauth1Token: '',
    oauth1TokenSecret: '',
    oauth1PrivateKey: '',
    oauth1Callback: '',
    oauth1Verifier: '',
    oauth1Timestamp: '',
    oauth1Nonce: '',
    oauth1Version: '1.0',
    oauth1Realm: '',
    oauth1AddParamsTo: 'header',
    oauth1IncludeBodyHash: false,
    oauth2AccessToken: '',
    oauth2HeaderPrefix: 'Bearer',
    oauth2AddTokenTo: 'header',
    oauth2TokenParamName: 'access_token',
    edgeAccessToken: '',
    edgeClientToken: '',
    edgeClientSecret: '',
    edgeNonce: '',
    edgeTimestamp: '',
    edgeBaseUrl: '',
    edgeHeadersToSign: '',
    asapIssuer: '',
    asapSubject: '',
    asapAudience: '',
    asapKeyId: '',
    asapPrivateKey: '',
    asapClaims: '',
    asapExpiresIn: '3600',
    awsAccessKey: '',
    awsSecretKey: '',
    awsRegion: '',
    awsService: '',
    awsSessionToken: '',
    awsAddAuthDataTo: 'header',
    ntlmUsername: '',
    ntlmPassword: '',
    ntlmDomain: '',
    ntlmWorkstation: '',
  }
}

export type AuthPersisted = {
  type: AuthType
  bearer_token?: string
  basic_username?: string
  basic_password?: string
  config?: Partial<AuthConfig>
}

export function authConfigToPersisted(cfg: AuthConfig): AuthPersisted {
  const { type, bearerToken, basicUsername, basicPassword, ...rest } = cfg
  return {
    type,
    bearer_token: bearerToken,
    basic_username: basicUsername,
    basic_password: basicPassword,
    config: { ...rest, type, bearerToken, basicUsername, basicPassword },
  }
}

export function authConfigFromPersisted(raw: Record<string, unknown> | null | undefined): AuthConfig {
  const base = emptyAuthConfig()
  if (!raw) return base
  const typeRaw = String(raw.type ?? 'none')
  const type = (AUTH_TYPES as readonly string[]).includes(typeRaw)
    ? (typeRaw as AuthType)
    : 'none'
  const nested = (raw.config && typeof raw.config === 'object' ? raw.config : {}) as Partial<AuthConfig>
  const merged: AuthConfig = {
    ...base,
    ...nested,
    type,
    bearerToken: String(nested.bearerToken ?? raw.bearer_token ?? ''),
    basicUsername: String(nested.basicUsername ?? raw.basic_username ?? ''),
    basicPassword: String(nested.basicPassword ?? raw.basic_password ?? ''),
  }
  return merged
}

export type AuthArtifacts = {
  headers: Array<{ key: string; value: string }>
  query: Array<{ key: string; value: string }>
}

export type AuthApplyContext = {
  method: string
  url: string
  body: string
  contentType?: string
}
