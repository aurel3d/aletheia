<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Policy {
  id: string
  policy_name: string
  policy_value: string
  created_at: string
  updated_at: string
}

const props = defineProps<{
  apiBaseUrl: string
}>()

const policies = ref<Policy[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)
const showForm = ref(false)
const newPolicy = ref({
  policy_name: '',
  policy_value: '',
})
const isSubmitting = ref(false)

onMounted(fetchPolicies)

async function fetchPolicies() {
  isLoading.value = true
  error.value = null
  try {
    const response = await fetch(`${props.apiBaseUrl}/policy`)
    if (!response.ok) throw new Error('Failed to fetch policies')
    const data = await response.json()
    // Policy endpoint returns an object, convert to array
    if (data && typeof data === 'object') {
      policies.value = Object.entries(data).map(([key, value]) => ({
        id: key,
        policy_name: key,
        policy_value: String(value),
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }))
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    isLoading.value = false
  }
}

async function handleSubmit() {
  if (!newPolicy.value.policy_name || !newPolicy.value.policy_value) {
    error.value = 'Please enter both policy name and value'
    return
  }

  isSubmitting.value = true
  error.value = null
  try {
    const payload = {
      [newPolicy.value.policy_name]: newPolicy.value.policy_value,
    }

    const response = await fetch(`${props.apiBaseUrl}/policy`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    })

    if (!response.ok) {
      const err = await response.json()
      throw new Error(err.error || 'Failed to create policy')
    }

    showForm.value = false
    newPolicy.value = { policy_name: '', policy_value: '' }
    await fetchPolicies()
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
      <h2 class="text-2xl font-bold text-slate-900">CA Policies</h2>
      <button
        @click="showForm = !showForm"
        class="px-4 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition"
      >
        {{ showForm ? '✕ Cancel' : '⚙️ Add Policy' }}
      </button>
    </div>

    <div v-if="showForm" class="bg-white rounded-lg p-6 mb-6 border border-slate-200">
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-2">
            Policy Name
          </label>
          <input
            v-model="newPolicy.policy_name"
            type="text"
            placeholder="e.g., max_cert_lifetime"
            class="block w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-2">
            Policy Value
          </label>
          <input
            v-model="newPolicy.policy_value"
            type="text"
            placeholder="e.g., 365"
            class="block w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>
        <div class="flex gap-3">
          <button
            type="submit"
            :disabled="!newPolicy.policy_name || !newPolicy.policy_value || isSubmitting"
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
      <p class="text-slate-600 mt-2">Loading policies...</p>
    </div>

    <div v-else-if="policies.length > 0" class="bg-white rounded-lg border border-slate-200 overflow-hidden">
      <table class="w-full">
        <thead class="bg-slate-50 border-b border-slate-200">
          <tr>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Policy Name</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Value</th>
            <th class="px-6 py-3 text-left text-sm font-semibold text-slate-900">Updated</th>
            <th class="px-6 py-3 text-right text-sm font-semibold text-slate-900">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="policy in policies" :key="policy.id" class="border-b border-slate-200 hover:bg-slate-50">
            <td class="px-6 py-4 text-sm font-medium text-slate-900">{{ policy.policy_name }}</td>
            <td class="px-6 py-4 text-sm font-mono text-slate-600">{{ policy.policy_value }}</td>
            <td class="px-6 py-4 text-sm text-slate-600">{{ formatDate(policy.updated_at) }}</td>
            <td class="px-6 py-4 text-right">
              <button class="text-blue-600 hover:text-blue-700 text-sm font-medium">
                Edit
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-else class="text-center py-12 bg-white rounded-lg border border-slate-200">
      <p class="text-slate-600">No policies configured</p>
    </div>
  </div>
</template>
