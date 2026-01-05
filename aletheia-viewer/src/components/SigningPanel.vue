<script setup lang="ts">
import { ref, computed } from 'vue'
import { sign_file_with_ca, parse_certificate } from '../lib/wasm-pkg/aletheia.js'
import { parsePrivateKey, detectContentTypeFromFilename, downloadFile, formatBytes, parseCertificateBytes } from '../lib/utils'
import FileUploader from './FileUploader.vue'

// State
const caPrivateKey = ref<Uint8Array | null>(null)
const caCertificate = ref<Uint8Array | null>(null)
const caCertInfo = ref<{ subjectId: string; subjectName: string; issuerId: string } | null>(null)

const payloadFile = ref<File | null>(null)
const payloadBytes = ref<Uint8Array | null>(null)

const contentType = ref('')
const description = ref('')
const enableCompression = ref(false)

const isSigning = ref(false)
const signedFileBytes = ref<Uint8Array | null>(null)
const error = ref<string | null>(null)
const keyError = ref<string | null>(null)
const certError = ref<string | null>(null)

// Computed
const caLoaded = computed(() => caPrivateKey.value !== null && caCertificate.value !== null)
const canSign = computed(() => caLoaded.value && payloadBytes.value !== null && !isSigning.value)
const outputFilename = computed(() => {
  if (!payloadFile.value) return 'signed.alx'
  return payloadFile.value.name + '.alx'
})

// CA Key Loading
async function handleCAKeyLoad(files: File[]) {
  if (files.length === 0) return
  
  keyError.value = null
  error.value = null
  signedFileBytes.value = null
  
  try {
    caPrivateKey.value = await parsePrivateKey(files[0])
    console.log('✅ CA private key loaded')
  } catch (e) {
    keyError.value = e instanceof Error ? e.message : 'Failed to load private key'
    caPrivateKey.value = null
  }
}

// CA Certificate Loading
async function handleCACertLoad(files: File[]) {
  if (files.length === 0) return
  
  certError.value = null
  error.value = null
  signedFileBytes.value = null
  
  try {
    const rawBytes = new Uint8Array(await files[0].arrayBuffer())
    console.log(`Loading certificate: ${files[0].name}, ${rawBytes.length} bytes`)
    
    // Parse certificate bytes (handles base64 encoding)
    const bytes = parseCertificateBytes(rawBytes)
    console.log(`Decoded certificate: ${bytes.length} bytes`)
    
    // Validate by parsing
    const certInfo = parse_certificate(bytes)
    console.log('Parsed certificate:', certInfo)
    
    caCertificate.value = bytes
    caCertInfo.value = {
      subjectId: certInfo.subjectId,
      subjectName: certInfo.subjectName,
      issuerId: certInfo.issuerId,
    }
    
    console.log('✅ CA certificate loaded:', caCertInfo.value)
  } catch (e) {
    console.error('Certificate load error:', e)
    certError.value = e instanceof Error ? e.message : String(e)
    caCertificate.value = null
    caCertInfo.value = null
  }
}

// File to Sign Loading
async function handlePayloadLoad(files: File[]) {
  if (files.length === 0) return
  
  error.value = null
  signedFileBytes.value = null
  
  try {
    const file = files[0]
    payloadFile.value = file
    payloadBytes.value = new Uint8Array(await file.arrayBuffer())
    
    // Auto-detect content type from filename
    contentType.value = detectContentTypeFromFilename(file.name)
    
    console.log(`✅ File loaded: ${file.name} (${formatBytes(payloadBytes.value.length)})`)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load file'
    payloadFile.value = null
    payloadBytes.value = null
  }
}

// Sign the file
async function signFile() {
  if (!canSign.value || !caPrivateKey.value || !caCertificate.value || !payloadBytes.value) {
    return
  }
  
  isSigning.value = true
  error.value = null
  signedFileBytes.value = null
  
  try {
    // Use creator_id from certificate or default
    const creatorId = caCertInfo.value?.subjectId || 'unknown@local'
    
    const result = sign_file_with_ca(
      payloadBytes.value,
      caPrivateKey.value,
      caCertificate.value,
      creatorId,
      contentType.value || undefined,
      payloadFile.value?.name || undefined,
      description.value || undefined,
      enableCompression.value
    )
    
    signedFileBytes.value = result
    console.log(`✅ File signed successfully: ${formatBytes(result.length)}`)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Signing failed: ' + String(e)
    console.error('Signing error:', e)
  } finally {
    isSigning.value = false
  }
}

// Download the signed file
function downloadSignedFile() {
  if (!signedFileBytes.value) return
  downloadFile(signedFileBytes.value, outputFilename.value)
}

// Reset for signing another file
function resetForNewFile() {
  payloadFile.value = null
  payloadBytes.value = null
  contentType.value = ''
  description.value = ''
  enableCompression.value = false
  signedFileBytes.value = null
  error.value = null
}
</script>

