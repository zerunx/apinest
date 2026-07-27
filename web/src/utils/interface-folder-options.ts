import type { ApiTreeNode } from '@/types/workspace'

export interface FolderOption {
  folderId: number
  label: string
}

export function collectFolderOptions(
  nodes: ApiTreeNode[],
  parseFolderId: (id: string) => number | null,
  depth = 0,
): FolderOption[] {
  const result: FolderOption[] = []
  for (const node of nodes) {
    if (node.type !== 'folder') continue
    const folderId = parseFolderId(node.id)
    if (folderId) {
      result.push({
        folderId,
        label: `${'\u3000'.repeat(depth)}${node.name}`,
      })
    }
    if (node.children?.length) {
      result.push(...collectFolderOptions(node.children, parseFolderId, depth + 1))
    }
  }
  return result
}
