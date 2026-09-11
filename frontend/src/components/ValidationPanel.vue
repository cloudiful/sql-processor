<script setup>
import { computed } from 'vue'

const props = defineProps({
  target: {
    type: Object,
    required: true,
  },
  nightMode: {
    type: Boolean,
    default: true,
  },
})

const cardClass = computed(() =>
  props.nightMode
    ? 'rounded-lg border border-[#16321d] bg-[#081008] p-3'
    : 'rounded-lg border border-[#d5ddd5] bg-[#ffffff] p-3',
)
const headingClass = computed(() => (props.nightMode ? 'text-[#dbffe5]' : 'text-[#102033]'))
const summaryClass = computed(() => (props.nightMode ? 'text-sm text-[#8ce29f]' : 'text-sm text-[#31455b]'))
const issueClass = computed(() =>
  props.nightMode
    ? 'rounded-md border border-[#17341f] bg-[#0b150b] px-3 py-2'
    : 'rounded-md border border-[#dce4dc] bg-[#f8fbf8] px-3 py-2',
)

const statusBadgeClass = computed(() => {
  switch (props.target.status) {
    case 'passed':
      return props.nightMode
        ? 'rounded-full border border-[#1b5526] bg-[#0f2a15] px-2 py-0.5 text-xs text-[#8bffb0]'
        : 'rounded-full border border-[#a8d5b3] bg-[#eefbf1] px-2 py-0.5 text-xs text-[#1f6b35]'
    case 'failed':
      return props.nightMode
        ? 'rounded-full border border-[#6b1f1f] bg-[#2a0f0f] px-2 py-0.5 text-xs text-[#ffb8b8]'
        : 'rounded-full border border-[#e3c7c7] bg-[#fff4f4] px-2 py-0.5 text-xs text-[#8d2222]'
    case 'partial':
      return props.nightMode
        ? 'rounded-full border border-[#6b5520] bg-[#2b210d] px-2 py-0.5 text-xs text-[#ffe49b]'
        : 'rounded-full border border-[#e7d7ac] bg-[#fff9ea] px-2 py-0.5 text-xs text-[#8a6114]'
    default:
      return props.nightMode
        ? 'rounded-full border border-[#2a3d53] bg-[#0d1620] px-2 py-0.5 text-xs text-[#a5c6ff]'
        : 'rounded-full border border-[#cad8eb] bg-[#f1f6fd] px-2 py-0.5 text-xs text-[#2f4f7c]'
  }
})

function targetLabel(targetName) {
  return targetName === 'original' ? '原始 SQL 校验' : '处理后 SQL 校验'
}
</script>

<template>
  <section :class="cardClass">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <h4 :class="['m-0 text-base font-semibold', headingClass]">{{ targetLabel(target.target) }}</h4>
      <span :class="statusBadgeClass">{{ target.status }}</span>
    </div>

    <ul class="mt-3 space-y-1">
      <li v-for="summary in target.summary || []" :key="summary" :class="summaryClass">
        {{ summary }}
      </li>
    </ul>

    <div v-if="target.issues?.length" class="mt-3 space-y-2">
      <article v-for="issue in target.issues" :key="`${target.target}-${issue.unitIndex}-${issue.status}-${issue.message}`" :class="issueClass">
        <p :class="['m-0 text-sm font-semibold', headingClass]">
          {{ issue.status }} · {{ issue.statementType }} · Unit {{ issue.unitIndex }} · Line {{ issue.line }}
        </p>
        <p v-if="issue.code" :class="['mt-1 text-sm', summaryClass]">{{ issue.code }}</p>
        <p :class="['mt-1 text-sm', summaryClass]">{{ issue.message }}</p>
        <p v-if="issue.snippet" :class="['mt-1 break-all font-mono text-xs', summaryClass]">{{ issue.snippet }}</p>
      </article>
    </div>
  </section>
</template>
