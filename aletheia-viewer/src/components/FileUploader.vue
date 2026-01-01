<script setup lang="ts">
import { ref, computed } from 'vue'

interface Props {
  label: string
  accept?: string
  multiple?: boolean
  variant?: 'primary' | 'secondary'
}

const props = withDefaults(defineProps<Props>(), {
  accept: '*',
  multiple: false,
  variant: 'primary',
})

const emit = defineEmits<{
  load: [files: File[]]
}>()

const fileInput = ref<HTMLInputElement | null>(null)
const isDragging = ref(false)

function handleClick() {
  fileInput.value?.click()
}

function handleFileSelect(event: Event) {
  const target = event.target as HTMLInputElement
  if (target.files && target.files.length > 0) {
    emit('load', Array.from(target.files))
    target.value = '' // Reset input
  }
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  isDragging.value = true
}

function handleDragLeave() {
  isDragging.value = false
}

function handleDrop(event: DragEvent) {
  event.preventDefault()
  isDragging.value = false

  const files = event.dataTransfer?.files
  if (files && files.length > 0) {
    emit('load', Array.from(files))
  }
}

const buttonClasses = computed(() => {
  const base = 'px-4 py-2 rounded-md font-medium transition-colors'
  if (props.variant === 'primary') {
    return `${base} bg-blue-600 text-white hover:bg-blue-700`
  } else {
    return `${base} bg-gray-200 text-gray-800 hover:bg-gray-300`
  }
})
</script>

<template>
  <div>
    <input
      ref="fileInput"
      type="file"
      :accept="accept"
      :multiple="multiple"
      class="hidden"
      @change="handleFileSelect"
    />
    <button
      type="button"
      :class="buttonClasses"
      @click="handleClick"
      @dragover="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop"
    >
      {{ label }}
    </button>
  </div>
</template>
