<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowDown, Folder } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import {
  fetchShareContent,
  fetchShareInterfaceDetail,
  fetchSharePreview,
  type ShareContent,
  type SharedFolderItem,
  type ShareTreeNode,
  type SharedInterfaceItem,
} from '@/api/share'
import { useLocale } from '@/composables/useLocale'
import { useSharePageMeta } from '@/composables/useSharePageMeta'
import InterfaceRequestBodyRaw from '@/interface/InterfaceRequestBodyRaw.vue'
import type {
  InterfaceBodyField,
  InterfaceDetail,
  InterfaceParamItem,
  InterfaceRequestBody,
  InterfaceResponseExample,
  InterfaceResponseField,
  InterfaceResponseResult,
} from '@/types/workspace'

const route = useRoute()
const { t } = useLocale()
const { setSharePageMeta, clearSharePageMeta } = useSharePageMeta()

const shareCode = computed(() => String(route.query.shareCode || '').trim())

const loading = ref(true)
const unlocking = ref(false)
const loadingDetail = ref(false)
const errorMessage = ref('')
const password = ref('')
const unlocked = ref(false)

const previewName = ref('')
const projectName = ref('')
const content = ref<ShareContent | null>(null)
const selectedId = ref<number | null>(null)
const detail = ref<InterfaceDetail | null>(null)
const collapsedFolders = ref<Set<number>>(new Set())

interface FlatParamRow {
  name: string
  type: string
  required?: boolean
  description?: string
  example?: string
  depth: number
}

interface FlatTreeRow {
  node: ShareTreeNode
  depth: number
}

const treeNodes = computed<ShareTreeNode[]>(() => {
  const data = content.value
  if (!data) return []
  if (data.folders?.length || data.interfaces?.length) {
    return buildTreeFromFolders(data.folders || [], data.interfaces || [])
  }
  if (data.tree?.length) return data.tree
  return fallbackTreeFromInterfaces(data.interfaces || [])
})

const flatTreeRows = computed<FlatTreeRow[]>(() => flattenVisibleTree(treeNodes.value))

const hasTree = computed(() => flatTreeRows.value.length > 0)

const hasRequestHeaders = computed(() => Boolean(detail.value?.request_headers?.length))
const hasQueryParams = computed(() => Boolean(detail.value?.query_params?.length))
const hasRequestBody = computed(() => {
  const body = detail.value?.request_body
  if (!body) return false
  return Boolean(body.raw?.trim()) || Boolean(body.fields?.length)
})
const hasResponseHeaders = computed(() => Boolean(detail.value?.response_headers?.length))
const hasResponseResults = computed(() =>
  Boolean(detail.value?.response_results?.some((r) => r.fields?.length)),
)
const hasResponseExamples = computed(() =>
  Boolean(detail.value?.response_examples?.some((e) => e.raw?.trim())),
)

function buildTreeFromFolders(
  folders: SharedFolderItem[],
  interfaces: SharedInterfaceItem[],
): ShareTreeNode[] {
  const childrenMap = new Map<number, SharedFolderItem[]>()
  const folderIds = new Set<number>()

  for (const folder of folders) {
    folderIds.add(folder.id)
    const parentId = folder.parent_id || 0
    if (!childrenMap.has(parentId)) childrenMap.set(parentId, [])
    childrenMap.get(parentId)!.push(folder)
  }

  for (const list of childrenMap.values()) {
    list.sort((a, b) => a.id - b.id)
  }

  const apisByFolder = new Map<number, SharedInterfaceItem[]>()
  for (const item of interfaces) {
    const folderId = item.folder_id || 0
    if (!apisByFolder.has(folderId)) apisByFolder.set(folderId, [])
    apisByFolder.get(folderId)!.push(item)
  }

  function build(parentId: number): ShareTreeNode[] {
    const nodes: ShareTreeNode[] = []
    for (const folder of childrenMap.get(parentId) || []) {
      const children = build(folder.id)
      for (const api of apisByFolder.get(folder.id) || []) {
        children.push({
          id: api.id,
          name: api.name,
          type: 'api',
          method: api.method,
          url: api.url,
          status: api.status,
        })
      }
      nodes.push({
        id: folder.id,
        name: folder.name,
        type: 'folder',
        children,
      })
    }

    if (parentId === 0) {
      for (const api of apisByFolder.get(0) || []) {
        nodes.push({
          id: api.id,
          name: api.name,
          type: 'api',
          method: api.method,
          url: api.url,
          status: api.status,
        })
      }
      for (const [folderId, apis] of apisByFolder) {
        if (folderId === 0 || folderIds.has(folderId)) continue
        for (const api of apis) {
          nodes.push({
            id: api.id,
            name: api.name,
            type: 'api',
            method: api.method,
            url: api.url,
            status: api.status,
          })
        }
      }
    }

    return nodes
  }

  return build(0)
}

