import { ref } from 'vue'

export function useTableRowDrag(options: {
  canDrag: (index: number) => boolean
  onReorder: (fromIndex: number, toIndex: number) => void
}) {
  const dragIndex = ref<number | null>(null)
  const dragOverIndex = ref<number | null>(null)

  function resetDragState() {
    dragIndex.value = null
    dragOverIndex.value = null
  }

  function handleDragStart(event: DragEvent, index: number) {
    if (!options.canDrag(index)) {
      event.preventDefault()
      return
    }
    dragIndex.value = index
    dragOverIndex.value = null
    event.dataTransfer?.setData('text/plain', String(index))
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move'
    }
  }

  function handleDragOver(event: DragEvent, index: number) {
    if (dragIndex.value === null || !options.canDrag(index)) return
    if (index === dragIndex.value) return
    event.preventDefault()
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move'
    }
    dragOverIndex.value = index
  }

  function handleDragLeave(index: number) {
    if (dragOverIndex.value === index) {
      dragOverIndex.value = null
    }
  }

  function handleDrop(event: DragEvent, index: number) {
    event.preventDefault()
    const from = dragIndex.value
    if (from === null || from === index || !options.canDrag(index)) {
      resetDragState()
      return
    }
    options.onReorder(from, index)
    resetDragState()
  }

  function handleDragEnd() {
    resetDragState()
  }

  function rowClass(index: number) {
    return {
      'interface-param-table__row--dragging': dragIndex.value === index,
      'interface-param-table__row--drag-over': dragOverIndex.value === index,
    }
  }

  return {
    handleDragStart,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    handleDragEnd,
    rowClass,
  }
}

export function reorderList<T>(items: T[], fromIndex: number, toIndex: number): T[] {
  if (fromIndex === toIndex) return [...items]
  const next = [...items]
  const [moved] = next.splice(fromIndex, 1)
  const insertIndex = fromIndex < toIndex ? toIndex : toIndex + 1
  next.splice(insertIndex, 0, moved)
  return next
}
