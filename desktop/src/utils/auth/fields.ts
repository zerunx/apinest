import type { AuthConfig, AuthType } from './types'

export type AuthFieldKind = 'text' | 'password' | 'textarea' | 'select' | 'checkbox'

export type AuthFieldDef = {
  key: keyof AuthConfig
  labelKey: string
  kind: AuthFieldKind
  placeholderKey?: string
  options?: Array<{ value: string; labelKey: string }>
  rows?: number
}

const addToHeaderQuery: AuthFieldDef['options'] = [
  { value: 'header', labelKey: 'request.authAddToHeader' },
  { value: 'query', labelKey: 'request.authAddToQuery' },
]

export const AUTH_TYPE_LABEL_KEYS: Record<AuthType, string> = {
  none: 'request.authNone',
  apikey: 'request.authApiKey',
  basic: 'request.authBasic',
  bearer: 'request.authBearer',
  digest: 'request.authDigest',
  hawk: 'request.authHawk',
  jwt: 'request.authJwt',
  oauth1: 'request.authOauth1',
  oauth2: 'request.authOauth2',
  edgegrid: 'request.authEdgeGrid',
  asap: 'request.authAsap',
  awsv4: 'request.authAws',
  ntlm: 'request.authNtlm',
}

export const AUTH_TYPE_HINT_KEYS: Record<AuthType, string> = {
  none: 'request.authNoneHint',
  apikey: 'request.authApiKeyHint',
  basic: 'request.authBasicHint',
  bearer: 'request.authBearerHint',
  digest: 'request.authDigestHint',
  hawk: 'request.authHawkHint',
  jwt: 'request.authJwtHint',
  oauth1: 'request.authOauth1Hint',
  oauth2: 'request.authOauth2Hint',
  edgegrid: 'request.authEdgeGridHint',
  asap: 'request.authAsapHint',
  awsv4: 'request.authAwsHint',
  ntlm: 'request.authNtlmHint',
}

/** Types shown above the separator (like Postman). */
export const AUTH_TYPE_PRIMARY: AuthType[] = [
  'none',
  'apikey',
  'basic',
  'bearer',
  'digest',
  'hawk',
  'jwt',
  'oauth1',
  'oauth2',
]

export const AUTH_TYPE_SECONDARY: AuthType[] = ['edgegrid', 'asap', 'awsv4', 'ntlm']