function fallbackTreeFromInterfaces(list: SharedInterfaceItem[]): ShareTreeNode[] {
  const order: number[] = []
  const map = new Map<number, ShareTreeNode>()
  for (const item of list) {
    const folderId = item.folder_id || 0
    if (folderId && !map.has(folderId)) {
      order.push(folderId)
      map.set(folderId, {
        id: folderId,
        name: item.folder_name?.trim() || t('share.public.ungrouped'),
        type: 'folder',
        children: [],
      })
    }
    const apiNode: ShareTreeNode = {
      id: item.id,
      name: item.name,
      type: 'api',
      method: item.method,
      url: item.url,
      status: item.status,
    }
    if (folderId && map.has(folderId)) {
      map.get(folderId)!.children!.push(apiNode)
    } else {
      order.push(-item.id)
      map.set(-item.id, apiNode)
    }
  }
  return order.map((id) => map.get(id)!).filter(Boolean)
}

function flattenVisibleTree(nodes: ShareTreeNode[], depth = 0): FlatTreeRow[] {
  const rows: FlatTreeRow[] = []
  for (const node of nodes) {
    rows.push({ node, depth })
    if (
      node.type === 'folder' &&
      !collapsedFolders.value.has(node.id) &&
      node.children?.length
    ) {
      rows.push(...flattenVisibleTree(node.children, depth + 1))
    }
  }
  return rows
}

function isFolderCollapsed(id: number) {
  return collapsedFolders.value.has(id)
}

