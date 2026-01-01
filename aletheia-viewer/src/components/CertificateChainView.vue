<script setup lang="ts">
import { ref, computed } from 'vue'
import { formatTimestamp, bytesToHex, copyToClipboard } from '../lib/utils'

interface Props {
  certificates: any[] // WASM certificate array
}

const props = defineProps<Props>()

const expandedCerts = ref<Set<number>>(new Set([0])) // Expand first cert by default

interface CertDisplay {
  index: number
  cert: any
  role: string
  fingerprint: string | null
}

const certDisplays = computed<CertDisplay[]>(() => {
  return props.certificates.map((cert, idx) => {
    let role = 'Intermediate'
    if (idx === 0) {
      role = 'Creator'
    } else if (cert.isCa && cert.subjectId === cert.issuerId) {
      role = 'Root CA'
    } else if (cert.isCa) {
      role = 'Intermediate CA'
    }

    return {
      index: idx,
      cert,
      role,
      fingerprint: null, // Will compute on demand
    }
  })
})

function toggleCert(index: number) {
  if (expandedCerts.value.has(index)) {
    expandedCerts.value.delete(index)
  } else {
    expandedCerts.value.add(index)
  }
}

async function copyValue(value: string) {
  try {
    await copyToClipboard(value)
  } catch (error) {
    console.error('Failed to copy:', error)
  }
}

function toUint8Array(data: any): Uint8Array {
  return new Uint8Array(data)
}
</script>

<template>
  <div class="h-full overflow-auto bg-gray-50">
    <div class="p-6 max-w-5xl mx-auto">
      <div class="mb-6">
        <h3 class="text-xl font-bold text-gray-900">Certificate Chain</h3>
        <p class="text-sm text-gray-600 mt-1">
          {{ certificates.length }} certificate{{ certificates.length > 1 ? 's' : '' }} in chain
        </p>
      </div>

      <!-- Chain visualization -->
      <div class="relative">
        <!-- Vertical line -->
        <div class="absolute left-6 top-8 bottom-8 w-0.5 bg-gray-300"></div>

        <!-- Certificates -->
        <div class="space-y-6">
          <div
            v-for="display in certDisplays"
            :key="display.index"
            class="relative"
          >
            <!-- Circle marker -->
            <div
              :class="[
                'absolute left-3 top-6 w-6 h-6 rounded-full border-4 z-10',
                display.role === 'Root CA'
                  ? 'bg-green-500 border-green-200'
                  : display.role === 'Creator'
                    ? 'bg-blue-500 border-blue-200'
                    : 'bg-gray-400 border-gray-200',
              ]"
            ></div>

            <!-- Certificate card -->
            <div class="ml-16 bg-white rounded-lg shadow-sm border border-gray-200">
              <!-- Header -->
              <button
                type="button"
                class="w-full p-4 text-left hover:bg-gray-50 transition-colors"
                @click="toggleCert(display.index)"
              >
                <div class="flex items-start justify-between gap-4">
                  <div class="flex-1">
                    <div class="flex items-center gap-2">
                      <span
                        :class="[
                          'px-2 py-0.5 text-xs font-semibold rounded',
                          display.role === 'Root CA'
                            ? 'bg-green-100 text-green-800'
                            : display.role === 'Creator'
                              ? 'bg-blue-100 text-blue-800'
                              : 'bg-gray-100 text-gray-800',
                        ]"
                      >
                        {{ display.role }}
                      </span>
                      <span v-if="display.cert.isCa" class="text-xs text-gray-500">
                        (Certificate Authority)
                      </span>
                    </div>
                    <div class="mt-2">
                      <div class="font-bold text-gray-900">
                        {{ display.cert.subjectName }}
                      </div>
                      <div class="text-sm text-gray-600 font-mono">
                        {{ display.cert.subjectId }}
                      </div>
                    </div>
                  </div>

                  <svg
                    :class="[
                      'w-5 h-5 text-gray-400 transition-transform',
                      expandedCerts.has(display.index) ? 'transform rotate-180' : '',
                    ]"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                  </svg>
                </div>
              </button>

              <!-- Expanded details -->
              <div v-if="expandedCerts.has(display.index)" class="border-t border-gray-200 p-4 space-y-4">
                <!-- Issuer -->
                <div>
                  <div class="text-sm font-semibold text-gray-600 mb-1">Issued By</div>
                  <div class="text-sm text-gray-900 font-mono">{{ display.cert.issuerId }}</div>
                </div>

                <!-- Issued At -->
                <div>
                  <div class="text-sm font-semibold text-gray-600 mb-1">Issued At</div>
                  <div class="text-sm text-gray-900">{{ formatTimestamp(display.cert.issuedAt) }}</div>
                  <div class="text-xs text-gray-500 mt-0.5">
                    Unix: {{ display.cert.issuedAt }}
                  </div>
                </div>

                <!-- Serial -->
                <div>
                  <div class="text-sm font-semibold text-gray-600 mb-1">Serial Number</div>
                  <div class="flex items-center gap-2">
                    <code class="text-xs bg-gray-100 px-2 py-1 rounded font-mono">
                      {{ bytesToHex(toUint8Array(display.cert.serial)).substring(0, 32) }}...
                    </code>
                    <button
                      type="button"
                      class="p-1 text-gray-400 hover:text-gray-600"
                      @click="copyValue(bytesToHex(toUint8Array(display.cert.serial)))"
                    >
                      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                      </svg>
                    </button>
                  </div>
                </div>

                <!-- Public Key -->
                <div>
                  <div class="text-sm font-semibold text-gray-600 mb-1">Public Key (Ed25519)</div>
                  <div class="flex items-start gap-2">
                    <code class="text-xs bg-gray-100 px-2 py-1 rounded font-mono break-all flex-1">
                      {{ bytesToHex(toUint8Array(display.cert.publicKey)) }}
                    </code>
                    <button
                      type="button"
                      class="p-1 text-gray-400 hover:text-gray-600 flex-shrink-0"
                      @click="copyValue(bytesToHex(toUint8Array(display.cert.publicKey)))"
                    >
                      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                      </svg>
                    </button>
                  </div>
                </div>

                <!-- Fingerprint (computed) -->
                <div>
                  <div class="text-sm font-semibold text-gray-600 mb-1">SHA-256 Fingerprint</div>
                  <div class="text-xs text-gray-500 italic">
                    Click "Show" to compute fingerprint
                  </div>
                </div>

                <!-- Signature -->
                <div>
                  <div class="text-sm font-semibold text-gray-600 mb-1">Signature</div>
                  <code class="text-xs bg-gray-100 px-2 py-1 rounded font-mono break-all block">
                    {{ bytesToHex(toUint8Array(display.cert.signature)).substring(0, 64) }}...
                  </code>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
