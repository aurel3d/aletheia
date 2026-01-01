<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import init, { parse_aletheia_file, verify_aletheia_file } from './lib/wasm-pkg/aletheia.js'

import FileUploader from './components/FileUploader.vue'
import VerificationPanel from './components/VerificationPanel.vue'
import FileStructureTree from './components/FileStructureTree.vue'
import DetailsPanel from './components/DetailsPanel.vue'

// WASM initialization
const wasmInitialized = ref(false)

onMounted(async () => {
  try {
    await init()
    wasmInitialized.value = true
  } catch (error) {
    console.error('Failed to initialize WASM:', error)
  }
})

const aletheiaFile = ref<any | null>(null) // WASM parsed file
const rawFileBytes = ref<Uint8Array | null>(null) // Keep raw bytes for verification
const verificationResult = ref<any | null>(null) // WASM verification result
const trustedRoots = ref<Uint8Array[]>([])
const parseError = ref<string | null>(null)
const selectedNode = ref<string | null>(null)
const selectedRange = ref<[number, number] | null>(null)
const isVerifying = ref(false)

// Load .alx file
async function handleFileLoad(file: File) {
  if (!wasmInitialized.value) {
    parseError.value = 'WASM module not initialized yet'
    return
  }

  parseError.value = null
  aletheiaFile.value = null
  rawFileBytes.value = null
  verificationResult.value = null
  selectedNode.value = null
  selectedRange.value = null

  try {
    const arrayBuffer = await file.arrayBuffer()
    const bytes = new Uint8Array(arrayBuffer)
    rawFileBytes.value = bytes

    // Parse file using WASM
    const parsed = parse_aletheia_file(bytes)
    aletheiaFile.value = parsed

    // Auto-verify if we have trusted roots
    if (trustedRoots.value.length > 0) {
      await performVerification()
    }
  } catch (error) {
    parseError.value = error instanceof Error ? error.message : 'Parse error: ' + String(error)
  }
}

// Load trusted root certificates
async function handleTrustedRootsLoad(files: File[]) {
  try {
    const { decode } = await import('cbor-x')
    const roots: Uint8Array[] = []

    for (const file of files) {
      const arrayBuffer = await file.arrayBuffer()
      const bytes = new Uint8Array(arrayBuffer)

      // Try to parse as raw public key (32 bytes)
      if (bytes.length === 32) {
        roots.push(bytes)
        console.log(`Loaded raw public key from ${file.name}`)
        continue
      }

      // Try to parse as hex-encoded
      const text = new TextDecoder().decode(bytes).trim()
      if (/^[0-9a-fA-F]{64}$/.test(text)) {
        const hexBytes = new Uint8Array(32)
        for (let i = 0; i < 32; i++) {
          hexBytes[i] = parseInt(text.substr(i * 2, 2), 16)
        }
        roots.push(hexBytes)
        console.log(`Loaded hex-encoded public key from ${file.name}`)
        continue
      }

      // Try to parse as CBOR certificate (.cert file)
      try {
        const cert = decode(bytes)
        console.log('Decoded certificate structure:', cert)

        // Try different possible field names
        const publicKey = cert?.public_key || cert?.publicKey

        if (publicKey) {
          const keyBytes = new Uint8Array(publicKey)
          if (keyBytes.length === 32) {
            roots.push(keyBytes)
            const name = cert.subject_name || cert.subjectName || 'Unknown'
            const id = cert.subject_id || cert.subjectId || 'Unknown'
            console.log(`✅ Loaded certificate from ${file.name}: ${name} (${id})`)
            continue
          }
        }
      } catch (cborError) {
        console.log('CBOR decode attempt failed:', cborError.message)
        // Not a valid CBOR certificate
      }

      console.warn(`Skipping file '${file.name}': not a recognized format (expected: 32-byte raw key, 64-char hex, or .cert CBOR file)`)
    }

    trustedRoots.value = roots

    // Auto-verify if we have a file loaded
    if (aletheiaFile.value && rawFileBytes.value) {
      await performVerification()
    }
  } catch (error) {
    console.error('Failed to load trusted roots:', error)
  }
}

// Perform verification
async function performVerification() {
  if (!rawFileBytes.value || !wasmInitialized.value) return

  isVerifying.value = true
  try {
    // Convert trusted roots to format WASM expects
    const rootsArray = trustedRoots.value.map(root => Array.from(root))

    // Call WASM verification
    const result = verify_aletheia_file(rawFileBytes.value, rootsArray)

    // Convert WASM result to our format
    verificationResult.value = {
      isValid: result.valid,
      steps: [
        { id: 'parse', label: 'Parse file structure', status: 'success' },
        { id: 'cbor', label: 'Decode CBOR (header + certificates)', status: 'success' },
        { id: 'creator_id', label: 'Verify creator ID matches certificate', status: result.valid ? 'success' : 'error', error: result.valid ? undefined : 'Verification failed' },
        { id: 'cert_chain', label: 'Verify certificate chain signatures', status: result.valid ? 'success' : 'error' },
        { id: 'trusted_root', label: 'Verify root is trusted', status: result.valid ? 'success' : 'error' },
        { id: 'file_sig', label: 'Verify file signature', status: result.valid ? 'success' : 'error' },
      ],
    }
  } catch (error) {
    console.error('Verification error:', error)
    verificationResult.value = {
      isValid: false,
      steps: [
        { id: 'parse', label: 'Parse file structure', status: 'success' },
        { id: 'cbor', label: 'Decode CBOR (header + certificates)', status: 'success' },
        { id: 'creator_id', label: 'Verify creator ID matches certificate', status: 'error', error: String(error) },
        { id: 'cert_chain', label: 'Verify certificate chain signatures', status: 'pending' },
        { id: 'trusted_root', label: 'Verify root is trusted', status: 'pending' },
        { id: 'file_sig', label: 'Verify file signature', status: 'pending' },
      ],
    }
  } finally {
    isVerifying.value = false
  }
}

