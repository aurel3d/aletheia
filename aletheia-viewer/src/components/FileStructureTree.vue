<script setup lang="ts">
import { computed } from 'vue'
import { formatBytes } from '../lib/utils'

interface Props {
  file: any // WASM parsed file
  selectedNode: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: [nodeId: string, range: [number, number]]
}>()

interface TreeNode {
  id: string
  label: string
  range: [number, number]
  size: number
  icon: string
}

const nodes = computed<TreeNode[]>(() => {
  if (!props.file) return []

  return [
    {
      id: 'magic',
      label: 'Magic Bytes',
      range: props.file.magicRange,
      size: props.file.magicRange[1] - props.file.magicRange[0],
      icon: '🔮',
    },
    {
      id: 'version',
      label: `Version ${props.file.versionMajor}.${props.file.versionMinor}`,
      range: props.file.versionRange,
      size: props.file.versionRange[1] - props.file.versionRange[0],
      icon: '📌',
    },
    {
      id: 'flags',
      label: 'Flags',
      range: props.file.flagsRange,
      size: props.file.flagsRange[1] - props.file.flagsRange[0],
      icon: '🚩',
    },
    {
      id: 'header',
      label: 'Header (CBOR)',
      range: props.file.headerRange,
      size: props.file.headerRange[1] - props.file.headerRange[0],
      icon: '📄',
    },
    {
      id: 'payload',
      label: `Payload${props.file.isCompressed ? ' (Compressed)' : ''}`,
      range: props.file.payloadRange,
      size: props.file.payloadRange[1] - props.file.payloadRange[0],
      icon: '📦',
    },
    {
      id: 'certificate_chain',
      label: `Certificate Chain (${props.file.certificateChain.length})`,
      range: props.file.certChainRange,
      size: props.file.certChainRange[1] - props.file.certChainRange[0],
      icon: '🔐',
    },
    {
      id: 'signature',
      label: 'Signature (Ed25519)',
      range: props.file.signatureRange,
      size: props.file.signatureRange[1] - props.file.signatureRange[0],
      icon: '✍️',
    },
  ]
})

function handleSelect(node: TreeNode) {
  emit('select', node.id, node.range)
}
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="p-4 border-b border-gray-200 bg-gray-50">
      <h3 class="text-lg font-bold text-gray-900">File Structure</h3>
      <p class="text-xs text-gray-600 mt-1">Click to inspect</p>
    </div>

    <div class="flex-1 overflow-auto p-2">
      <ul class="space-y-1">
        <li v-for="node in nodes" :key="node.id">
          <button
            type="button"
            :class="[
              'w-full text-left px-3 py-2 rounded-md transition-colors text-sm',
              selectedNode === node.id
                ? 'bg-blue-100 border-2 border-blue-500 text-blue-900'
                : 'hover:bg-gray-100 border-2 border-transparent',
            ]"
            @click="handleSelect(node)"
          >
            <div class="flex items-start gap-2">
              <span class="text-lg flex-shrink-0">{{ node.icon }}</span>
              <div class="flex-1 min-w-0">
                <div class="font-medium text-gray-900">{{ node.label }}</div>
                <div class="text-xs text-gray-600 mt-0.5">
                  <span class="font-mono">0x{{ node.range[0].toString(16).padStart(4, '0') }}</span>
                  -
                  <span class="font-mono">0x{{ node.range[1].toString(16).padStart(4, '0') }}</span>
                  <span class="ml-2">({{ formatBytes(node.size) }})</span>
                </div>
              </div>
            </div>
          </button>
        </li>
      </ul>
    </div>

    <div class="p-4 border-t border-gray-200 bg-gray-50 text-xs text-gray-600">
      Total: {{ formatBytes(file.signatureRange[1]) }}
    </div>
  </div>
</template>
