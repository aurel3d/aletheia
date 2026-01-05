<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Certificate {
  id: string
  subject: string
  serial_number: string
  fingerprint: string
  not_before: string
  not_after: string
  created_at: string
  is_revoked: boolean
}

const props = defineProps<{
  apiBaseUrl: string
}>()

const certificates = ref<Certificate[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)

onMounted(fetchCertificates)

async function fetchCertificates() {
  isLoading.value = true
  error.value = null
  try {
    const response = await fetch(`${props.apiBaseUrl}/certificates`)
    if (!response.ok) throw new Error('Failed to fetch certificates')
    certificates.value = await response.json()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    isLoading.value = false
  }
}

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

function isExpired(notAfter: string): boolean {
  return new Date(notAfter) < new Date()
}
</script>

<template>
  <div>
    <div class="mb-6">
      <h2 class="text-2xl font-bold text-slate-900">Issued Certificates</h2>
      <p class="text-slate-600 mt-2">View and manage all issued certificates</p>
    </div>

    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6 text-red-700">
      {{ error }}
    </div>

    <div v-if="isLoading" class="text-center py-12">
      <div class="inline-block animate-spin">⏳</div>
      <p class="text-slate-600 mt-2">Loading certificates...</p>
    </div>

    <div v-else-if="certificates.length > 0" class="bg-white rounded-lg border border-slate-200 overflow-hidden">
      <table class="w-full">
        <thead class="bg-slate-50 border-b border-slate-200">
          <tr>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Subject</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Serial</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Status</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Valid Until</th>
            <th class="px-6 py-3 text-right text-sm font-semibold text-slate-900">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="cert in certificates" :key="cert.id" class="border-b border-slate-200 hover:bg-slate-50">
            <td class="px-6 py-4 text-sm text-slate-900 font-medium">{{ cert.subject }}</td>
            <td class="px-6 py-4 text-sm font-mono text-slate-600">
              {{ cert.serial_number.substring(0, 8) }}...
            </td>
            <td class="px-6 py-4 text-sm">
              <span
                v-if="cert.is_revoked"
                class="px-3 py-1 bg-red-100 text-red-800 rounded-full text-xs font-medium"
              >
                Revoked
              </span>
              <span
                v-else-if="isExpired(cert.not_after)"
                class="px-3 py-1 bg-yellow-100 text-yellow-800 rounded-full text-xs font-medium"
              >
                Expired
              </span>
              <span v-else class="px-3 py-1 bg-green-100 text-green-800 rounded-full text-xs font-medium">
                Valid
              </span>
            </td>
            <td class="px-6 py-4 text-sm text-slate-600">{{ formatDate(cert.not_after) }}</td>
            <td class="px-6 py-4 text-right">
              <button class="text-blue-600 hover:text-blue-700 text-sm font-medium">
                View
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-else class="text-center py-12 bg-white rounded-lg border border-slate-200">
      <p class="text-slate-600">No certificates found</p>
    </div>
  </div>
</template>
