import type { DictionaryEntryRow } from '../types'

export function normalizeDictKey(name: string) {
  return name.trim().toLowerCase()
}

export function dictMatchesParam(entry: DictionaryEntryRow, paramName: string) {
  const key = normalizeDictKey(paramName)
  return !!key && normalizeDictKey(entry.name) === key
}

export function paramHasDictMatch(entries: DictionaryEntryRow[], paramName: string) {
  return entries.some((entry) => dictMatchesParam(entry, paramName))
}

/** Matched entries first (stable), then the rest. */
export function sortDictEntriesForPicker(entries: DictionaryEntryRow[], paramName: string) {
  const matched: DictionaryEntryRow[] = []
  const rest: DictionaryEntryRow[] = []
  for (const entry of entries) {
    if (dictMatchesParam(entry, paramName)) matched.push(entry)
    else rest.push(entry)
  }
  return [...matched, ...rest]
}

/** First dictionary entry wins for each normalized name. */
export function buildDictByNameMap(entries: DictionaryEntryRow[]) {
  const map = new Map<string, DictionaryEntryRow>()
  for (const entry of entries) {
    const key = normalizeDictKey(entry.name)
    if (!key || map.has(key)) continue
    map.set(key, entry)
  }
  return map
}

/**
 * Apply dictionary description (always when matched) and default_value
 * only when the row value is empty. Never changes type.
 * Returns how many rows were updated.
 */
export function applyDictDescriptionsToFormData(
  rows: Array<{ name: string; description: string; example?: string; type?: string }>,
  entries: DictionaryEntryRow[],
) {
  const map = buildDictByNameMap(entries)
  let count = 0
  for (const row of rows) {
    const key = normalizeDictKey(row.name)
    if (!key) continue
    const hit = map.get(key)
    if (!hit) continue

    let changed = false
    const nextDesc = hit.description ?? ''
    if (row.description !== nextDesc) {
      row.description = nextDesc
      changed = true
    }

    // Text rows only: fill default when value is empty.
    if (row.type !== 'file') {
      const current = (row.example ?? '').trim()
      const fallback = (hit.default_value ?? '').trim()
      if (!current && fallback) {
        row.example = fallback
        changed = true
      }
    }

    if (changed) count += 1
  }
  return count
}