<template>
  <div class="p-6 space-y-6 max-w-3xl mx-auto">
    <!-- Security Notice -->
    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
      <div class="flex items-start gap-3">
        <svg class="h-6 w-6 text-blue-600 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <div class="text-sm text-blue-800">
          <p class="font-medium">Your keys never leave your browser</p>
          <p class="mt-1">All cryptographic operations are performed locally using WebAssembly. Your CA private key is never sent to any server.</p>
        </div>
      </div>
    </div>

    <!-- Step 1: CA Credentials -->
    <div class="bg-white border border-gray-200 rounded-lg p-6">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Step 1: Load CA Credentials</h3>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- CA Private Key -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">CA Private Key</label>
          <FileUploader
            label="Select ca.key file"
            accept=".key,*"
            variant="secondary"
            @load="handleCAKeyLoad"
          />
          <div v-if="caPrivateKey" class="mt-2 text-sm text-green-600 flex items-center gap-1">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            Key loaded (32 bytes)
          </div>
          <div v-if="keyError" class="mt-2 text-sm text-red-600">{{ keyError }}</div>
        </div>
        
        <!-- CA Certificate -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">CA Certificate</label>
          <FileUploader
            label="Select ca.cert file"
            accept=".cert,*"
            variant="secondary"
            @load="handleCACertLoad"
          />
          <div v-if="caCertInfo" class="mt-2 text-sm text-green-600">
            <div class="flex items-center gap-1">
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              {{ caCertInfo.subjectName }}
            </div>
            <div class="text-xs text-gray-500 mt-1">{{ caCertInfo.subjectId }}</div>
          </div>
          <div v-if="certError" class="mt-2 text-sm text-red-600">{{ certError }}</div>
        </div>
      </div>
      
      <p class="mt-4 text-xs text-gray-500">
        Generate CA credentials using: <code class="bg-gray-100 px-1 rounded">aletheia ca init --ca-id "you@example.com" --ca-name "Your Name"</code>
      </p>
    </div>

    <!-- Step 2: Select File -->
    <div :class="['bg-white border rounded-lg p-6', caLoaded ? 'border-gray-200' : 'border-gray-100 opacity-60']">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Step 2: Select File to Sign</h3>
      
      <FileUploader
        label="Choose file"
        accept="*"
        :variant="caLoaded ? 'primary' : 'secondary'"
        @load="handlePayloadLoad"
      />
      
      <div v-if="payloadFile && payloadBytes" class="mt-4 p-3 bg-gray-50 rounded-md">
        <div class="flex items-center gap-3">
          <svg class="h-8 w-8 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
          </svg>
          <div>
            <div class="font-medium text-gray-900">{{ payloadFile.name }}</div>
            <div class="text-sm text-gray-500">{{ formatBytes(payloadBytes.length) }}</div>
          </div>
        </div>
      </div>
      
      <p v-if="!caLoaded" class="mt-4 text-sm text-gray-500">
        Load CA credentials first to enable file selection.
      </p>
    </div>

    <!-- Step 3: Metadata -->
    <div :class="['bg-white border rounded-lg p-6', payloadFile ? 'border-gray-200' : 'border-gray-100 opacity-60']">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Step 3: Metadata (Optional)</h3>
      
      <div class="space-y-4">
        <!-- Content Type -->
        <div>
          <label for="contentType" class="block text-sm font-medium text-gray-700 mb-1">Content Type</label>
          <input
            id="contentType"
            v-model="contentType"
            type="text"
            placeholder="e.g., image/png, text/plain"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            :disabled="!payloadFile"
          />
          <p class="mt-1 text-xs text-gray-500">Auto-detected from filename. Override if needed.</p>
        </div>
        
        <!-- Description -->
        <div>
          <label for="description" class="block text-sm font-medium text-gray-700 mb-1">Description</label>
          <textarea
            id="description"
            v-model="description"
            rows="2"
            placeholder="Optional description of the content"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
            :disabled="!payloadFile"
          />
        </div>
        
        <!-- Compression -->
        <div class="flex items-center gap-2">
          <input
            id="compression"
            v-model="enableCompression"
            type="checkbox"
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            :disabled="!payloadFile"
          />
          <label for="compression" class="text-sm text-gray-700">
            Enable LZ4 compression
          </label>
          <span class="text-xs text-gray-500">(recommended for text files)</span>
        </div>
      </div>
    </div>

    <!-- Sign Button -->
    <div class="flex justify-center">
      <button
        type="button"
        :disabled="!canSign"
        :class="[
          'px-8 py-3 rounded-lg font-semibold text-lg transition-colors',
          canSign
            ? 'bg-blue-600 text-white hover:bg-blue-700'
            : 'bg-gray-200 text-gray-400 cursor-not-allowed'
        ]"
        @click="signFile"
      >
        <span v-if="isSigning" class="flex items-center gap-2">
          <svg class="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Signing...
        </span>
        <span v-else>Sign File</span>
      </button>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
      <div class="flex items-start gap-3">
        <svg class="h-6 w-6 text-red-600 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <div>
          <p class="font-medium text-red-900">Signing Error</p>
          <p class="text-sm text-red-800 mt-1 font-mono">{{ error }}</p>
        </div>
      </div>
    </div>

    <!-- Success Result -->
    <div v-if="signedFileBytes" class="bg-green-50 border border-green-200 rounded-lg p-6">
      <div class="flex items-start gap-3">
        <svg class="h-8 w-8 text-green-600 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <div class="flex-1">
          <h4 class="text-lg font-semibold text-green-900">File Signed Successfully!</h4>
          <p class="text-sm text-green-800 mt-1">
            Output: <span class="font-mono">{{ outputFilename }}</span> ({{ formatBytes(signedFileBytes.length) }})
          </p>
          
          <div class="flex gap-3 mt-4">
            <button
              type="button"
              class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 font-medium"
              @click="downloadSignedFile"
            >
              Download .alx File
            </button>
            <button
              type="button"
              class="px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 font-medium"
              @click="resetForNewFile"
            >
              Sign Another File
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
