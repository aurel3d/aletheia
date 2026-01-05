<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface TrustBundle {
  id: string
  name: string
  description: string
  bundle_data: string
  created_at: string
  updated_at: string
}

const props = defineProps<{
  apiBaseUrl: string
}>()

const bundles = ref<TrustBundle[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)
const showForm = ref(false)
const newBundle = ref({
  name: '',
  description: '',
})
const isSubmitting = ref(false)

onMounted(fetchBundles)

async function fetchBundles() {
  isLoading.value = true
  error.value = null
  try {
    const response = await fetch(`${props.apiBaseUrl}/trust-bundles`)
    if (!response.ok) throw new Error('Failed to fetch trust bundles')
    bundles.value = await response.json()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    isLoading.value = false
  }
}

async function handleSubmit() {
  if (!newBundle.value.name) {
    error.value = 'Please enter a bundle name'
    return
  }

  isSubmitting.value = true
  error.value = null
  try {
    const response = await fetch(`${props.apiBaseUrl}/trust-bundles`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newBundle.value),
    })

    if (!response.ok) {
      const err = await response.json()
      throw new Error(err.error || 'Failed to create trust bundle')
    }

    showForm.value = false
    newBundle.value = { name: '', description: '' }
    await fetchBundles()
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
      <h2 class="text-2xl font-bold text-slate-900">Trust Bundles</h2>
      <button
        @click="showForm = !showForm"
        class="px-4 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition"
      >
        {{ showForm ? '✕ Cancel' : '📦 Create Bundle' }}
      </button>
    </div>

    <div v-if="showForm" class="bg-white rounded-lg p-6 mb-6 border border-slate-200">
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-2">
            Bundle Name
          </label>
          <input
            v-model="newBundle.name"
            type="text"
            placeholder="e.g., Public Roots 2026"
            class="block w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-2">
            Description (optional)
          </label>
          <textarea
            v-model="newBundle.description"
            placeholder="Describe this trust bundle..."
            rows="3"
            class="block w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          ></textarea>
        </div>
        <div class="flex gap-3">
          <button
            type="submit"
            :disabled="!newBundle.name || isSubmitting"
            class="px-4 py-2 bg-green-600 text-white rounded-lg font-medium hover:bg-green-700 transition disabled:opacity-50"
          >
            {{ isSubmitting ? 'Creating...' : 'Create' }}
          </button>
        </div>
      </form>
    </div>

    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6 text-red-700">
      {{ error }}
    </div>

    <div v-if="isLoading" class="text-center py-12">
      <div class="inline-block animate-spin">⏳</div>
      <p class="text-slate-600 mt-2">Loading trust bundles...</p>
    </div>

    <div v-else-if="bundles.length > 0" class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div
        v-for="bundle in bundles"
        :key="bundle.id"
        class="bg-white rounded-lg border border-slate-200 p-6 hover:shadow-lg transition"
      >
        <h3 class="text-lg font-semibold text-slate-900">{{ bundle.name }}</h3>
        <p class="text-slate-600 text-sm mt-2">{{ bundle.description }}</p>
        <div class="mt-4 pt-4 border-t border-slate-200 flex items-center justify-between">
          <span class="text-xs text-slate-500">{{ formatDate(bundle.created_at) }}</span>
          <button class="text-blue-600 hover:text-blue-700 text-sm font-medium">
            View
          </button>
        </div>
      </div>
    </div>

    <div v-else class="text-center py-12 bg-white rounded-lg border border-slate-200">
      <p class="text-slate-600">No trust bundles found</p>
    </div>
  </div>
</template>
