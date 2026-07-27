import { ref } from 'vue'

const sharePageTitle = ref('')
const sharePageSubtitle = ref('')

export function useSharePageMeta() {
  function setSharePageMeta(title: string, subtitle = '') {
    sharePageTitle.value = title
    sharePageSubtitle.value = subtitle
  }

  function clearSharePageMeta() {
    sharePageTitle.value = ''
    sharePageSubtitle.value = ''
  }

  return {
    sharePageTitle,
    sharePageSubtitle,
    setSharePageMeta,
    clearSharePageMeta,
  }
}
