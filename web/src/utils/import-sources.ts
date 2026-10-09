export type ImportSource = {
  id: string
  name: string
  enabled: boolean
  icon?: string
  letter?: string
}

export const IMPORT_SOURCES: ImportSource[] = [
  { id: 'apinest', name: 'ApiNest', enabled: true, icon: '/nest.png' },
  { id: 'postman', name: 'Postman', enabled: true, icon: '/import-icons/postman.svg' },
  { id: 'openapi', name: 'OpenAPI', enabled: true, icon: '/import-icons/openapi.png' },
  { id: 'apifox', name: 'Apifox', enabled: true, icon: '/import-icons/apifox.png' },
  { id: 'yapi', name: 'YApi', enabled: true, icon: '/import-icons/yapi.png' },
  { id: 'eolink', name: 'Eolink', enabled: true, icon: '/import-icons/eolink.png' },
  { id: 'apipost', name: 'ApiPost', enabled: true, icon: '/import-icons/apipost.png' },
]
