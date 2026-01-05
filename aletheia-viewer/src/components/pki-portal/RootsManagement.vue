<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Root {
  id: string
  subject: string
  issuer: string
  fingerprint: string
  not_before: string
  not_after: string
  created_at: string
}

const props = defineProps<{
  apiBaseUrl: string
}>()

const roots = ref<Root[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)
const showForm = ref(false)
const newCertFile = ref<File | null>(null)
const isSubmitting = ref(false)

onMounted(fetchRoots)

async function fetchRoots() {
  isLoading.value = true
  error.value = null
  try {
    const response = await fetch(`${props.apiBaseUrl}/roots`)
    if (!response.ok) throw new Error('Failed to fetch roots')
    roots.value = await response.json()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    isLoading.value = false
  }
}

async function handleSubmit() {
  if (!newCertFile.value) {
    error.value = 'Please select a certificate file'
    return
  }

  isSubmitting.value = true
  error.value = null
  try {
    const formData = new FormData()
    formData.append('certificate', newCertFile.value)

    const response = await fetch(`${props.apiBaseUrl}/roots`, {
      method: 'POST',
      body: formData,
    })

    if (!response.ok) {
      const err = await response.json()
      throw new Error(err.error || 'Failed to upload root certificate')
    }

    showForm.value = false
    newCertFile.value = null
    await fetchRoots()
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
    <!-- Header with Add Button -->
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-2xl font-bold text-slate-900">Root Certificates</h2>
      <button
        @click="showForm = !showForm"
        class="px-4 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition"
      >
        {{ showForm ? '✕ Cancel' : '+ Add Root' }}
      </button>
    </div>

    <!-- Add Form -->
    <div v-if="showForm" class="bg-white rounded-lg p-6 mb-6 border border-slate-200">
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-2">
            Certificate File (.pem or .der)
          </label>
          <input
            type="file"
            accept=".pem,.der,.crt"
            @change="newCertFile = ($event.target as HTMLInputElement).files?.[0] || null"
            class="block w-full text-sm text-slate-600
              file:mr-4 file:py-2 file:px-4
              file:rounded-lg file:border-0
              file:text-sm file:font-medium
              file:bg-blue-50 file:text-blue-700
              hover:file:bg-blue-100"
          />
        </div>
        <div class="flex gap-3">
          <button
            type="submit"
            :disabled="!newCertFile || isSubmitting"
            class="px-4 py-2 bg-green-600 text-white rounded-lg font-medium hover:bg-green-700 transition disabled:opacity-50"
          >
            {{ isSubmitting ? 'Uploading...' : 'Upload' }}
          </button>
        </div>
      </form>
    </div>

    <!-- Error -->
    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6 text-red-700">
      {{ error }}
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="text-center py-12">
      <div class="inline-block animate-spin">⏳</div>
      <p class="text-slate-600 mt-2">Loading root certificates...</p>
    </div>

    <!-- Table -->
    <div v-else-if="roots.length > 0" class="bg-white rounded-lg border border-slate-200 overflow-hidden">
      <table class="w-full">
        <thead class="bg-slate-50 border-b border-slate-200">
          <tr>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Subject</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Valid Until</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Fingerprint</th>
            <th class="px-6 py-3 text-right text-sm font-semibold text-slate-900">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="root in roots" :key="root.id" class="border-b border-slate-200 hover:bg-slate-50">
            <td class="px-6 py-4 text-sm text-slate-900 font-medium">{{ root.subject }}</td>
            <td class="px-6 py-4 text-sm text-slate-600">{{ formatDate(root.not_after) }}</td>
            <td class="px-6 py-4 text-sm font-mono text-slate-600">
              {{ root.fingerprint.substring(0, 16) }}...
            </td>
            <td class="px-6 py-4 text-right">
              <button class="text-blue-600 hover:text-blue-700 text-sm font-medium">
                View
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty State -->
    <div v-else class="text-center py-12 bg-white rounded-lg border border-slate-200">
      <p class="text-slate-600">No root certificates found</p>
    </div>
  </div>
</template>
