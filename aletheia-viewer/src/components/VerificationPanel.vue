<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  verificationResult: any | null
  isVerifying: boolean
  hasTrustedRoots: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  verify: []
}>()

const statusColor = computed(() => {
  if (!props.verificationResult) return 'bg-gray-100 border-gray-300'
  return props.verificationResult.isValid
    ? 'bg-green-50 border-green-300'
    : 'bg-red-50 border-red-300'
})

const statusText = computed(() => {
  if (!props.verificationResult) return 'NOT VERIFIED'
  return props.verificationResult.isValid ? 'VERIFIED ✓' : 'NOT VERIFIED ✗'
})

const statusTextColor = computed(() => {
  if (!props.verificationResult) return 'text-gray-700'
  return props.verificationResult.isValid ? 'text-green-900' : 'text-red-900'
})
</script>

<template>
  <div class="border-b border-gray-200 bg-white px-6 py-4">
    <!-- Status Banner -->
    <div :class="[statusColor, 'border-2 rounded-lg p-4 mb-4']">
      <div class="flex items-center justify-between">
        <h2 :class="[statusTextColor, 'text-2xl font-bold']">{{ statusText }}</h2>
        <button
          v-if="!isVerifying && hasTrustedRoots"
          type="button"
          class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 font-medium"
          @click="emit('verify')"
        >
          Verify
        </button>
        <div v-else-if="isVerifying" class="text-gray-600">
          Verifying...
        </div>
        <div v-else class="text-sm text-gray-600">
          Load trusted roots to verify
        </div>
      </div>
    </div>

    <!-- Verification Steps -->
    <div v-if="verificationResult" class="space-y-2">
      <h3 class="text-sm font-semibold text-gray-700 mb-2">Verification Steps:</h3>
      <div
        v-for="step in verificationResult.steps"
        :key="step.id"
        class="flex items-start gap-3 text-sm"
      >
        <!-- Status Icon -->
        <div class="flex-shrink-0 mt-0.5">
          <svg
            v-if="step.status === 'success'"
            class="h-5 w-5 text-green-600"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <svg
            v-else-if="step.status === 'error'"
            class="h-5 w-5 text-red-600"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <svg
            v-else
            class="h-5 w-5 text-gray-400"
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
        </div>

        <!-- Step Info -->
        <div class="flex-1">
          <div :class="step.status === 'error' ? 'text-red-900 font-medium' : 'text-gray-700'">
            {{ step.label }}
          </div>
          <div v-if="step.error" class="text-red-700 text-xs mt-1 font-mono">
            {{ step.error }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