export const AUTH_FIELDS: Partial<Record<AuthType, AuthFieldDef[]>> = {
  apikey: [
    { key: 'apiKey', labelKey: 'request.authKey', kind: 'text', placeholderKey: 'request.authKey' },
    {
      key: 'apiValue',
      labelKey: 'request.authValue',
      kind: 'text',
      placeholderKey: 'request.authValue',
    },
    {
      key: 'apiKeyAddTo',
      labelKey: 'request.authAddTo',
      kind: 'select',
      options: addToHeaderQuery,
    },
  ],
  bearer: [
    {
      key: 'bearerToken',
      labelKey: 'request.authToken',
      kind: 'text',
      placeholderKey: 'request.authToken',
    },
  ],
  basic: [
    {
      key: 'basicUsername',
      labelKey: 'request.authUsername',
      kind: 'text',
      placeholderKey: 'request.authUsername',
    },
    {
      key: 'basicPassword',
      labelKey: 'request.authPassword',
      kind: 'password',
      placeholderKey: 'request.authPassword',
    },
  ],
  digest: [
    {
      key: 'digestUsername',
      labelKey: 'request.authUsername',
      kind: 'text',
      placeholderKey: 'request.authUsername',
    },
    {
      key: 'digestPassword',
      labelKey: 'request.authPassword',
      kind: 'password',
      placeholderKey: 'request.authPassword',
    },
    { key: 'digestRealm', labelKey: 'request.authRealm', kind: 'text' },
    { key: 'digestNonce', labelKey: 'request.authNonce', kind: 'text' },
    {
      key: 'digestAlgorithm',
      labelKey: 'request.authAlgorithm',
      kind: 'select',
      options: [
        { value: 'MD5', labelKey: 'request.authAlgoMd5' },
        { value: 'MD5-sess', labelKey: 'request.authAlgoMd5Sess' },
        { value: 'SHA-256', labelKey: 'request.authAlgoSha256' },
        { value: 'SHA-256-sess', labelKey: 'request.authAlgoSha256Sess' },
      ],
    },
    {
      key: 'digestQop',
      labelKey: 'request.authQop',
      kind: 'select',
      options: [
        { value: '', labelKey: 'request.authQopEmpty' },
        { value: 'auth', labelKey: 'request.authQopAuth' },
        { value: 'auth-int', labelKey: 'request.authQopAuthInt' },
      ],
    },
    { key: 'digestNonceCount', labelKey: 'request.authNonceCount', kind: 'text' },
    { key: 'digestClientNonce', labelKey: 'request.authClientNonce', kind: 'text' },
    { key: 'digestOpaque', labelKey: 'request.authOpaque', kind: 'text' },
  ],
  hawk: [
    { key: 'hawkAuthId', labelKey: 'request.authHawkId', kind: 'text' },
    { key: 'hawkAuthKey', labelKey: 'request.authHawkKey', kind: 'password' },
    {
      key: 'hawkAlgorithm',
      labelKey: 'request.authAlgorithm',
      kind: 'select',
      options: [
        { value: 'sha256', labelKey: 'request.authAlgoSha256Lower' },
        { value: 'sha1', labelKey: 'request.authAlgoSha1Lower' },
      ],
    },
    { key: 'hawkUser', labelKey: 'request.authUsername', kind: 'text' },
    { key: 'hawkNonce', labelKey: 'request.authNonce', kind: 'text' },
    { key: 'hawkExt', labelKey: 'request.authExt', kind: 'text' },
    { key: 'hawkApp', labelKey: 'request.authApp', kind: 'text' },
    { key: 'hawkDlg', labelKey: 'request.authDlg', kind: 'text' },
    { key: 'hawkTimestamp', labelKey: 'request.authTimestamp', kind: 'text' },
    {
      key: 'hawkIncludePayloadHash',
      labelKey: 'request.authIncludePayloadHash',
      kind: 'checkbox',
    },
  ],
  jwt: [
    {
      key: 'jwtAlgorithm',
      labelKey: 'request.authAlgorithm',
      kind: 'select',
      options: [
        { value: 'HS256', labelKey: 'request.authAlgoHs256' },
        { value: 'HS384', labelKey: 'request.authAlgoHs384' },
        { value: 'HS512', labelKey: 'request.authAlgoHs512' },
      ],
    },
    { key: 'jwtSecret', labelKey: 'request.authSecret', kind: 'password' },
    {
      key: 'jwtSecretBase64',
      labelKey: 'request.authSecretBase64',
      kind: 'checkbox',
    },
    {
      key: 'jwtPayload',
      labelKey: 'request.authPayload',
      kind: 'textarea',
      rows: 6,
    },
    {
      key: 'jwtHeaders',
      labelKey: 'request.authJwtHeader',
      kind: 'textarea',
      rows: 3,
    },
    {
      key: 'jwtHeaderPrefix',
      labelKey: 'request.authHeaderPrefix',
      kind: 'text',
    },
  ],
  oauth1: [
    {
      key: 'oauth1SignatureMethod',
      labelKey: 'request.authSignatureMethod',
      kind: 'select',
      options: [
        { value: 'HMAC-SHA1', labelKey: 'request.authSigHmacSha1' },
        { value: 'HMAC-SHA256', labelKey: 'request.authSigHmacSha256' },
        { value: 'PLAINTEXT', labelKey: 'request.authSigPlaintext' },
      ],
    },
    { key: 'oauth1ConsumerKey', labelKey: 'request.authConsumerKey', kind: 'text' },
    {
      key: 'oauth1ConsumerSecret',
      labelKey: 'request.authConsumerSecret',
      kind: 'password',
    },
    { key: 'oauth1Token', labelKey: 'request.authAccessToken', kind: 'text' },
    {
      key: 'oauth1TokenSecret',
      labelKey: 'request.authTokenSecret',
      kind: 'password',
    },
    { key: 'oauth1Callback', labelKey: 'request.authCallbackUrl', kind: 'text' },
    { key: 'oauth1Verifier', labelKey: 'request.authVerifier', kind: 'text' },
    { key: 'oauth1Timestamp', labelKey: 'request.authTimestamp', kind: 'text' },
    { key: 'oauth1Nonce', labelKey: 'request.authNonce', kind: 'text' },
    { key: 'oauth1Version', labelKey: 'request.authVersion', kind: 'text' },
    { key: 'oauth1Realm', labelKey: 'request.authRealm', kind: 'text' },
    {
      key: 'oauth1AddParamsTo',
      labelKey: 'request.authAddTo',
      kind: 'select',
      options: [
        ...addToHeaderQuery!,
        { value: 'body', labelKey: 'request.authAddToBody' },
      ],
    },
  ],
  oauth2: [
    {
      key: 'oauth2AccessToken',
      labelKey: 'request.authAccessToken',
      kind: 'text',
      placeholderKey: 'request.authAccessToken',
    },
    {
      key: 'oauth2HeaderPrefix',
      labelKey: 'request.authHeaderPrefix',
      kind: 'text',
    },
    {
      key: 'oauth2AddTokenTo',
      labelKey: 'request.authAddTo',
      kind: 'select',
      options: addToHeaderQuery,
    },
    {
      key: 'oauth2TokenParamName',
      labelKey: 'request.authTokenParamName',
      kind: 'text',
    },
  ],
  edgegrid: [
    { key: 'edgeAccessToken', labelKey: 'request.authAccessToken', kind: 'text' },
    { key: 'edgeClientToken', labelKey: 'request.authClientToken', kind: 'text' },
    {
      key: 'edgeClientSecret',
      labelKey: 'request.authClientSecret',
      kind: 'password',
    },
    { key: 'edgeNonce', labelKey: 'request.authNonce', kind: 'text' },
    { key: 'edgeTimestamp', labelKey: 'request.authTimestamp', kind: 'text' },
    { key: 'edgeBaseUrl', labelKey: 'request.authBaseUrl', kind: 'text' },
    {
      key: 'edgeHeadersToSign',
      labelKey: 'request.authHeadersToSign',
      kind: 'text',
    },
  ],
  asap: [
    { key: 'asapIssuer', labelKey: 'request.authIssuer', kind: 'text' },
    { key: 'asapSubject', labelKey: 'request.authSubject', kind: 'text' },
    { key: 'asapAudience', labelKey: 'request.authAudience', kind: 'text' },
    { key: 'asapKeyId', labelKey: 'request.authKeyId', kind: 'text' },
    {
      key: 'asapPrivateKey',
      labelKey: 'request.authPrivateKey',
      kind: 'textarea',
      rows: 5,
    },
    { key: 'asapClaims', labelKey: 'request.authAdditionalClaims', kind: 'textarea', rows: 3 },
    { key: 'asapExpiresIn', labelKey: 'request.authExpiresIn', kind: 'text' },
  ],
  awsv4: [
    { key: 'awsAccessKey', labelKey: 'request.authAccessKey', kind: 'text' },
    { key: 'awsSecretKey', labelKey: 'request.authSecretKey', kind: 'password' },
    { key: 'awsRegion', labelKey: 'request.authAwsRegion', kind: 'text' },
    { key: 'awsService', labelKey: 'request.authAwsService', kind: 'text' },
    { key: 'awsSessionToken', labelKey: 'request.authSessionToken', kind: 'text' },
    {
      key: 'awsAddAuthDataTo',
      labelKey: 'request.authAddTo',
      kind: 'select',
      options: addToHeaderQuery,
    },
  ],
  ntlm: [
    {
      key: 'ntlmUsername',
      labelKey: 'request.authUsername',
      kind: 'text',
      placeholderKey: 'request.authUsername',
    },
    {
      key: 'ntlmPassword',
      labelKey: 'request.authPassword',
      kind: 'password',
      placeholderKey: 'request.authPassword',
    },
    { key: 'ntlmDomain', labelKey: 'request.authDomain', kind: 'text' },
    { key: 'ntlmWorkstation', labelKey: 'request.authWorkstation', kind: 'text' },
  ],
}
