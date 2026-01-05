<script setup lang="ts">
import { ref, onMounted } from 'vue'
import CertificateManagement from './pki-portal/CertificateManagement.vue'
import RootsManagement from './pki-portal/RootsManagement.vue'
import IntermediatesManagement from './pki-portal/IntermediatesManagement.vue'
import RevocationManagement from './pki-portal/RevocationManagement.vue'
import TrustBundlesManagement from './pki-portal/TrustBundlesManagement.vue'
import PolicyManagement from './pki-portal/PolicyManagement.vue'
import AuditLogs from './pki-portal/AuditLogs.vue'

type ActiveTab = 'roots' | 'intermediates' | 'certificates' | 'revocations' | 'trust-bundles' | 'policy' | 'audit'

const activeTab = ref<ActiveTab>('roots')
const apiBaseUrl = ref(import.meta.env.VITE_API_URL || 'http://localhost:8080')
const isConnected = ref(false)
const connectionError = ref<string | null>(null)

// Check API connectivity on mount
onMounted(async () => {
  try {
    const response = await fetch(`${apiBaseUrl.value}/health`)
    isConnected.value = response.ok
  } catch (error) {
    isConnected.value = false
    connectionError.value = `Cannot connect to PKI Portal API at ${apiBaseUrl.value}`
  }
})

const tabs = [
  { id: 'roots', name: 'Root Certificates', icon: '🔐' },
  { id: 'intermediates', name: 'Intermediates', icon: '🔗' },
  { id: 'certificates', name: 'Certificates', icon: '📜' },
  { id: 'revocations', name: 'Revocations', icon: '⛔' },
  { id: 'trust-bundles', name: 'Trust Bundles', icon: '📦' },
  { id: 'policy', name: 'Policy', icon: '⚙️' },
  { id: 'audit', name: 'Audit Logs', icon: '📋' },
]
</script>

<template>
  <div class="h-screen bg-gradient-to-br from-slate-50 to-slate-100">
    <!-- Header -->
    <div class="bg-white border-b border-slate-200 shadow-sm">
      <div class="max-w-7xl mx-auto px-6 py-6">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-4xl font-bold text-slate-900">PKI Portal</h1>
            <p class="text-slate-600 mt-2">Certificate Authority Management System</p>
          </div>
          <div class="flex items-center gap-3">
            <div :class="['h-3 w-3 rounded-full', isConnected ? 'bg-green-500' : 'bg-red-500']"></div>
            <span :class="isConnected ? 'text-green-600' : 'text-red-600'" class="font-medium">
              {{ isConnected ? 'Connected' : 'Disconnected' }}
            </span>
          </div>
        </div>
        <p v-if="connectionError" class="mt-4 text-sm text-red-600">
          {{ connectionError }}
        </p>
      </div>
    </div>

    <!-- Navigation Tabs -->
    <div class="bg-white border-b border-slate-200 sticky top-0 z-10">
      <div class="max-w-7xl mx-auto px-6">
        <div class="flex gap-8 overflow-x-auto">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            @click="activeTab = tab.id as ActiveTab"
            :class="[
              'px-4 py-4 text-sm font-medium border-b-2 whitespace-nowrap transition-colors',
              activeTab === tab.id
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-slate-600 hover:text-slate-900'
            ]"
          >
            <span class="mr-2">{{ tab.icon }}</span>{{ tab.name }}
          </button>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="max-w-7xl mx-auto px-6 py-8">
      <template v-if="!isConnected">
        <div class="bg-red-50 border border-red-200 rounded-lg p-8 text-center">
          <svg class="mx-auto h-12 w-12 text-red-600 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4v2m0 0v2m0-6v-2m0 0V7a2 2 0 012-2h.5a4.5 4.5 0 100-9H12a2 2 0 00-2 2v.5" />
          </svg>
          <h3 class="text-lg font-medium text-red-900">Unable to Connect</h3>
          <p class="text-red-700 mt-2">Please ensure the PKI Portal API is running at {{ apiBaseUrl }}</p>
        </div>
      </template>

      <template v-else>
        <!-- Root Certificates -->
        <RootsManagement v-if="activeTab === 'roots'" :api-base-url="apiBaseUrl" />

        <!-- Intermediates -->
        <IntermediatesManagement v-if="activeTab === 'intermediates'" :api-base-url="apiBaseUrl" />

        <!-- Certificates -->
        <CertificateManagement v-if="activeTab === 'certificates'" :api-base-url="apiBaseUrl" />

        <!-- Revocations -->
        <RevocationManagement v-if="activeTab === 'revocations'" :api-base-url="apiBaseUrl" />

        <!-- Trust Bundles -->
        <TrustBundlesManagement v-if="activeTab === 'trust-bundles'" :api-base-url="apiBaseUrl" />

        <!-- Policy -->
        <PolicyManagement v-if="activeTab === 'policy'" :api-base-url="apiBaseUrl" />

        <!-- Audit Logs -->
        <AuditLogs v-if="activeTab === 'audit'" :api-base-url="apiBaseUrl" />
      </template>
    </div>
  </div>
</template>

<style scoped>
/* Smooth scrolling for tabs */
::-webkit-scrollbar {
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: #94a3b8;
}
</style>
