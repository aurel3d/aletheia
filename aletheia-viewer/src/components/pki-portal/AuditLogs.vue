<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface AuditLog {
  id: string
  action: string
  actor: string
  details: string
  timestamp: string
}

const props = defineProps<{
  apiBaseUrl: string
}>()

const logs = ref<AuditLog[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)
const selectedLog = ref<AuditLog | null>(null)

onMounted(fetchLogs)

async function fetchLogs() {
  isLoading.value = true
  error.value = null
  try {
    const response = await fetch(`${props.apiBaseUrl}/audit-logs`)
    if (!response.ok) throw new Error('Failed to fetch audit logs')
    logs.value = await response.json()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    isLoading.value = false
  }
}

function formatDate(dateString: string) {
  const date = new Date(dateString)
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

function getActionColor(action: string) {
  switch (action.toLowerCase()) {
    case 'create':
      return 'bg-green-100 text-green-800'
    case 'update':
      return 'bg-blue-100 text-blue-800'
    case 'delete':
      return 'bg-red-100 text-red-800'
    case 'revoke':
      return 'bg-red-100 text-red-800'
    default:
      return 'bg-slate-100 text-slate-800'
  }
}
</script>

<template>
  <div>
    <div class="mb-6">
      <h2 class="text-2xl font-bold text-slate-900">Audit Logs</h2>
      <p class="text-slate-600 mt-2">Track all PKI portal operations and changes</p>
    </div>

    <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6 text-red-700">
      {{ error }}
    </div>

    <div v-if="isLoading" class="text-center py-12">
      <div class="inline-block animate-spin">⏳</div>
      <p class="text-slate-600 mt-2">Loading audit logs...</p>
    </div>

    <div v-else-if="logs.length > 0" class="space-y-4">
      <div
        v-for="log in logs"
        :key="log.id"
        class="bg-white rounded-lg border border-slate-200 p-4 hover:shadow-md transition cursor-pointer"
        @click="selectedLog = selectedLog?.id === log.id ? null : log"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1">
            <div class="flex items-center gap-3 mb-2">
              <span :class="['px-3 py-1 rounded-full text-xs font-medium', getActionColor(log.action)]">
                {{ log.action.toUpperCase() }}
              </span>
              <span class="text-sm text-slate-600">{{ formatDate(log.timestamp) }}</span>
            </div>
            <p class="text-slate-900 font-medium">{{ log.details }}</p>
            <p class="text-sm text-slate-600 mt-1">By: <span class="font-mono">{{ log.actor }}</span></p>
          </div>
          <svg
            :class="[
              'h-5 w-5 text-slate-400 transition-transform',
              selectedLog?.id === log.id && 'rotate-180',
            ]"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </div>

        <!-- Expanded Details -->
        <div v-if="selectedLog?.id === log.id" class="mt-4 pt-4 border-t border-slate-200">
          <div class="bg-slate-50 rounded p-3 font-mono text-sm text-slate-700">
            <p><strong>ID:</strong> {{ log.id }}</p>
            <p class="mt-2"><strong>Action:</strong> {{ log.action }}</p>
            <p class="mt-2"><strong>Actor:</strong> {{ log.actor }}</p>
            <p class="mt-2"><strong>Timestamp:</strong> {{ log.timestamp }}</p>
            <p class="mt-2"><strong>Details:</strong></p>
            <p class="mt-1">{{ log.details }}</p>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-center py-12 bg-white rounded-lg border border-slate-200">
      <p class="text-slate-600">No audit logs found</p>
    </div>
  </div>
</template>
