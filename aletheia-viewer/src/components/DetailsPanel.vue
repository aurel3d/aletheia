<script setup lang="ts">
import { ref, computed } from 'vue'
import HexView from './HexView.vue'
import HeaderView from './HeaderView.vue'
import CertificateChainView from './CertificateChainView.vue'
import PayloadPreview from './PayloadPreview.vue'

interface Props {
  file: any // WASM parsed file
  rawBytes: Uint8Array // Raw file bytes for hex view
  selectedNode: string | null
  selectedRange: [number, number] | null
  isVerified: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  hexClick: [offset: number]
}>()

type Tab = 'hex' | 'header' | 'certificates' | 'payload'

const activeTab = ref<Tab>('hex')

const tabs = computed(() => [
  { id: 'hex' as Tab, label: 'Hex View', icon: '🔢' },
  { id: 'header' as Tab, label: 'Header', icon: '📄' },
  { id: 'certificates' as Tab, label: 'Certificates', icon: '🔐' },
  { id: 'payload' as Tab, label: 'Payload', icon: '📦', disabled: !props.isVerified },
])

function setActiveTab(tab: Tab) {
  if (tab === 'payload' && !props.isVerified) {
    return // Don't allow accessing payload if not verified
  }
  activeTab.value = tab
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Tabs -->
    <div class="border-b border-gray-200 bg-white">
      <nav class="flex" aria-label="Tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          :disabled="tab.disabled"
          :class="[
            'px-4 py-3 text-sm font-medium border-b-2 transition-colors',
            activeTab === tab.id
              ? 'border-blue-500 text-blue-600'
              : tab.disabled
                ? 'border-transparent text-gray-400 cursor-not-allowed'
                : 'border-transparent text-gray-600 hover:text-gray-800 hover:border-gray-300',
          ]"
          @click="setActiveTab(tab.id)"
        >
          <span class="mr-2">{{ tab.icon }}</span>
          {{ tab.label }}
          <span v-if="tab.disabled" class="ml-2 text-xs">(Verify first)</span>
        </button>
      </nav>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 overflow-hidden">
      <!-- Hex View -->
      <HexView
        v-if="activeTab === 'hex'"
        :raw-bytes="rawBytes"
        :selected-range="selectedRange"
        @click="emit('hexClick', $event)"
      />

      <!-- Header View -->
      <HeaderView
        v-if="activeTab === 'header'"
        :header="file.header"
      />

      <!-- Certificates View -->
      <CertificateChainView
        v-if="activeTab === 'certificates'"
        :certificates="file.certificateChain"
      />

      <!-- Payload Preview -->
      <PayloadPreview
        v-if="activeTab === 'payload'"
        :payload="file.payload"
        :is-compressed="file.isCompressed"
        :content-type="file.header.contentType"
        :is-verified="isVerified"
      />
    </div>
  </div>
</template>
