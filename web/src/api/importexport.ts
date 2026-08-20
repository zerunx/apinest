import http, { getData } from '@/utils/request'

export type ApinestExportDocument = {
  format: string
  version: number
  exported_at?: string
  scope?: string
  project?: { name?: string }
  tree?: unknown[]
}

export type ImportResult = {
  folders: number
  apis: number
}

export function exportApis(workspaceId: number, projectId: number, folderId?: number) {
  return getData<ApinestExportDocument>(
    http.get('/v1/import/export', {
      params: {
        workspace_id: workspaceId,
        project_id: projectId,
        ...(folderId ? { folder_id: folderId } : {}),
      },
    }),
  )
}

export function importApis(
  workspaceId: number,
  projectId: number,
  source: string,
  document: unknown,
) {
  return getData<ImportResult>(
    http.post(
      '/v1/import/import',
      {
        workspace_id: workspaceId,
        project_id: projectId,
        source,
        document,
      },
      { timeout: 120000 },
    ),
  )
}
