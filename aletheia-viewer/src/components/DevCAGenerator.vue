<script setup lang="ts">
import { ref } from 'vue'
import { generate_root_ca } from '../lib/wasm-pkg/aletheia.js'
import { downloadFile } from '../lib/utils'

const subjectId = ref('dev@localhost')
const subjectName = ref('Development CA')

const isGenerating = ref(false)
const error = ref<string | null>(null)
const generatedCA = ref<{
  privateKeyHex: string
  certificateBase64: string
  subjectId: string
  subjectName: string
} | null>(null)

async function generateCA() {
  if (!subjectId.value || !subjectName.value) {
    error.value = 'Please fill in both Subject ID and Subject Name'
    return
  }

  isGenerating.value = true
  error.value = null
  generatedCA.value = null

  try {
    const result = generate_root_ca(subjectId.value, subjectName.value)
    generatedCA.value = result
    console.log('✅ CA generated:', result.subjectId, result.subjectName)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    console.error('CA generation error:', e)
  } finally {
    isGenerating.value = false
  }
}

function downloadPrivateKey() {
  if (!generatedCA.value) return
  const encoder = new TextEncoder()
  const bytes = encoder.encode(generatedCA.value.privateKeyHex)
  downloadFile(new Uint8Array(bytes), 'ca.key')
}

function downloadCertificate() {
  if (!generatedCA.value) return
  const encoder = new TextEncoder()
  const bytes = encoder.encode(generatedCA.value.certificateBase64)
  downloadFile(new Uint8Array(bytes), 'ca.cert')
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text)
}
</script>

<template>
  <div class="min-h-screen bg-gray-900 text-white p-8">
    <div class="max-w-2xl mx-auto">
      <!-- Warning Banner -->
      <div class="bg-red-900 border-2 border-red-500 rounded-lg p-4 mb-8">
        <div class="flex items-start gap-3">
          <svg class="h-8 w-8 text-red-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <div>
            <h2 class="text-xl font-bold text-red-300">⚠️ DEVELOPMENT ONLY</h2>
            <p class="text-red-200 mt-1">
              This page generates Certificate Authority keys in the browser. 
              <strong>Never use browser-generated CAs in production!</strong>
              For production, generate CAs offline using the CLI tool.
            </p>
          </div>
        </div>
      </div>

      <!-- Header -->
      <h1 class="text-3xl font-bold mb-2">Generate Root CA</h1>
      <p class="text-gray-400 mb-8">Create a new Certificate Authority for development and testing.</p>

      <!-- Form -->
      <div class="bg-gray-800 rounded-lg p-6 mb-6">
        <div class="space-y-4">
          <div>
            <label for="subjectId" class="block text-sm font-medium text-gray-300 mb-1">
              Subject ID (e.g., email)
            </label>
            <input
              id="subjectId"
              v-model="subjectId"
              type="text"
              placeholder="you@example.com"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label for="subjectName" class="block text-sm font-medium text-gray-300 mb-1">
              Subject Name (display name)
            </label>
            <input
              id="subjectName"
              v-model="subjectName"
              type="text"
              placeholder="My Development CA"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <button
            type="button"
            :disabled="isGenerating"
            class="w-full py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 rounded-md font-semibold transition-colors"
            @click="generateCA"
          >
            <span v-if="isGenerating">Generating...</span>
            <span v-else>Generate Root CA</span>
          </button>
        </div>
      </div>

      <!-- Error -->
      <div v-if="error" class="bg-red-900/50 border border-red-700 rounded-lg p-4 mb-6">
        <p class="text-red-300">{{ error }}</p>
      </div>

      <!-- Result -->
      <div v-if="generatedCA" class="bg-green-900/30 border border-green-700 rounded-lg p-6">
        <div class="flex items-center gap-2 mb-4">
          <svg class="h-6 w-6 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <h3 class="text-xl font-bold text-green-300">CA Generated Successfully</h3>
        </div>

        <div class="space-y-4">
          <!-- CA Info -->
          <div class="bg-gray-800 rounded p-3">
            <div class="text-sm text-gray-400">Identity</div>
            <div class="font-mono">{{ generatedCA.subjectName }}</div>
            <div class="text-sm text-gray-400">{{ generatedCA.subjectId }}</div>
          </div>

          <!-- Private Key -->
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="text-sm font-medium text-gray-300">Private Key (ca.key)</label>
              <div class="flex gap-2">
                <button
                  type="button"
                  class="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded"
                  @click="copyToClipboard(generatedCA.privateKeyHex)"
                >
                  Copy
                </button>
                <button
                  type="button"
                  class="px-3 py-1 text-sm bg-blue-600 hover:bg-blue-700 rounded"
                  @click="downloadPrivateKey"
                >
                  Download
                </button>
              </div>
            </div>
            <div class="bg-gray-800 rounded p-3 font-mono text-xs break-all text-yellow-300">
              {{ generatedCA.privateKeyHex }}
            </div>
            <p class="text-xs text-red-400 mt-1">🔒 Keep this secret! Anyone with this key can sign files as this CA.</p>
          </div>

          <!-- Certificate -->
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="text-sm font-medium text-gray-300">Certificate (ca.cert)</label>
              <div class="flex gap-2">
                <button
                  type="button"
                  class="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded"
                  @click="copyToClipboard(generatedCA.certificateBase64)"
                >
                  Copy
                </button>
                <button
                  type="button"
                  class="px-3 py-1 text-sm bg-blue-600 hover:bg-blue-700 rounded"
                  @click="downloadCertificate"
                >
                  Download
                </button>
              </div>
            </div>
            <div class="bg-gray-800 rounded p-3 font-mono text-xs break-all text-green-300 max-h-32 overflow-auto">
              {{ generatedCA.certificateBase64 }}
            </div>
            <p class="text-xs text-gray-400 mt-1">📜 This can be shared publicly. Use it as a trusted root for verification.</p>
          </div>
        </div>

        <!-- Next Steps -->
        <div class="mt-6 p-4 bg-gray-800 rounded">
          <h4 class="font-semibold mb-2">Next Steps:</h4>
          <ol class="list-decimal list-inside text-sm text-gray-300 space-y-1">
            <li>Download both files (ca.key and ca.cert)</li>
            <li>Go to the <a href="/" class="text-blue-400 hover:underline">Sign Files</a> tab</li>
            <li>Load the ca.key and ca.cert files</li>
            <li>Select a file to sign</li>
          </ol>
        </div>
      </div>

      <!-- Back Link -->
      <div class="mt-8 text-center">
        <a href="/" class="text-blue-400 hover:underline">← Back to Aletheia Viewer</a>
      </div>
    </div>
  </div>
</template>
