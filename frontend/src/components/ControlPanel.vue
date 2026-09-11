<script setup>
import { computed } from 'vue'
import OracleValidationConfigPanel from './OracleValidationConfigPanel.vue'

const props = defineProps({
  schema: {
    type: String,
    default: '',
  },
  inputStats: {
    type: String,
    required: true,
  },
  isLoading: {
    type: Boolean,
    default: false,
  },
  canProcess: {
    type: Boolean,
    default: false,
  },
  nightMode: {
    type: Boolean,
    default: true,
  },
  validationProfiles: {
    type: Array,
    default: () => [],
  },
  selectedValidationProfileId: {
    type: String,
    default: '',
  },
  validationProfile: {
    type: Object,
    required: true,
  },
  isValidationConfigExpanded: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits([
  'update:schema',
  'process',
  'fill-example',
  'clear-all',
  'update:selectedValidationProfileId',
  'update:validationProfile',
  'new-validation-profile',
  'save-validation-profile',
  'delete-validation-profile',
  'toggle-validation-config-expanded',
])

const schemaModel = computed({
  get: () => props.schema,
  set: (value) => emit('update:schema', value),
})

const panelClass = computed(() =>
  props.nightMode
    ? 'rounded-xl border border-[#16321d] bg-[rgba(7,12,7,0.88)] p-4 shadow-[0_0_0_1px_rgba(72,168,96,0.06)] backdrop-blur-sm'
    : 'rounded-xl border border-[#c7d1c7] bg-[rgba(242,246,242,0.96)] p-4 shadow-[0_0_0_1px_rgba(19,34,56,0.04)]',
)
const headingClass = computed(() => (props.nightMode ? 'text-[#dbffe5]' : 'text-[#102033]'))
const helperClass = computed(() => (props.nightMode ? 'text-xs text-[#5ea96e]' : 'text-xs text-[#6c8098]'))
const labelClass = computed(() =>
  props.nightMode ? 'mb-1.5 block text-sm text-[#74c786]' : 'mb-1.5 block text-sm text-[#334961]',
)
const inputClass = computed(() =>
  props.nightMode
    ? 'h-10 w-full rounded-md border border-[#17341f] bg-[#081008] px-3 text-[#d6ffe1] outline-none transition focus:border-[#4dff79] focus:shadow-[0_0_0_1px_rgba(77,255,121,0.22)]'
    : 'h-10 w-full rounded-md border border-[#c9d4c9] bg-[#f7faf7] px-3 text-[#102033] outline-none transition focus:border-[#1455ff] focus:shadow-[0_0_0_1px_rgba(20,85,255,0.14)]',
)
const ghostButtonClass = computed(() =>
  props.nightMode
    ? 'rounded-md border border-[#16321d] bg-[#091109] px-3 py-2 text-sm text-[#c7ffd4] transition hover:border-[#2a5d34] disabled:cursor-not-allowed disabled:opacity-40'
    : 'rounded-md border border-[#d1d9d1] bg-[#ffffff] px-3 py-2 text-sm text-[#16324f] transition hover:border-[#9da99d] disabled:cursor-not-allowed disabled:opacity-40',
)
const primaryButtonClass = computed(() =>
  props.nightMode
    ? 'inline-flex min-w-[144px] items-center justify-center rounded-md border border-[#1e6f2f] bg-[#0f2a15] px-4 py-2.5 text-sm text-[#bfffcf] transition hover:border-[#4dff79] hover:text-[#d6ffe1] disabled:cursor-not-allowed disabled:opacity-40'
    : 'inline-flex min-w-[144px] items-center justify-center rounded-md border border-[#1f5acc] bg-[#1455ff] px-4 py-2.5 text-sm text-[#f5f9ff] transition hover:border-[#0f44cb] hover:bg-[#0f44cb] disabled:cursor-not-allowed disabled:opacity-40',
)
</script>

<template>
  <article :class="panelClass">
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 :class="['m-0 text-[22px] leading-none font-semibold', headingClass]">设置与操作</h2>
        </div>
        <span :class="helperClass">当前输入：{{ inputStats }}</span>
      </div>

      <div class="grid gap-3 xl:grid-cols-[minmax(0,1fr)_auto] xl:items-start">
        <div class="grid gap-3">
          <div>
            <label :class="labelClass" for="schema">目标 Schema</label>
            <input id="schema" v-model.trim="schemaModel" :class="inputClass" placeholder="可选，例如：EBANK" />
          </div>

          <OracleValidationConfigPanel
            :profiles="validationProfiles"
            :selected-profile-id="selectedValidationProfileId"
            :profile="validationProfile"
            :is-expanded="isValidationConfigExpanded"
            :night-mode="nightMode"
            @update:selected-profile-id="$emit('update:selectedValidationProfileId', $event)"
            @update:profile="$emit('update:validationProfile', $event)"
            @new-profile="$emit('new-validation-profile')"
            @save-profile="$emit('save-validation-profile')"
            @delete-profile="$emit('delete-validation-profile')"
            @toggle-expanded="$emit('toggle-validation-config-expanded')"
          />
        </div>

        <div class="flex flex-wrap gap-2 xl:w-[160px] xl:flex-col xl:items-stretch">
          <button :class="ghostButtonClass" type="button" @click="$emit('fill-example')">填充示例</button>
          <button :class="ghostButtonClass" type="button" @click="$emit('clear-all')">清空输入</button>
          <button :class="primaryButtonClass" type="button" :disabled="isLoading || !canProcess" @click="$emit('process')">
            {{ isLoading ? '处理中...' : '开始处理' }}
          </button>
        </div>
      </div>
    </div>
  </article>
</template>
