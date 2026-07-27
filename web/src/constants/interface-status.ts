import type { InterfaceStatus } from '@/types/workspace'

export interface StatusOption {
  value: InterfaceStatus
  labelKey: string
  color: string
}

export const INTERFACE_STATUS_OPTIONS: StatusOption[] = [
  { value: 1, labelKey: 'workspace.status.published', color: '#49cc90' },
  { value: 2, labelKey: 'workspace.status.testing', color: '#409eff' },
  { value: 3, labelKey: 'workspace.status.developing', color: '#61affe' },
  { value: 4, labelKey: 'workspace.status.abnormal', color: '#f93e3e' },
  { value: 5, labelKey: 'workspace.status.maintenance', color: '#fca130' },
  { value: 6, labelKey: 'workspace.status.deprecated', color: '#909399' },
]

export function interfaceStatusKey(status: number): string {
  const item = INTERFACE_STATUS_OPTIONS.find((option) => option.value === status)
  return item?.labelKey.split('.').pop() ?? 'testing'
}

export function interfaceStatusColor(status: number): string {
  const item = INTERFACE_STATUS_OPTIONS.find((option) => option.value === status)
  return item?.color ?? '#409eff'
}
