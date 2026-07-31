import http, { getData } from '@/utils/request'
import type {
  InstallPayload,
  InstallResult,
  InstallStatus,
  TestDatabaseResult,
  DatabaseConfig,
} from '@/types/install'

export function fetchInstallStatus() {
  return getData<InstallStatus>(http.get('/v1/install/status'))
}

export function testDatabaseConnection(database: DatabaseConfig) {
  return getData<TestDatabaseResult>(
    http.post('/v1/install/test-database', { database }),
  )
}

export function submitInstall(payload: InstallPayload) {
  return getData<InstallResult>(http.post('/v1/install', payload))
}
