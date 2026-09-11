<script setup>
import { computed } from 'vue'
import SqlEditor from './SqlEditor.vue'

const props = defineProps({
  processedSql: {
    type: String,
    default: '',
  },
  outputStats: {
    type: String,
    required: true,
  },
  nightMode: {
    type: Boolean,
    default: true,
  },
  editorTheme: {
    type: String,
    required: true,
  },
})

defineEmits(['download'])

const panelClass = computed(() =>
  props.nightMode
    ? 'flex h-full min-h-[460px] flex-col rounded-xl border border-[#16321d] bg-[rgba(7,12,7,0.88)] p-4 shadow-[0_0_0_1px_rgba(72,168,96,0.06)] backdrop-blur-sm'
    : 'flex h-full min-h-[460px] flex-col rounded-xl border border-[#c7d1c7] bg-[rgba(242,246,242,0.96)] p-4 shadow-[0_0_0_1px_rgba(19,34,56,0.04)]',
)
const headingClass = computed(() => (props.nightMode ? 'text-[#dbffe5]' : 'text-[#102033]'))
const metaClass = computed(() => (props.nightMode ? 'text-xs text-[#5ea96e]' : 'text-xs text-[#6c8098]'))
const ghostButtonClass = computed(() =>
  props.nightMode
    ? 'rounded-md border border-[#16321d] bg-[#091109] px-3 py-2 text-sm text-[#c7ffd4] transition hover:border-[#2a5d34] disabled:cursor-not-allowed disabled:opacity-40'
    : 'rounded-md border border-[#d1d9d1] bg-[#ffffff] px-3 py-2 text-sm text-[#16324f] transition hover:border-[#9da99d] disabled:cursor-not-allowed disabled:opacity-40',
)
</script>

<template>
  <article :class="panelClass">
    <div class="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
      <div class="flex min-w-0 items-center gap-3">
        <h2 :class="['m-0 text-[22px] leading-none font-semibold', headingClass]">处理结果</h2>
      </div>
      <div class="flex shrink-0 flex-wrap items-center gap-2 self-start lg:justify-end">
        <span :class="['whitespace-nowrap', metaClass]">{{ outputStats }}</span>
        <button :class="ghostButtonClass" type="button" :disabled="!processedSql" @click="$emit('download')">下载 .sql</button>
      </div>
    </div>

    <div class="mt-3 flex-1 min-h-[360px]">
      <SqlEditor :model-value="processedSql" :theme="editorTheme" :read-only="true" aria-label="处理结果编辑器" />
    </div>
  </article>
</template>
