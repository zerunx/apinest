import type { InterfaceResponseField } from '@/types/workspace'
import { reorderList } from '@/composables/useTableRowDrag'

export interface FieldTreeNode {
  id?: number
  parent_id: number
  name: string
  type: string
  required: boolean
  description: string
  example: string
  mock?: string
  children?: FieldTreeNode[]
}

export interface FlatFieldRow {
  node: FieldTreeNode
  depth: number
  path: number[]
}

export function emptyFieldNode(parentId = 0): FieldTreeNode {
  return {
    parent_id: parentId,
    name: '',
    type: 'string',
    required: true,
    description: '',
    example: '',
    mock: '',
    children: [],
  }
}

export function hasFieldContent(node: FieldTreeNode): boolean {
  return !!(
    node.name.trim() ||
    node.description.trim() ||
    node.example.trim()
  )
}

function deepCloneTree(nodes: FieldTreeNode[]): FieldTreeNode[] {
  return nodes.map((node) => ({
    ...node,
    children: node.children ? deepCloneTree(node.children) : [],
  }))
}

export function getNodeAtPath(root: FieldTreeNode[], path: number[]): FieldTreeNode | null {
  let list = root
  let current: FieldTreeNode | null = null
  for (const idx of path) {
    current = list[idx] ?? null
    if (!current) return null
    list = current.children ?? []
  }
  return current
}

function resolveParentArray(root: FieldTreeNode[], path: number[]): FieldTreeNode[] | null {
  if (!path.length) return null
  if (path.length === 1) return root
  const parent = getNodeAtPath(root, path.slice(0, -1))
  if (!parent) return null
  if (!parent.children) parent.children = []
  return parent.children
}

export function ensureTrailingEmptyRoot(nodes: FieldTreeNode[]): FieldTreeNode[] {
  const tree = deepCloneTree(nodes)
  if (!tree.length) return [emptyFieldNode()]
  const last = tree[tree.length - 1]
  if (hasFieldContent(last)) {
    tree.push(emptyFieldNode())
  }
  return tree
}

export function flattenFieldTree(nodes: FieldTreeNode[], depth = 0, pathPrefix: number[] = []): FlatFieldRow[] {
  const result: FlatFieldRow[] = []
  for (let i = 0; i < nodes.length; i++) {
    const path = [...pathPrefix, i]
    const node = nodes[i]
    result.push({ node, depth, path })
    if (node.children?.length) {
      result.push(...flattenFieldTree(node.children, depth + 1, path))
    }
  }
  return result
}

export function fieldTreeFromApi(nodes: FieldTreeNode[]): FieldTreeNode[] {
  const cloned = deepCloneTree(nodes)
  return ensureTrailingEmptyRoot(cloned.length ? cloned : [])
}

export function updateNodeAtPath(
  root: FieldTreeNode[],
  path: number[],
  patch: Partial<FieldTreeNode>,
): FieldTreeNode[] {
  const tree = deepCloneTree(root)
  const list = resolveParentArray(tree, path)
  const idx = path[path.length - 1]
  if (!list || idx === undefined || !list[idx]) return tree
  list[idx] = { ...list[idx], ...patch }
  return ensureTrailingEmptyRoot(tree)
}

export function removeNodeAtPath(root: FieldTreeNode[], path: number[]): FieldTreeNode[] {
  const tree = deepCloneTree(root)
  const list = resolveParentArray(tree, path)
  const idx = path[path.length - 1]
  if (!list || idx === undefined) return tree
  list.splice(idx, 1)
  return ensureTrailingEmptyRoot(tree)
}

export function addSiblingAtPath(root: FieldTreeNode[], path: number[]): FieldTreeNode[] {
  const tree = deepCloneTree(root)
  const list = resolveParentArray(tree, path)
  const idx = path[path.length - 1]
  if (!list || idx === undefined) return tree
  const parentId = path.length > 1 ? (getNodeAtPath(tree, path.slice(0, -1))?.id ?? 0) : 0
  list.splice(idx + 1, 0, emptyFieldNode(parentId))
  return ensureTrailingEmptyRoot(tree)
}

