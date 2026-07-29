export const PANEL_MIN_WIDTH = 230
export const PANEL_MAX_WIDTH = 450

const STORAGE_KEY = 'apinest_workspace_layout'

export interface StoredWorkspaceTab {
  id: string
  kind: string
  module: string
  label: string
  method?: string
  folderId?: string
  apiId?: string
  environmentId?: number
  closable?: boolean
}

export interface WorkspaceLayoutState {
  workspaceId: number | null
  projectId: number | null
  panelWidth: number
  activeModule?: string
  activeTabId?: string | null
  selectedFolderId?: string | null
  selectedApiId?: string | null
  workspaceTabs?: StoredWorkspaceTab[]
}

const defaultState: WorkspaceLayoutState = {
  workspaceId: null,
  projectId: null,
  panelWidth: PANEL_MIN_WIDTH,
}

function parseStoredId(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value
  }

  if (typeof value === 'string' && value.trim() !== '') {
    const parsed = Number.parseInt(value, 10)
    if (!Number.isNaN(parsed)) {
      return parsed
    }
  }

  return null
}

export function clampPanelWidth(width: number) {
  return Math.min(PANEL_MAX_WIDTH, Math.max(PANEL_MIN_WIDTH, width))
}

export function readWorkspaceLayout(): WorkspaceLayoutState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) {
      return { ...defaultState }
    }

    const parsed = JSON.parse(raw) as Partial<WorkspaceLayoutState>
    return {
      workspaceId: parseStoredId(parsed.workspaceId),
      projectId: parseStoredId(parsed.projectId),
      panelWidth: clampPanelWidth(parseStoredId(parsed.panelWidth) ?? PANEL_MIN_WIDTH),
      activeModule: typeof parsed.activeModule === 'string' ? parsed.activeModule : undefined,
      activeTabId: typeof parsed.activeTabId === 'string' ? parsed.activeTabId : null,
      selectedFolderId:
        typeof parsed.selectedFolderId === 'string' ? parsed.selectedFolderId : null,
      selectedApiId: typeof parsed.selectedApiId === 'string' ? parsed.selectedApiId : null,
      workspaceTabs: Array.isArray(parsed.workspaceTabs) ? parsed.workspaceTabs : undefined,
    }
  } catch {
    return { ...defaultState }
  }
}

export function writeWorkspaceLayout(partial: Partial<WorkspaceLayoutState>) {
  const current = readWorkspaceLayout()
  const next: WorkspaceLayoutState = {
    workspaceId: partial.workspaceId !== undefined ? partial.workspaceId : current.workspaceId,
    projectId: partial.projectId !== undefined ? partial.projectId : current.projectId,
    panelWidth:
      partial.panelWidth !== undefined
        ? clampPanelWidth(partial.panelWidth)
        : current.panelWidth,
    activeModule:
      partial.activeModule !== undefined ? partial.activeModule : current.activeModule,
    activeTabId: partial.activeTabId !== undefined ? partial.activeTabId : current.activeTabId,
    selectedFolderId:
      partial.selectedFolderId !== undefined ? partial.selectedFolderId : current.selectedFolderId,
    selectedApiId:
      partial.selectedApiId !== undefined ? partial.selectedApiId : current.selectedApiId,
    workspaceTabs:
      partial.workspaceTabs !== undefined ? partial.workspaceTabs : current.workspaceTabs,
  }

  localStorage.setItem(STORAGE_KEY, JSON.stringify(next))
}
