<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { detectContentType, decodeText, formatBytes } from '../lib/utils'

interface Props {
  payload: Uint8Array
  isCompressed: boolean
  contentType?: string
  isVerified: boolean
}

const props = defineProps<Props>()

const decompressedPayload = ref<Uint8Array | null>(null)
const decompressError = ref<string | null>(null)
const isDecompressing = ref(false)

const actualContentType = computed(() => {
  const payloadToUse = decompressedPayload.value || props.payload
  return detectContentType(payloadToUse, props.contentType)
})

const isImage = computed(() => actualContentType.value.startsWith('image/'))
const isText = computed(() => actualContentType.value.startsWith('text/'))

const imageDataUrl = ref<string | null>(null)
const textContent = ref<string | null>(null)
const textError = ref<string | null>(null)

// Decompress if needed
watch(() => props.isVerified, async (isVerified) => {
  if (!isVerified) return

  if (props.isCompressed) {
    isDecompressing.value = true
    try {
      // Import WASM decompress function
      const { decompress_payload } = await import('../lib/wasm-pkg/aletheia.js')
      const decompressed = decompress_payload(props.payload, true)
      decompressedPayload.value = decompressed
    } catch (error) {
      decompressError.value = error instanceof Error ? error.message : 'Decompression failed'
    } finally {
      isDecompressing.value = false
    }
  } else {
    decompressedPayload.value = props.payload
  }
}, { immediate: true })

// Load image when payload is ready
watch(decompressedPayload, (payload) => {
  if (!payload) return
  
  // Check if it's an image
  const contentType = detectContentType(payload, props.contentType)
  if (!contentType.startsWith('image/')) return

  console.log('Loading image preview:', contentType, payload.length, 'bytes')
  const blob = new Blob([payload], { type: contentType })
  imageDataUrl.value = URL.createObjectURL(blob)
}, { immediate: true })

// Load text when payload is ready
watch(decompressedPayload, (payload) => {
  if (!payload) return
  
  // Check if it's text
  const contentType = detectContentType(payload, props.contentType)
  if (!contentType.startsWith('text/')) return

  try {
    textContent.value = decodeText(payload)
  } catch (error) {
    textError.value = error instanceof Error ? error.message : 'Failed to decode text'
  }
}, { immediate: true })

function downloadPayload() {
  if (!decompressedPayload.value) return

  const blob = new Blob([decompressedPayload.value as any], { type: actualContentType.value })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'payload' + getExtension(actualContentType.value)
  a.click()
  URL.revokeObjectURL(url)
}

function getExtension(contentType: string): string {
  const map: Record<string, string> = {
    'image/png': '.png',
    'image/jpeg': '.jpg',
    'image/gif': '.gif',
    'image/webp': '.webp',
    'text/plain': '.txt',
    'application/pdf': '.pdf',
    'application/json': '.json',
  }
  return map[contentType] || '.bin'
}
</script>

<template>
  <div class="h-full overflow-auto bg-white">
    <!-- Security Warning if not verified -->
    <div v-if="!isVerified" class="h-full flex items-center justify-center p-8">
      <div class="max-w-md text-center">
        <svg
          class="mx-auto h-16 w-16 text-red-500 mb-4"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
          />
        </svg>
        <h3 class="text-lg font-bold text-gray-900 mb-2">Payload Preview Disabled</h3>
        <p class="text-sm text-gray-600">
          For security reasons, payload preview is only available after successful cryptographic verification.
        </p>
        <div class="mt-4 p-3 bg-red-50 border border-red-200 rounded text-xs text-left">
          <strong class="text-red-900">Why?</strong>
          <p class="text-red-800 mt-1">
            Malicious content could exploit rendering vulnerabilities. We only show content after verifying:
          </p>
          <ul class="mt-2 space-y-1 text-red-700 list-disc list-inside">
            <li>Certificate chain is valid</li>
            <li>Root CA is trusted</li>
            <li>File signature is correct</li>
          </ul>
        </div>
      </div>
    </div>

    <!-- Verified content -->
    <div v-else class="p-6">
      <!-- Header -->
      <div class="mb-6 flex items-center justify-between">
        <div>
          <h3 class="text-xl font-bold text-gray-900">Payload Preview</h3>
          <p class="text-sm text-gray-600 mt-1">
            {{ formatBytes(payload.length) }}
            <span v-if="isCompressed"> (compressed)</span>
            <span v-if="decompressedPayload && isCompressed">
              → {{ formatBytes(decompressedPayload.length) }} (decompressed)
            </span>
          </p>
          <p class="text-xs text-gray-500 mt-1">
            Content-Type: {{ actualContentType }}
          </p>
        </div>

        <button
          type="button"
          class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 font-medium text-sm"
          @click="downloadPayload"
        >
          Download
        </button>
      </div>

      <!-- Decompression in progress -->
      <div v-if="isDecompressing" class="text-center py-12">
        <div class="text-gray-600">Decompressing...</div>
      </div>

      <!-- Decompression error -->
      <div v-else-if="decompressError" class="bg-red-50 border border-red-200 rounded-lg p-4">
        <div class="text-red-900 font-semibold">Decompression Failed</div>
        <div class="text-red-700 text-sm mt-1">{{ decompressError }}</div>
      </div>

      <!-- Image preview -->
      <div v-else-if="isImage && imageDataUrl" class="border border-gray-200 rounded-lg p-4 bg-gray-50">
        <img :src="imageDataUrl" alt="Payload image" class="max-w-full h-auto mx-auto rounded" />
      </div>

      <!-- Text preview -->
      <div v-else-if="isText && textContent" class="border border-gray-200 rounded-lg p-4 bg-gray-50">
        <pre class="text-sm font-mono whitespace-pre-wrap break-words">{{ textContent }}</pre>
      </div>

      <!-- Text error -->
      <div v-else-if="isText && textError" class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <div class="text-yellow-900 font-semibold">Text Decode Error</div>
        <div class="text-yellow-700 text-sm mt-1">{{ textError }}</div>
        <button
          type="button"
          class="mt-3 px-3 py-1 bg-yellow-600 text-white rounded text-sm hover:bg-yellow-700"
          @click="downloadPayload"
        >
          Download as binary
        </button>
      </div>

      <!-- PDF or other types -->
      <div v-else class="border border-gray-200 rounded-lg p-8 text-center">
        <svg
          class="mx-auto h-16 w-16 text-gray-400 mb-4"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
          />
        </svg>
        <p class="text-gray-600 mb-4">
          Preview not available for this content type
        </p>
        <button
          type="button"
          class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
          @click="downloadPayload"
        >
          Download to view
        </button>
      </div>
    </div>
  </div>
</template>
