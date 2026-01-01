<script setup lang="ts">
import { computed } from 'vue'
import { formatTimestamp, copyToClipboard } from '../lib/utils'

interface Props {
  header: any // WASM header object
}

const props = defineProps<Props>()

interface HeaderField {
  key: string
  label: string
  value: string
  type: 'text' | 'timestamp' | 'json'
}

const fields = computed<HeaderField[]>(() => {
  const result: HeaderField[] = []

  if (props.header.creatorId) {
    result.push({
      key: 'creator_id',
      label: 'Creator ID',
      value: props.header.creatorId,
      type: 'text',
    })
  }

  if (props.header.signedAt !== undefined) {
    result.push({
      key: 'signed_at',
      label: 'Signed At',
      value: formatTimestamp(props.header.signedAt),
      type: 'timestamp',
    })
  }

  if (props.header.contentType) {
    result.push({
      key: 'content_type',
      label: 'Content Type',
      value: props.header.contentType,
      type: 'text',
    })
  }

  if (props.header.originalName) {
    result.push({
      key: 'original_name',
      label: 'Original Name',
      value: props.header.originalName,
      type: 'text',
    })
  }

  if (props.header.description) {
    result.push({
      key: 'description',
      label: 'Description',
      value: props.header.description,
      type: 'text',
    })
  }

  if (props.header.custom && Object.keys(props.header.custom).length > 0) {
    result.push({
      key: 'custom',
      label: 'Custom Fields',
      value: JSON.stringify(props.header.custom, null, 2),
      type: 'json',
    })
  }

  return result
})

async function copyField(value: string) {
  try {
    await copyToClipboard(value)
  } catch (error) {
    console.error('Failed to copy:', error)
  }
}
</script>

<template>
  <div class="h-full overflow-auto bg-white">
    <div class="p-6 max-w-4xl mx-auto">
      <h3 class="text-xl font-bold text-gray-900 mb-6">Header Information</h3>

      <div class="space-y-4">
        <div
          v-for="field in fields"
          :key="field.key"
          class="border border-gray-200 rounded-lg p-4 hover:bg-gray-50 transition-colors"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1">
              <div class="text-sm font-semibold text-gray-600 mb-1">
                {{ field.label }}
              </div>
              <div
                v-if="field.type === 'json'"
                class="text-sm font-mono bg-gray-100 p-3 rounded overflow-x-auto"
              >
                <pre>{{ field.value }}</pre>
              </div>
              <div
                v-else-if="field.type === 'timestamp'"
                class="text-base text-gray-900"
              >
                <div class="font-mono">{{ field.value }}</div>
                <div class="text-xs text-gray-600 mt-1">
                  Unix timestamp: {{ header.signedAt }}
                </div>
              </div>
              <div v-else class="text-base text-gray-900 break-all">
                {{ field.value }}
              </div>
            </div>

            <button
              type="button"
              class="flex-shrink-0 p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded transition-colors"
              title="Copy to clipboard"
              @click="copyField(field.value)"
            >
              <svg
                class="w-5 h-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                />
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Raw JSON View -->
      <details class="mt-8 border border-gray-200 rounded-lg">
        <summary class="p-4 cursor-pointer hover:bg-gray-50 font-semibold text-gray-700">
          View Raw JSON
        </summary>
        <div class="p-4 bg-gray-50 border-t border-gray-200">
          <pre class="text-xs font-mono overflow-x-auto">{{ JSON.stringify(header, null, 2) }}</pre>
        </div>
      </details>
    </div>
  </div>
</template>
