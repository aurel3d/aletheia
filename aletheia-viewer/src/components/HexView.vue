<script setup lang="ts">
import { ref, computed, watch } from 'vue'

interface Props {
  rawBytes: Uint8Array
  selectedRange: [number, number] | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  click: [offset: number]
}>()

const BYTES_PER_LINE = 16
const VISIBLE_LINES = 50 // Number of lines to render at once

const scrollOffset = ref(0)
const containerRef = ref<HTMLElement | null>(null)

const totalLines = computed(() => Math.ceil(props.rawBytes.length / BYTES_PER_LINE))

const visibleLines = computed(() => {
  const startLine = Math.floor(scrollOffset.value)
  const endLine = Math.min(startLine + VISIBLE_LINES, totalLines.value)

  const lines = []
  for (let lineNum = startLine; lineNum < endLine; lineNum++) {
    const offset = lineNum * BYTES_PER_LINE
    const lineBytes = props.rawBytes.slice(offset, offset + BYTES_PER_LINE)

    if (lineBytes.length === 0) continue

    lines.push({
      offset,
      bytes: lineBytes,
      hex: Array.from(lineBytes).map(b => b.toString(16).padStart(2, '0')),
      ascii: Array.from(lineBytes).map(b =>
        b >= 32 && b <= 126 ? String.fromCharCode(b) : '.'
      ),
    })
  }

  return lines
})

function handleScroll(event: Event) {
  const target = event.target as HTMLElement
  const scrollTop = target.scrollTop
  const lineHeight = 24 // approximate line height in pixels
  scrollOffset.value = scrollTop / lineHeight
}

function handleByteClick(offset: number) {
  emit('click', offset)
}

function isInSelectedRange(offset: number): boolean {
  if (!props.selectedRange) return false
  return offset >= props.selectedRange[0] && offset < props.selectedRange[1]
}

// Scroll to selected range when it changes
watch(() => props.selectedRange, (range) => {
  if (!range || !containerRef.value) return

  const startLine = Math.floor(range[0] / BYTES_PER_LINE)
  const lineHeight = 24
  containerRef.value.scrollTop = startLine * lineHeight
})
</script>

<template>
  <div
    ref="containerRef"
    class="h-full overflow-auto bg-gray-900 text-gray-100 font-mono text-sm"
    @scroll="handleScroll"
  >
    <div class="p-4">
      <!-- Header -->
      <div class="flex gap-4 pb-2 border-b border-gray-700 mb-2 text-gray-400 text-xs sticky top-0 bg-gray-900 z-10">
        <div class="w-24">Offset</div>
        <div class="flex-1">Hex</div>
        <div class="w-32">ASCII</div>
      </div>

      <!-- Placeholder for scrolling -->
      <div :style="{ height: `${totalLines * 24}px`, position: 'relative' }">
        <div :style="{ transform: `translateY(${Math.floor(scrollOffset) * 24}px)` }">
          <!-- Lines -->
          <div
            v-for="line in visibleLines"
            :key="line.offset"
            class="flex gap-4 leading-6 hover:bg-gray-800"
          >
            <!-- Offset -->
            <div class="w-24 text-gray-500">
              {{ line.offset.toString(16).padStart(8, '0') }}
            </div>

            <!-- Hex bytes -->
            <div class="flex-1 flex gap-1">
              <button
                v-for="(byte, idx) in line.hex"
                :key="idx"
                type="button"
                :class="[
                  'px-0.5 hover:bg-blue-900 rounded',
                  isInSelectedRange(line.offset + idx) ? 'bg-blue-600 text-white' : '',
                ]"
                @click="handleByteClick(line.offset + idx)"
              >
                {{ byte }}
              </button>
            </div>

            <!-- ASCII -->
            <div class="w-32 text-gray-400">
              {{ line.ascii.join('') }}
            </div>
          </div>
        </div>
      </div>

      <!-- Info footer -->
      <div v-if="selectedRange" class="mt-4 p-3 bg-gray-800 rounded text-xs">
        <div class="text-gray-400">Selected Range:</div>
        <div class="mt-1">
          <span class="text-blue-400">Start:</span> 0x{{ selectedRange[0].toString(16).padStart(8, '0') }} ({{ selectedRange[0] }})
        </div>
        <div>
          <span class="text-blue-400">End:</span> 0x{{ selectedRange[1].toString(16).padStart(8, '0') }} ({{ selectedRange[1] }})
        </div>
        <div>
          <span class="text-blue-400">Length:</span> {{ selectedRange[1] - selectedRange[0] }} bytes
        </div>
      </div>
    </div>
  </div>
</template>
