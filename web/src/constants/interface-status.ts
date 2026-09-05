import type { InterfaceStatus } from '@/types/workspace'

export interface StatusOption {
  value: InterfaceStatus
  labelKey: string
  color: string
}

/** 1 已发布 / 2 联调中 / 3 开发中 / 6 已废弃（历史 4、5 已废弃，见 normalize） */
export const INTERFACE_STATUS_OPTIONS: StatusOption[] = [
  { value: 3, labelKey: 'workspace.status.developing', color: '#61affe' },
  { value: 2, labelKey: 'workspace.status.integrating', color: '#409eff' },
  { value: 1, labelKey: 'workspace.status.published', color: '#49cc90' },
  { value: 6, labelKey: 'workspace.status.deprecated', color: '#909399' },
]

export const DEFAULT_INTERFACE_STATUS: InterfaceStatus = 3

/** Map legacy abnormal(4) / maintenance(5) to developing. */
export function normalizeInterfaceStatus(status: number): InterfaceStatus {
  if (status === 1 || status === 2 || status === 3 || status === 6) {
    return status
  }
  return DEFAULT_INTERFACE_STATUS
}

export function interfaceStatusKey(status: number): string {
  const normalized = normalizeInterfaceStatus(status)
  const item = INTERFACE_STATUS_OPTIONS.find((option) => option.value === normalized)
  return item?.labelKey.split('.').pop() ?? 'developing'
}

export function interfaceStatusColor(status: number): string {
  const normalized = normalizeInterfaceStatus(status)
  const item = INTERFACE_STATUS_OPTIONS.find((option) => option.value === normalized)
  return item?.color ?? '#61affe'
}
