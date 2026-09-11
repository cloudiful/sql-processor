<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { createMatrixRainEngine } from '../lib/matrix-rain-engine.js'

const props = defineProps({
  enabled: {
    type: Boolean,
    default: true,
  },
  quality: {
    type: String,
    default: 'auto',
    validator: (value) => ['auto', 'low', 'normal'].includes(value),
  },
})

const canvasRef = ref(null)
const engine = createMatrixRainEngine({
  getCanvas: () => canvasRef.value,
  getEnabled: () => props.enabled,
  getQuality: () => props.quality,
})

onMounted(() => {
  engine.mount()
})

onBeforeUnmount(() => {
  engine.unmount()
})

watch(
  () => [props.enabled, props.quality],
  () => {
    engine.syncOptions()
  },
)
</script>

<template>
  <div class="pointer-events-none fixed inset-0 overflow-hidden">
    <canvas ref="canvasRef" class="block h-full w-full" aria-hidden="true"></canvas>
  </div>
</template>
