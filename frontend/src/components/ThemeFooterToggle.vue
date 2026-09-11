<script setup>
import { computed } from 'vue'

const props = defineProps({
  nightMode: {
    type: Boolean,
    default: true,
  },
})

defineEmits(['toggle'])

const buttonClass = computed(() =>
  props.nightMode
    ? 'inline-flex h-10 w-10 items-center justify-center rounded-md border border-[#16321d] bg-[#091109] text-[#bfffcf] shadow-[0_0_0_1px_rgba(72,168,96,0.06)] transition hover:border-[#2a5d34]'
    : 'inline-flex h-10 w-10 items-center justify-center rounded-md border border-[#d1d9d1] bg-[#ffffff] text-[#16324f] shadow-[0_0_0_1px_rgba(19,34,56,0.04)] transition hover:border-[#9da99d]',
)

// Icon source: MDI theme-light-dark from pictogrammers.com
const iconPath =
  'M7.5,2C5.71,3.15 4.5,5.18 4.5,7.5C4.5,9.82 5.71,11.85 7.53,13C4.46,13 2,10.54 2,7.5A5.5,5.5 0 0,1 7.5,2M19.07,3.5L20.5,4.93L4.93,20.5L3.5,19.07L19.07,3.5M12.89,5.93L11.41,5L9.97,6L10.39,4.3L9,3.24L10.75,3.12L11.33,1.47L12,3.1L13.73,3.13L12.38,4.26L12.89,5.93M9.59,9.54L8.43,8.81L7.31,9.59L7.65,8.27L6.56,7.44L7.92,7.35L8.37,6.06L8.88,7.33L10.24,7.36L9.19,8.23L9.59,9.54M19,13.5A5.5,5.5 0 0,1 13.5,19C12.28,19 11.15,18.6 10.24,17.93L17.93,10.24C18.6,11.15 19,12.28 19,13.5M14.6,20.08L17.37,18.93L17.13,22.28L14.6,20.08M18.93,17.38L20.08,14.61L22.28,17.15L18.93,17.38M20.08,12.42L18.94,9.64L22.28,9.88L20.08,12.42M9.63,18.93L12.4,20.08L9.87,22.27L9.63,18.93Z'
</script>

<template>
  <footer class="pointer-events-none mt-3 flex justify-center pb-4 xl:pb-5">
    <button
      type="button"
      :class="buttonClass"
      class="pointer-events-auto"
      :title="nightMode ? '切到日间模式' : '切到深夜模式'"
      :aria-label="nightMode ? '切到日间模式' : '切到深夜模式'"
      @click="$emit('toggle')"
    >
      <svg viewBox="0 0 24 24" class="h-4.5 w-4.5 fill-current" aria-hidden="true">
        <path :d="iconPath" />
      </svg>
      <span class="sr-only">{{ nightMode ? '切到日间模式' : '切到深夜模式' }}</span>
    </button>
  </footer>
</template>