function toggleFolder(id: number) {
  const next = new Set(collapsedFolders.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  collapsedFolders.value = next
}

function flattenParamRows(rows: InterfaceParamItem[] | undefined): FlatParamRow[] {
  return (rows || []).map((row) => ({
    name: row.name,
    type: row.type,
    required: row.required,
    description: row.description,
    example: row.example,
    depth: 0,
  }))
}

function flattenBodyFields(
  fields: InterfaceBodyField[] | undefined,
  depth = 0,
): FlatParamRow[] {
  const result: FlatParamRow[] = []
  for (const field of fields || []) {
    result.push({
      name: field.name,
      type: field.type,
      required: field.required,
      description: field.description,
      example: field.example,
      depth,
    })
    if (field.children?.length) {
      result.push(...flattenBodyFields(field.children, depth + 1))
    }
  }
  return result
}

function flattenResponseFields(
  fields: InterfaceResponseField[] | undefined,
  depth = 0,
): FlatParamRow[] {
  const result: FlatParamRow[] = []
  for (const field of fields || []) {
    result.push({
      name: field.name,
      type: field.type,
      required: field.required,
      description: field.description,
      example: field.example || field.mock,
      depth,
    })
    if (field.children?.length) {
      result.push(...flattenResponseFields(field.children, depth + 1))
    }
  }
  return result
}

function responseResultRows(result: InterfaceResponseResult): FlatParamRow[] {
  return flattenResponseFields(result.fields)
}

function requiredLabel(required?: boolean) {
  return required ? t('share.public.required') : t('share.public.optional')
}

function exampleRawBody(example: InterfaceResponseExample): InterfaceRequestBody {
  return {
    format: example.format || 'raw',
    data_type: example.content_type || example.data_type || 'JSON',
    raw: example.raw || '',
    fields: [],
  }
}

async function copyText(text: string) {
  const value = text?.trim()
  if (!value || value === '—') return
  try {
    await navigator.clipboard.writeText(value)
    ElMessage.success(t('share.public.copySuccess'))
  } catch {
    ElMessage.error(t('share.public.copyFailed'))
  }
}

async function loadPreview() {
  if (!shareCode.value) {
    errorMessage.value = t('share.public.invalid')
    loading.value = false
    return
  }

  loading.value = true
  errorMessage.value = ''
  try {
    const data = await fetchSharePreview(shareCode.value)
    if (!data.enabled) {
      errorMessage.value = t('share.public.closed')
      return
    }
    previewName.value = data.name
    projectName.value = data.project_name
    setSharePageMeta(data.project_name, data.name)
    if (!data.has_password) {
      await unlock()
    }
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : t('share.public.invalid')
  } finally {
    loading.value = false
  }
}

async function unlock() {
  if (!shareCode.value) return

  unlocking.value = true
  errorMessage.value = ''
  try {
    content.value = await fetchShareContent(shareCode.value, password.value)
    unlocked.value = true
    setSharePageMeta(content.value.project_name, content.value.name)
    const firstApi =
      findFirstApi(treeNodes.value) || content.value.interfaces[0]
    if (firstApi) {
      await selectInterface(firstApi)
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('share.public.unlockFailed'))
  } finally {
    unlocking.value = false
  }
}

function findFirstApi(nodes: ShareTreeNode[]): SharedInterfaceItem | null {
  for (const node of nodes) {
    if (node.type === 'api') {
      return {
        id: node.id,
        folder_id: 0,
        name: node.name,
        method: node.method || 'GET',
        url: node.url || '',
        status: node.status || 1,
      }
    }
    if (node.children?.length) {
      const found = findFirstApi(node.children)
      if (found) return found
    }
  }
  return null
}

async function selectInterface(item: SharedInterfaceItem | ShareTreeNode) {
  selectedId.value = item.id
  loadingDetail.value = true
  try {
    detail.value = await fetchShareInterfaceDetail(shareCode.value, item.id, password.value)
  } catch (error) {
    detail.value = null
    ElMessage.error(error instanceof Error ? error.message : t('share.public.detailFailed'))
  } finally {
    loadingDetail.value = false
  }
}

onMounted(() => {
  loadPreview()
})

onUnmounted(() => {
  clearSharePageMeta()
})
</script>

<template>
  <div class="share-view">
    <div v-if="loading" class="share-view__state">{{ t('share.public.loading') }}</div>

    <div v-else-if="errorMessage" class="share-view__state">
      <h2>{{ t('share.public.invalidTitle') }}</h2>
      <p>{{ errorMessage }}</p>
    </div>

    <div v-else-if="!unlocked" class="share-view__unlock">
      <h2>{{ t('share.public.unlockTitle') }}</h2>
      <p class="share-view__desc">
        {{ t('share.public.unlockDesc', { project: projectName, name: previewName }) }}
      </p>
      <el-input
        v-model="password"
        type="password"
        show-password
        :placeholder="t('share.public.passwordPlaceholder')"
        @keyup.enter="unlock"
      />
      <button
        type="button"
        class="share-view__btn"
        :disabled="unlocking"
        @click="unlock"
      >
        {{ unlocking ? t('share.public.unlocking') : t('share.public.unlock') }}
      </button>
    </div>

    <div v-else class="share-view__workspace">
      <div class="share-view__main">
        <aside class="share-view__sidebar">
          <div v-if="hasTree" class="share-view__tree">
            <template v-for="{ node, depth } in flatTreeRows" :key="`${node.type}-${node.id}`">
              <button
                v-if="node.type === 'folder'"
                type="button"
                class="share-view__folder-head"
                :style="{ paddingLeft: `${8 + depth * 16}px` }"
                @click="toggleFolder(node.id)"
              >
                <el-icon
                  :size="12"
                  class="share-view__folder-expand"
                  :class="{ 'is-open': !isFolderCollapsed(node.id) }"
                >
                  <ArrowDown />
                </el-icon>
                <el-icon :size="14" class="share-view__folder-icon"><Folder /></el-icon>
                <span class="share-view__folder-name">{{ node.name }}</span>
              </button>
              <button
                v-else
                type="button"
                class="share-view__api"
                :class="{ 'is-active': selectedId === node.id }"
                :style="{ paddingLeft: `${10 + depth * 16}px` }"
                @click="selectInterface(node)"
              >
                <span class="share-view__method-wrap">
                  <span
                    class="http-method-badge"
                    :class="`http-method-badge--${String(node.method || 'get').toLowerCase()}`"
                  >
                    {{ node.method }}
                  </span>
                </span>
                <span class="share-view__api-name">{{ node.name }}</span>
              </button>
            </template>
          </div>

          <div v-else class="share-view__empty">
            {{ t('share.public.noInterfaces') }}
          </div>
        </aside>

        <section v-loading="loadingDetail" class="share-view__detail">
          <template v-if="detail">
            <div class="share-view__detail-head">
              <span
                class="http-method-badge"
                :class="`http-method-badge--${detail.method.toLowerCase()}`"
              >
                {{ detail.method }}
              </span>
              <h3>{{ detail.name }}</h3>
            </div>
            <button
              v-if="detail.url"
              type="button"
              class="share-view__url share-view__copyable"
              :title="t('share.public.copyHint')"
              @click="copyText(detail.url)"
            >
              {{ detail.url }}
            </button>
            <p v-else class="share-view__url">—</p>

            <div v-if="hasRequestHeaders" class="share-view__block">
              <h4>{{ t('share.public.requestHeaders') }}</h4>
              <div class="share-view__table">
                <div class="share-view__row share-view__row--head">
                  <span>{{ t('workspace.interfaceForm.paramName') }}</span>
                  <span>{{ t('workspace.interfaceForm.paramType') }}</span>
                  <span>{{ t('share.public.required') }}</span>
                  <span>{{ t('workspace.interfaceForm.example') }}</span>
                </div>
                <div
                  v-for="(row, index) in flattenParamRows(detail.request_headers)"
                  :key="`h-${index}`"
                  class="share-view__row"
                >
                  <span
                    class="share-view__copyable share-view__copyable--cell"
                    :title="t('share.public.copyHint')"
                    @click="copyText(row.name)"
                  >{{ row.name }}</span>
                  <span>{{ row.type }}</span>
                  <span :class="{ 'is-required': row.required }">{{ requiredLabel(row.required) }}</span>
                  <span
                    class="share-view__copyable share-view__copyable--cell"
                    :title="t('share.public.copyHint')"
                    @click="copyText(row.example || '')"
                  >{{ row.example || '—' }}</span>
                </div>
              </div>
            </div>

            <div v-if="hasQueryParams" class="share-view__block">
              <h4>{{ t('share.public.queryParams') }}</h4>
              <div class="share-view__table">
                <div class="share-view__row share-view__row--head">
                  <span>{{ t('workspace.interfaceForm.paramName') }}</span>
                  <span>{{ t('workspace.interfaceForm.paramType') }}</span>
                  <span>{{ t('share.public.required') }}</span>
                  <span>{{ t('workspace.interfaceForm.example') }}</span>
                </div>
                <div
                  v-for="(row, index) in flattenParamRows(detail.query_params)"
                  :key="`q-${index}`"
                  class="share-view__row"
                >
                  <span
                    class="share-view__copyable share-view__copyable--cell"
                    :title="t('share.public.copyHint')"
                    @click="copyText(row.name)"
                  >{{ row.name }}</span>
                  <span>{{ row.type }}</span>
                  <span :class="{ 'is-required': row.required }">{{ requiredLabel(row.required) }}</span>
                  <span
                    class="share-view__copyable share-view__copyable--cell"
                    :title="t('share.public.copyHint')"
                    @click="copyText(row.example || '')"
                  >{{ row.example || '—' }}</span>
                </div>
              </div>
            </div>

            <div v-if="hasRequestBody" class="share-view__block">
              <h4>{{ t('share.public.requestBody') }}</h4>
              <pre
                v-if="detail.request_body?.raw"
                class="share-view__raw"
              >{{ detail.request_body.raw }}</pre>
              <div
                v-else-if="detail.request_body?.fields?.length"
                class="share-view__table"
              >
                <div class="share-view__row share-view__row--head">
                  <span>{{ t('workspace.interfaceForm.paramName') }}</span>
                  <span>{{ t('workspace.interfaceForm.paramType') }}</span>
                  <span>{{ t('share.public.required') }}</span>
                  <span>{{ t('workspace.interfaceForm.example') }}</span>
                </div>
                <div
                  v-for="(row, index) in flattenBodyFields(detail.request_body.fields)"
                  :key="`b-${index}`"
                  class="share-view__row"
                >
                  <span
                    class="share-view__copyable share-view__copyable--cell"
                    :style="{ paddingLeft: `${8 + row.depth * 14}px` }"
                    :title="t('share.public.copyHint')"
                    @click="copyText(row.name)"
                  >{{ row.name }}</span>
                  <span>{{ row.type }}</span>
                  <span :class="{ 'is-required': row.required }">{{ requiredLabel(row.required) }}</span>
                  <span
                    class="share-view__copyable share-view__copyable--cell"
                    :title="t('share.public.copyHint')"
                    @click="copyText(row.example || '')"
                  >{{ row.example || '—' }}</span>
                </div>
              </div>
            </div>

            <div v-if="hasResponseHeaders" class="share-view__block">
              <h4>{{ t('share.public.responseHeaders') }}</h4>
              <div class="share-view__table">
                <div class="share-view__row share-view__row--head share-view__row--3">
                  <span>{{ t('workspace.interfaceForm.paramName') }}</span>
                  <span>{{ t('workspace.interfaceForm.paramType') }}</span>
                  <span>{{ t('workspace.interfaceForm.example') }}</span>
                </div>
                <div
                  v-for="(row, index) in flattenParamRows(detail.response_headers)"
                  :key="`rh-${index}`"
                  class="share-view__row share-view__row--3"
                >
                  <span
                    class="share-view__copyable share-view__copyable--cell"
                    :title="t('share.public.copyHint')"
                    @click="copyText(row.name)"
                  >{{ row.name }}</span>
                  <span>{{ row.type }}</span>
                  <span
                    class="share-view__copyable share-view__copyable--cell"
                    :title="t('share.public.copyHint')"
                    @click="copyText(row.example || '')"
                  >{{ row.example || '—' }}</span>
                </div>
              </div>
            </div>

            <template v-if="hasResponseResults">
              <div
                v-for="(result, rIndex) in detail.response_results.filter((r) => r.fields?.length)"
                :key="`rr-${rIndex}`"
                class="share-view__block"
              >
                <h4>
                  {{ t('share.public.responseResult') }}
                  <span v-if="result.name" class="share-view__block-meta">
                    {{ result.name }}
                    <template v-if="result.status_code"> · {{ result.status_code }}</template>
                  </span>
                </h4>
                <div class="share-view__table">
                  <div class="share-view__row share-view__row--head">
                    <span>{{ t('workspace.interfaceForm.paramName') }}</span>
                    <span>{{ t('workspace.interfaceForm.paramType') }}</span>
                    <span>{{ t('share.public.required') }}</span>
                    <span>{{ t('workspace.interfaceForm.example') }}</span>
                  </div>
                  <div
                    v-for="(row, index) in responseResultRows(result)"
                    :key="`rr-${rIndex}-${index}`"
                    class="share-view__row"
                  >
                    <span
                      class="share-view__copyable share-view__copyable--cell"
                      :style="{ paddingLeft: `${8 + row.depth * 14}px` }"
                      :title="t('share.public.copyHint')"
                      @click="copyText(row.name)"
                    >{{ row.name }}</span>
                    <span>{{ row.type }}</span>
                    <span :class="{ 'is-required': row.required }">{{ requiredLabel(row.required) }}</span>
                    <span
                      class="share-view__copyable share-view__copyable--cell"
                      :title="t('share.public.copyHint')"
                      @click="copyText(row.example || '')"
                    >{{ row.example || '—' }}</span>
                  </div>
                </div>
              </div>
            </template>

            <template v-if="hasResponseExamples">
              <div
                v-for="(example, eIndex) in detail.response_examples.filter((e) => e.raw?.trim())"
                :key="`re-${eIndex}`"
                class="share-view__block"
              >
                <h4>
                  {{ t('share.public.responseExample') }}
                  <span v-if="example.name" class="share-view__block-meta">
                    {{ example.name }}
                    <template v-if="example.status_code"> · {{ example.status_code }}</template>
                  </span>
                </h4>
                <div class="share-view__example-raw">
                  <InterfaceRequestBodyRaw
                    :model-value="exampleRawBody(example)"
                    readonly
                    hide-content-type
                  />
                </div>
              </div>
            </template>
          </template>
          <div v-else class="share-view__empty">{{ t('share.public.selectInterface') }}</div>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.share-view {
  color: var(--color-text);
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: stretch;
  min-height: 0;
  overflow: hidden;
}

.share-view__state,
.share-view__unlock {
  max-width: 380px;
  align-self: center;
}

.share-view__state h2,
.share-view__unlock h2 {
  margin: 0 0 8px;
  font-size: 28px;
  font-weight: 700;
  color: var(--color-text);
}

.share-view__desc {
  margin: 0 0 20px;
  color: var(--color-text-secondary);
  font-size: 14px;
  line-height: 1.6;
}

.share-view__btn {
  width: 100%;
  height: 42px;
  margin-top: 16px;
  border: none;
  border-radius: 10px;
  background: var(--color-text);
  color: var(--color-bg);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}

.share-view__btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.share-view__workspace {
  width: min(1360px, 96vw);
  height: 100%;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.share-view__main {
  display: grid;
  grid-template-columns: 320px minmax(0, 1fr);
  gap: 16px;
  flex: 1;
  min-width: 0;
  min-height: 0;
  height: 100%;
}

.share-view__sidebar,
.share-view__detail {
  border: 1px solid var(--color-border);
  border-radius: 12px;
  background: var(--color-surface);
  box-shadow: var(--shadow-card);
  min-height: 0;
}

.share-view__sidebar {
  overflow: auto;
  padding: 10px;
}

.share-view__folder-head {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 6px;
  padding-top: 7px;
  padding-right: 8px;
  padding-bottom: 7px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text);
  cursor: pointer;
  text-align: left;
  box-sizing: border-box;
}

.share-view__folder-head:hover {
  background: var(--color-hover);
}

.share-view__folder-expand {
  flex-shrink: 0;
  color: var(--color-text-secondary);
  transition: transform 0.15s ease;
  transform: rotate(-90deg);
}

.share-view__folder-expand.is-open {
  transform: rotate(0deg);
}

.share-view__folder-icon {
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.share-view__folder-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
  font-weight: 600;
}

.share-view__api {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
  padding-top: 8px;
  padding-right: 10px;
  padding-bottom: 8px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text);
  text-align: left;
  cursor: pointer;
  box-sizing: border-box;
}

