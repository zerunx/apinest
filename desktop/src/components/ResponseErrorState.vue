<template>
  <div class="res-error" role="alert">
    <div class="res-error__card">
      <svg class="res-error__icon" viewBox="0 0 24 24" width="32" height="32" aria-hidden="true">
        <circle cx="12" cy="12" r="9.25" fill="none" stroke="currentColor" stroke-width="1.6" />
        <path
          d="M12 7.4v6.1M12 16.35v.1"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
        />
      </svg>
      <p class="res-error__title">{{ parsed.title }}</p>
      <pre v-if="parsed.rest" class="res-error__detail">{{ parsed.rest }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  message: string
  title?: string
}>()

const parsed = computed(() => parseSendError(props.message, props.title))

function capitalize(text: string) {
  const trimmed = text.trim()
  if (!trimmed) return trimmed
  return trimmed.charAt(0).toUpperCase() + trimmed.slice(1)
}

function parseSendError(raw: string, titleOverride?: string) {
  const text = raw.trim()
  const urlMatch = text.match(/\((https?:\/\/[^)]+)\)\s*$/i)
  const url = urlMatch?.[1] ?? ''
  const withoutUrl = url && urlMatch?.index != null ? text.slice(0, urlMatch.index).trim() : text
  const colon = withoutUrl.indexOf(': ')
  const head = colon >= 0 ? withoutUrl.slice(0, colon).trim() : ''
  const detail = colon >= 0 ? withoutUrl.slice(colon + 2).trim() : withoutUrl
  const rest = [detail, url ? `(${url})` : ''].filter(Boolean).join(' ')
  return {
    title: capitalize(titleOverride || head || 'Error'),
    rest,
  }
}
</script>

<style scoped>
.res-error {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: stretch;
  justify-content: center;
  padding: 16px 24px;
  overflow: auto;
}

.res-error__card {
  display: flex;
  flex-direction: column;
  align-items: center;
  align-self: center;
  width: min(920px, 100%);
  max-height: 100%;
  padding: 22px 36px;
  text-align: center;
  color: var(--color-danger);
  background: color-mix(in srgb, var(--color-danger) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-danger) 22%, transparent);
  border-radius: 10px;
  overflow: auto;
}

.res-error__icon {
  display: block;
  margin-bottom: 8px;
  flex-shrink: 0;
}

.res-error__title {
  margin: 0;
  font-size: 15px;
  font-weight: 650;
  line-height: 1.35;
  flex-shrink: 0;
}

.res-error__detail {
  margin: 8px 0 0;
  max-width: 100%;
  font-size: 13px;
  line-height: 1.55;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  word-break: break-word;
  text-align: left;
  user-select: text;
  -webkit-user-select: text;
  opacity: 0.92;
  font-family: inherit;
}
</style>
