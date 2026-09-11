<script setup>
import { computed } from 'vue'
import ValidationPanel from './ValidationPanel.vue'

const props = defineProps({
  validation: {
    type: Object,
    default: null,
  },
  validationProfile: {
    type: Object,
    required: true,
  },
  isValidationConfigComplete: {
    type: Boolean,
    required: true,
  },
  missingValidationFields: {
    type: Array,
    default: () => [],
  },
  isLoading: {
    type: Boolean,
    default: false,
  },
  nightMode: {
    type: Boolean,
    default: true,
  },
})

const wrapperClass = computed(() =>
  props.nightMode
    ? 'rounded-xl border border-[#16321d] bg-[rgba(7,12,7,0.88)] p-4 shadow-[0_0_0_1px_rgba(72,168,96,0.06)] backdrop-blur-sm'
    : 'rounded-xl border border-[#c7d1c7] bg-[rgba(242,246,242,0.96)] p-4 shadow-[0_0_0_1px_rgba(19,34,56,0.04)]',
)
const headingClass = computed(() => (props.nightMode ? 'text-[#dbffe5]' : 'text-[#102033]'))
const bodyClass = computed(() => (props.nightMode ? 'text-sm text-[#8ce29f]' : 'text-sm text-[#31455b]'))
const emptyClass = computed(() =>
  props.nightMode
    ? 'mt-3 rounded-md border border-[#17341f] bg-[#081008] px-3 py-3 text-sm text-[#8ce29f]'
    : 'mt-3 rounded-md border border-[#dce4dc] bg-[#ffffff] px-3 py-3 text-sm text-[#31455b]',
)

const profileLabel = computed(() => props.validationProfile.profileName || '未命名配置')

const emptyMessage = computed(() => {
  if (!props.validationProfile.enabled) {
    return 'Oracle 校验未启用。启用并填写服务器配置后，处理 SQL 时会在这里显示校验结果。'
  }
  if (!props.isValidationConfigComplete) {
    return `当前配置缺少以下字段：${props.missingValidationFields.join(', ')}。`
  }
  if (props.isLoading) {
    return '正在执行 Oracle 校验，请稍候。'
  }
  if (!props.validation?.targets?.length) {
    return '配置已就绪。执行一次处理后，这里会显示原始 SQL 和处理后 SQL 的校验结果。'
  }
  return ''
})
</script>

<template>
  <section :class="wrapperClass">
    <div class="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
      <div>
        <h2 :class="['m-0 text-[22px] leading-none font-semibold', headingClass]">Oracle 校验</h2>
        <p :class="['mt-2', bodyClass]">当前配置：{{ profileLabel }}</p>
      </div>
    </div>

    <p v-if="emptyMessage" :class="emptyClass">{{ emptyMessage }}</p>

    <div v-else class="mt-3 grid gap-3 xl:grid-cols-2">
      <ValidationPanel
        v-for="target in validation.targets"
        :key="target.target"
        :target="target"
        :night-mode="nightMode"
      />
    </div>
  </section>
</template>