// Handle node selection from tree
function handleNodeSelect(nodeId: string, range: [number, number]) {
  selectedNode.value = nodeId
  selectedRange.value = range
}

// Handle hex click
function handleHexClick(offset: number) {
  // Find which node contains this offset
  if (!aletheiaFile.value) return

  const ranges = [
    { id: 'magic', range: aletheiaFile.value.magicRange },
    { id: 'version', range: aletheiaFile.value.versionRange },
    { id: 'flags', range: aletheiaFile.value.flagsRange },
    { id: 'header', range: aletheiaFile.value.headerRange },
    { id: 'payload', range: aletheiaFile.value.payloadRange },
    { id: 'certificate_chain', range: aletheiaFile.value.certChainRange },
    { id: 'signature', range: aletheiaFile.value.signatureRange },
  ]

  const found = ranges.find((r) => offset >= r.range[0] && offset < r.range[1])
  if (found) {
    selectedNode.value = found.id
    selectedRange.value = found.range
  }
}

const hasFile = computed(() => aletheiaFile.value !== null)
const isVerified = computed(() => verificationResult.value?.isValid === true)
</script>

<template>
  <div class="flex flex-col h-screen bg-gray-50">
    <!-- Header -->
    <header class="bg-white border-b border-gray-200 px-6 py-4 flex-shrink-0">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold text-gray-900">Aletheia File Viewer</h1>
          <p class="text-sm text-gray-600">Cryptographic proof of human-created content authenticity</p>
        </div>

        <div class="flex gap-3">
          <FileUploader
            label="Load Trusted Root(s)"
            accept=".cert,.pub,.key,.pem,.der,*"
            multiple
            variant="secondary"
            @load="handleTrustedRootsLoad"
          />
          <FileUploader
            label="Open .alx File"
            accept=".alx"
            @load="(files) => files[0] && handleFileLoad(files[0])"
          />
        </div>
      </div>

      <!-- Trusted roots indicator -->
      <div v-if="trustedRoots.length > 0" class="mt-2 text-sm text-green-700">
        ✓ {{ trustedRoots.length }} trusted root{{ trustedRoots.length > 1 ? 's' : '' }} loaded
      </div>
    </header>

    <!-- Main content -->
    <div v-if="!hasFile && !parseError" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <svg
          class="mx-auto h-24 w-24 text-gray-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
          />
        </svg>
        <h3 class="mt-4 text-lg font-medium text-gray-900">No file loaded</h3>
        <p class="mt-2 text-sm text-gray-600">
          Drop an .alx file here or click "Open .alx File" to begin
        </p>
      </div>
    </div>

    <!-- Parse error -->
    <div v-if="parseError" class="flex-1 flex items-center justify-center">
      <div class="max-w-2xl bg-red-50 border border-red-200 rounded-lg p-6">
        <div class="flex items-start">
          <svg
            class="h-6 w-6 text-red-600 mt-0.5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <div class="ml-3">
            <h3 class="text-lg font-medium text-red-900">Parse Error</h3>
            <p class="mt-2 text-sm text-red-800 font-mono">{{ parseError }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- File viewer -->
    <div v-if="hasFile" class="flex-1 flex flex-col overflow-hidden">
      <!-- Verification Panel -->
      <VerificationPanel
        :verification-result="verificationResult"
        :is-verifying="isVerifying"
        :has-trusted-roots="trustedRoots.length > 0"
        @verify="performVerification"
      />

      <!-- Split view -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Left: File Structure Tree -->
        <div class="w-1/3 border-r border-gray-200 overflow-auto bg-white">
          <FileStructureTree
            :file="aletheiaFile!"
            :selected-node="selectedNode"
            @select="handleNodeSelect"
          />
        </div>

        <!-- Right: Details Panel -->
        <div class="flex-1 overflow-auto bg-white">
          <DetailsPanel
            :file="aletheiaFile!"
            :raw-bytes="rawFileBytes!"
            :selected-node="selectedNode"
            :selected-range="selectedRange"
            :is-verified="isVerified"
            @hex-click="handleHexClick"
          />
        </div>
      </div>
    </div>
  </div>
</template>
