export type ImportSource = {
  id: string
  name: string
  enabled: boolean
  icon?: string
  letter?: string
}

export const IMPORT_SOURCES: ImportSource[] = [
  { id: 'apinest', name: 'ApiNest', enabled: true, icon: '/nest.png' },
  { id: 'postman', name: 'Postman', enabled: false, letter: 'P' },
  { id: 'openapi', name: 'OpenAPI', enabled: false, letter: 'O' },
  { id: 'apifox', name: 'Apifox', enabled: false, letter: 'A' },
  { id: 'yapi', name: 'YApi', enabled: false, letter: 'Y' },
  { id: 'eolink', name: 'Eolink', enabled: false, letter: 'E' },
  { id: 'har', name: 'HAR', enabled: false, letter: 'H' },
  { id: 'apipost', name: 'ApiPost', enabled: false, letter: 'A' },
]