export function addChildAtPath(root: FieldTreeNode[], path: number[]): FieldTreeNode[] {
  const tree = deepCloneTree(root)
  const node = getNodeAtPath(tree, path)
  if (!node) return tree
  if (!node.children) node.children = []
  node.children.push(emptyFieldNode(node.id ?? 0))
  return ensureTrailingEmptyRoot(tree)
}

export function reorderSiblingAtPath(
  root: FieldTreeNode[],
  parentPath: number[],
  fromIndex: number,
  toIndex: number,
): FieldTreeNode[] {
  if (fromIndex === toIndex) return root
  const tree = deepCloneTree(root)
  let list: FieldTreeNode[]
  if (parentPath.length === 0) {
    list = tree
    const reordered = reorderList(list, fromIndex, toIndex)
    return ensureTrailingEmptyRoot(reordered)
  }
  const parent = getNodeAtPath(tree, parentPath)
  if (!parent) return tree
  if (!parent.children) parent.children = []
  parent.children = reorderList(parent.children, fromIndex, toIndex)
  return ensureTrailingEmptyRoot(tree)
}

export function isTrailingPlaceholderRow(row: FlatFieldRow, rows: FlatFieldRow[]): boolean {
  if (hasFieldContent(row.node)) return false
  if (row.depth !== 0) return false
  const last = rows[rows.length - 1]
  return last.path.join('-') === row.path.join('-')
}

export function siblingParentPath(path: number[]): number[] {
  return path.slice(0, -1)
}

export function areSiblingPaths(left: number[], right: number[]): boolean {
  if (left.length !== right.length) return false
  return siblingParentPath(left).join('-') === siblingParentPath(right).join('-')
}

export function compactFieldTree(nodes: FieldTreeNode[]): FieldTreeNode[] {
  return nodes
    .filter(hasFieldContent)
    .map((node) => {
      const children = compactFieldTree(node.children ?? [])
      return {
        ...node,
        children: children.length ? children : undefined,
      }
    })
}

export function normalizeResponseFields(nodes: FieldTreeNode[]): InterfaceResponseField[] {
  return nodes.map((node) => ({
    id: node.id,
    parent_id: node.parent_id,
    name: node.name,
    type: node.type,
    required: node.required,
    description: node.description,
    mock: node.mock ?? '',
    example: node.example,
    children: node.children?.length ? normalizeResponseFields(node.children) : undefined,
  }))
}

export function responseFieldTreeFromApi(nodes: InterfaceResponseField[]): InterfaceResponseField[] {
  const cloned = deepCloneTree(nodes as FieldTreeNode[])
  return normalizeResponseFields(ensureTrailingEmptyRoot(cloned.length ? cloned : []))
}

export function compactResponseFieldTree(nodes: FieldTreeNode[]): InterfaceResponseField[] {
  return normalizeResponseFields(compactFieldTree(nodes))
}

export function setAllFieldsRequired(nodes: FieldTreeNode[], required: boolean): FieldTreeNode[] {
  const tree = deepCloneTree(nodes)

  function walk(list: FieldTreeNode[]) {
    for (const node of list) {
      node.required = required
      if (node.children?.length) {
        walk(node.children)
      }
    }
  }

  walk(tree)
  return ensureTrailingEmptyRoot(tree)
}

export function countFilledFieldRequired(nodes: FieldTreeNode[]): { filled: number; required: number } {
  let filled = 0
  let required = 0

  function walk(list: FieldTreeNode[]) {
    for (const node of list) {
      if (hasFieldContent(node)) {
        filled += 1
        if (node.required) required += 1
      }
      if (node.children?.length) walk(node.children)
    }
  }

  walk(nodes)
  return { filled, required }
}
