<script setup>
import { computed } from 'vue'

const props = defineProps({
  messages: {
    type: Array,
    default: () => [],
  },
  errorMessage: {
    type: String,
    default: '',
  },
  nightMode: {
    type: Boolean,
    default: true,
  },
})

const panelClass = computed(() =>
  props.nightMode
    ? 'rounded-xl border border-[#16321d] bg-[rgba(7,12,7,0.88)] p-4 shadow-[0_0_0_1px_rgba(72,168,96,0.06)] backdrop-blur-sm'
    : 'rounded-xl border border-[#c7d1c7] bg-[rgba(242,246,242,0.96)] p-4 shadow-[0_0_0_1px_rgba(19,34,56,0.04)]',
)
const headingClass = computed(() => (props.nightMode ? 'text-[#dbffe5]' : 'text-[#102033]'))
const helperClass = computed(() => (props.nightMode ? 'text-sm text-[#8ce29f]' : 'text-sm text-[#31455b]'))
const logListClass = computed(() =>
  props.nightMode ? 'mt-3 list-disc pl-4 text-[#8ce29f]' : 'mt-3 list-disc pl-4 text-[#31455b]',
)
const emptyClass = computed(() =>
  props.nightMode
    ? 'mt-3 rounded-md border border-[#17341f] bg-[#081008] px-3 py-3 text-sm text-[#8ce29f]'
    : 'mt-3 rounded-md border border-[#dce4dc] bg-[#ffffff] px-3 py-3 text-sm text-[#31455b]',
)
const errorClass = computed(() =>
  props.nightMode
    ? 'mt-3 rounded-md border border-[#4f1a1a] bg-[#1a0b0b] px-3 py-2 text-[#ffb8b8]'
    : 'mt-3 rounded-md border border-[#e3c7c7] bg-[#fff4f4] px-3 py-2 text-[#7c1919]',
)
</script>

<template>
  <section :class="panelClass">
    <div>
      <h2 :class="['m-0 text-[22px] leading-none font-semibold', headingClass]">处理日志</h2>
    </div>

    <p v-if="errorMessage" :class="errorClass">{{ errorMessage }}</p>

    <ul v-if="messages.length" :class="logListClass">
      <li v-for="message in messages" :key="message" class="mt-2 first:mt-0">{{ message }}</li>
    </ul>
    <p v-else :class="emptyClass">还没有日志。处理一次后这里会显示处理动作。</p>
  </section>
</template>
