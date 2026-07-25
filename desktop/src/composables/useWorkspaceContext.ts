import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { EnvironmentRow, ProjectRow, WorkspaceRow } from '../types'

const workspaces = ref<WorkspaceRow[]>([])
const projects = ref<ProjectRow[]>([])
const environments = ref<EnvironmentRow[]>([])
const workspaceId = ref<number | null>(null)
const projectId = ref<number | null>(null)
const environmentId = ref<number | null>(null)
const envVarsRevision = ref(0)
const ready = ref(false)

function bumpEnvVarsRevision() {
  envVarsRevision.value += 1
}

const currentWorkspace = computed(
  () => workspaces.value.find((w) => w.id === workspaceId.value) ?? null,
)

async function persistContext() {
  try {
    await invoke('set_active_context', {
      workspaceId: workspaceId.value,
      projectId: projectId.value,
      environmentId: environmentId.value,
    })
  } catch {
    // browser preview without Tauri
  }
}

async function loadWorkspaces() {
  try {
    workspaces.value = await invoke('list_local_workspaces')
  } catch {
    workspaces.value = []
  }
}

async function loadProjects() {
  if (!workspaceId.value) {
    projects.value = []
    return
  }
  try {
    projects.value = await invoke('list_local_projects', { workspaceId: workspaceId.value })
  } catch {
    projects.value = []
  }
}

async function loadEnvs() {
  if (!workspaceId.value || !projectId.value) {
    environments.value = []
    return
  }
  try {
    environments.value = await invoke('list_local_environments', {
      workspaceId: workspaceId.value,
      projectId: projectId.value,
    })
  } catch {
    environments.value = []
  }
}

async function selectWorkspace(id: number) {
  if (workspaceId.value === id) return
  workspaceId.value = id
  projectId.value = null
  environmentId.value = null
  await loadProjects()
  if (projects.value[0]) {
    projectId.value = projects.value[0].id
    await loadEnvs()
    const def = environments.value.find((x) => x.is_default) ?? environments.value[0]
    environmentId.value = def?.id ?? null
  } else {
    environments.value = []
  }
  await persistContext()
}

async function selectProject(id: number) {
  projectId.value = id
  await loadEnvs()
  const def = environments.value.find((x) => x.is_default) ?? environments.value[0]
  environmentId.value = def?.id ?? null
  await persistContext()
}

async function selectEnvironment(id: number | null) {
  environmentId.value = id
  await persistContext()
}

async function initWorkspaceContext() {
  await loadWorkspaces()
  let ctx: [number | null, number | null, number | null] = [null, null, null]
  try {
    ctx = await invoke('get_active_context')
  } catch {
    // browser preview
  }
  if (ctx[0] && workspaces.value.some((w) => w.id === ctx[0])) {
    workspaceId.value = ctx[0]
  } else if (workspaces.value[0]) {
    workspaceId.value = workspaces.value[0].id
  }

  if (workspaceId.value) {
    await loadProjects()
    if (ctx[1] && projects.value.some((p) => p.id === ctx[1])) {
      projectId.value = ctx[1]
    } else if (projects.value[0]) {
      projectId.value = projects.value[0].id
    }
  }

  if (workspaceId.value && projectId.value) {
    await loadEnvs()
    if (ctx[2] && environments.value.some((e) => e.id === ctx[2])) {
      environmentId.value = ctx[2]
    } else {
      const def = environments.value.find((e) => e.is_default) ?? environments.value[0]
      environmentId.value = def?.id ?? null
    }
  }

  await persistContext()
  ready.value = true
}

async function refreshAfterSync() {
  const prevWs = workspaceId.value
  const prevProject = projectId.value
  await loadWorkspaces()
  if (prevWs && workspaces.value.some((w) => w.id === prevWs)) {
    workspaceId.value = prevWs
  } else {
    workspaceId.value = workspaces.value[0]?.id ?? null
  }
  if (workspaceId.value) {
    await loadProjects()
    if (prevProject && projects.value.some((p) => p.id === prevProject)) {
      projectId.value = prevProject
    } else {
      projectId.value = projects.value[0]?.id ?? null
    }
    if (projectId.value) {
      await loadEnvs()
      const def = environments.value.find((e) => e.is_default) ?? environments.value[0]
      environmentId.value = def?.id ?? null
    }
  } else {
    projects.value = []
    environments.value = []
    projectId.value = null
    environmentId.value = null
  }
  await persistContext()
}

export function useWorkspaceContext() {
  return {
    workspaces,
    projects,
    environments,
    workspaceId,
    projectId,
    environmentId,
    envVarsRevision,
    currentWorkspace,
    ready,
    loadWorkspaces,
    loadProjects,
    loadEnvs,
    selectWorkspace,
    selectProject,
    selectEnvironment,
    initWorkspaceContext,
    refreshAfterSync,
    persistContext,
    bumpEnvVarsRevision,
  }
}