.share-view__api:hover,
.share-view__api.is-active {
  background: var(--color-hover);
}

.share-view__method-wrap {
  width: 52px;
  min-width: 52px;
  flex-shrink: 0;
  display: inline-flex;
  justify-content: center;
}

.share-view__api-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
}

.share-view__detail {
  padding: 20px;
  overflow: auto;
}

.share-view__detail-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.share-view__detail-head h3 {
  margin: 0;
  font-size: 22px;
  color: var(--color-text);
}

.share-view__url {
  display: inline-block;
  margin: 0 0 22px;
  padding: 2px 4px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 14px;
  word-break: break-all;
  text-align: left;
  font-family: 'Consolas', 'Monaco', monospace;
}

.share-view__copyable {
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.15s ease;
}

.share-view__copyable--cell {
  display: block;
  width: 100%;
  min-width: 0;
  padding: 8px 10px;
  margin: -8px -10px;
  box-sizing: border-box;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.share-view__copyable:hover {
  color: inherit;
  background: var(--color-workspace-item-hover);
}

.share-view__url.share-view__copyable:hover {
  color: var(--color-text);
}

.share-view__example-raw {
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--color-workspace-inset, var(--color-bg));
}

.share-view__example-raw :deep(.interface-body-raw) {
  gap: 0;
}

