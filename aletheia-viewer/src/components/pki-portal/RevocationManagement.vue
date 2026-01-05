<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Revocation {
  id: string
  certificate_id: string
  serial_number: string
  revocation_date: string
  reason: string
  created_at: string
}

const props = defineProps<{
  apiBaseUrl: string
}>()

const revocations = ref<Revocation[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)
const showForm = ref(false)
const serialNumber = ref('')
const reason = ref('')
const isSubmitting = ref(false)

onMounted(fetchRevocations)

async function fetchRevocations() {
  isLoading.value = true
  error.value = null
  try {
    const response = await fetch(`${props.apiBaseUrl}/revocations`)
    if (!response.ok) throw new Error('Failed to fetch revocations')
    revocations.value = await response.json()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    isLoading.value = false
  }
}

async function handleSubmit() {
  if (!serialNumber.value) {
    error.value = 'Please enter a serial number'
    return
  }

  isSubmitting.value = true
  error.value = null
  try {
    const response = await fetch(`${props.apiBaseUrl}/revocations`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        certificate_id: serialNumber.value,
        reason: reason.value || 'Unspecified',
      }),
    })

    if (!response.ok) {
      const err = await response.json()
      throw new Error(err.error || 'Failed to revoke certificate')
    }

    showForm.value = false
    serialNumber.value = ''
    reason.value = ''
    await fetchRevocations()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    isSubmitting.value = false
  }
}

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-2xl font-bold text-slate-900">Certificate Revocations</h2>
      <button
        @click="showForm = !showForm"
        class="px-4 py-2 bg-red-600 text-white rounded-lg font-medium hover:bg-red-700 transition"
      >
        {{ showForm ? '✕ Cancel' : '⛔ Revoke Certificate' }}
      </button>
    </div>

    <div v-if="showForm" class="bg-white rounded-lg p-6 mb-6 border border-slate-200">
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-2">
            Certificate Serial Number
          </label>
          <input
            v-model="serialNumber"
            type="text"
            placeholder="e.g., 123456789..."
            class="block w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-red-500 focus:border-transparent"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-2">
            Revocation Reason (optional)
          </label>
          <select
            v-model="reason"
            class="block w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-red-500 focus:border-transparent"
          >
            <option value="">Unspecified</option>
            <option value="keyCompromise">Key Compromise</option>
            <option value="caCompromise">CA Compromise</option>
            <option value="affiliationChanged">Affiliation Changed</option>
            <option value="superseded">Superseded</option>
            <option value="cessationOfOperation">Cessation of Operation</option>
          </select>
        </div>
        <div class="flex gap-3">
          <button
            type="submit"
            :disabled="!serialNumber || isSubmitting"
            class="px-4 py-2 bg-red-600 text-white rounded-lg font-medium hover:bg-red-700 transition disabled:opacity-50"
          >
            {{ isSubmitting ? 'Revoking...' : 'Revoke' }}
          </button>
        </div>
      </form>
    </div>

    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6 text-red-700">
      {{ error }}
    </div>

    <div v-if="isLoading" class="text-center py-12">
      <div class="inline-block animate-spin">⏳</div>
      <p class="text-slate-600 mt-2">Loading revocations...</p>
    </div>

    <div v-else-if="revocations.length > 0" class="bg-white rounded-lg border border-slate-200 overflow-hidden">
      <table class="w-full">
        <thead class="bg-slate-50 border-b border-slate-200">
          <tr>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Serial Number</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Reason</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Revoked On</th>
            <th class="px-6 py-3 text-right text-sm font-semibold text-slate-900">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="revocation in revocations" :key="revocation.id" class="border-b border-slate-200 hover:bg-slate-50">
            <td class="px-6 py-4 text-sm font-mono text-slate-900">
              {{ revocation.serial_number.substring(0, 16) }}...
            </td>
            <td class="px-6 py-4 text-sm text-slate-600">{{ revocation.reason }}</td>
            <td class="px-6 py-4 text-sm text-slate-600">{{ formatDate(revocation.revocation_date) }}</td>
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
      <p class="text-slate-600">No revocations found</p>
    </div>
  </div>
</template>