.share-view__example-raw :deep(.interface-body-raw__toolbar) {
  padding: 8px 10px;
  border-bottom: 1px solid var(--color-border);
  background: transparent;
}

.share-view__example-raw :deep(.interface-body-raw__editor) {
  min-height: 160px;
  max-height: 360px;
}

.share-view__block {
  margin-bottom: 22px;
}

.share-view__block h4 {
  margin: 0 0 10px;
  font-size: 15px;
  color: var(--color-text);
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: wrap;
}

.share-view__block-meta {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.share-view__table {
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;
}

.share-view__row {
  display: grid;
  grid-template-columns: 1.3fr 0.7fr 0.6fr 1.4fr;
  gap: 8px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--color-border);
  font-size: 14px;
  color: var(--color-text);
  align-items: center;
}

.share-view__row--3 {
  grid-template-columns: 1.2fr 0.8fr 1.4fr;
}

.share-view__row--head {
  background: var(--color-workspace-inset, var(--color-bg));
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.share-view__row:last-child {
  border-bottom: none;
}

.share-view__row .is-required {
  color: #ef4444;
  font-weight: 600;
}

.share-view__raw {
  margin: 0;
  padding: 14px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-workspace-inset, var(--color-bg));
  color: var(--color-text);
  font-size: 14px;
  line-height: 1.55;
  overflow: auto;
  white-space: pre-wrap;
}

.share-view__empty {
  color: var(--color-text-secondary);
  font-size: 14px;
  padding: 8px 4px;
}

@media (max-width: 960px) {
  .share-view__main {
    grid-template-columns: 1fr;
    height: auto;
  }

  .share-view__workspace {
    width: 100%;
    overflow: auto;
  }

  .share-view__sidebar {
    max-height: 320px;
  }
}
</style>
