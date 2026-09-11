<script setup>
import { computed } from 'vue'

const props = defineProps({
  profiles: {
    type: Array,
    default: () => [],
  },
  selectedProfileId: {
    type: String,
    default: '',
  },
  profile: {
    type: Object,
    required: true,
  },
  isExpanded: {
    type: Boolean,
    default: false,
  },
  nightMode: {
    type: Boolean,
    default: true,
  },
})

const emit = defineEmits([
  'update:selectedProfileId',
  'update:profile',
  'new-profile',
  'save-profile',
  'delete-profile',
  'toggle-expanded',
])

const panelClass = computed(() =>
  props.nightMode
    ? 'rounded-lg border border-[#17341f] bg-[#081008] p-3'
    : 'rounded-lg border border-[#d6dfd6] bg-[#ffffff] p-3',
)
const headingClass = computed(() => (props.nightMode ? 'text-[#dbffe5]' : 'text-[#102033]'))
const labelClass = computed(() =>
  props.nightMode ? 'mb-1 block text-xs text-[#74c786]' : 'mb-1 block text-xs text-[#49627c]',
)
const inputClass = computed(() =>
  props.nightMode
    ? 'h-10 w-full rounded-md border border-[#17341f] bg-[#091109] px-3 text-sm text-[#d6ffe1] outline-none transition focus:border-[#4dff79]'
    : 'h-10 w-full rounded-md border border-[#c9d4c9] bg-[#f7faf7] px-3 text-sm text-[#102033] outline-none transition focus:border-[#1455ff]',
)
const checkboxClass = computed(() =>
  props.nightMode ? 'h-4 w-4 rounded border-[#2a5d34] bg-[#091109]' : 'h-4 w-4 rounded border-[#9da99d] bg-[#ffffff]',
)
const buttonClass = computed(() =>
  props.nightMode
    ? 'rounded-md border border-[#16321d] bg-[#091109] px-3 py-2 text-sm text-[#c7ffd4] transition hover:border-[#2a5d34]'
    : 'rounded-md border border-[#d1d9d1] bg-[#ffffff] px-3 py-2 text-sm text-[#16324f] transition hover:border-[#9da99d]',
)
const helperClass = computed(() => (props.nightMode ? 'text-xs text-[#5ea96e]' : 'text-xs text-[#6c8098]'))
const summaryCardClass = computed(() =>
  props.nightMode
    ? 'mt-2 rounded-md border border-[#153119] bg-[#091109] px-3 py-2'
    : 'mt-2 rounded-md border border-[#dce4dc] bg-[#f8fbf8] px-3 py-2',
)
const badgeClass = computed(() =>
  props.profile.enabled
    ? props.nightMode
      ? 'rounded-full border border-[#1b5526] bg-[#0f2a15] px-2 py-0.5 text-xs text-[#8bffb0]'
      : 'rounded-full border border-[#a8d5b3] bg-[#eefbf1] px-2 py-0.5 text-xs text-[#1f6b35]'
    : props.nightMode
      ? 'rounded-full border border-[#3b4542] bg-[#111817] px-2 py-0.5 text-xs text-[#9bb6a3]'
      : 'rounded-full border border-[#d1d9d1] bg-[#f4f7f4] px-2 py-0.5 text-xs text-[#61756b]',
)

const profileModel = computed({
  get: () => props.profile,
  set: (value) => emit('update:profile', value),
})

function updateField(field, value) {
  profileModel.value = {
    ...props.profile,
    [field]: value,
  }
}

const serverSummary = computed(() => {
  const host = props.profile.host?.trim()
  const port = String(props.profile.port || '').trim()
  const service = props.profile.service?.trim()
  if (!host || !port || !service) {
    return '未配置服务器'
  }
  return `${host}:${port}/${service}`
})

const usernameSummary = computed(() => props.profile.username?.trim() || '未配置用户')
const passwordSummary = computed(() => (props.profile.password ? '已配置密码' : '未配置密码'))
</script>

<template>
  <section :class="panelClass">
    <div class="flex flex-col gap-2">
      <div class="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h3 :class="['m-0 text-lg font-semibold', headingClass]">Oracle 校验服务器</h3>
        </div>
        <div class="flex flex-wrap gap-2">
          <button :class="buttonClass" type="button" @click="$emit('new-profile')">新建</button>
          <button :class="buttonClass" type="button" @click="$emit('save-profile')">保存</button>
          <button :class="buttonClass" type="button" @click="$emit('delete-profile')">删除</button>
          <button :class="buttonClass" type="button" @click="$emit('toggle-expanded')">
            {{ isExpanded ? '收起配置' : '展开配置' }}
          </button>
        </div>
      </div>

      <div :class="summaryCardClass">
        <div class="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
          <div class="min-w-0">
            <p :class="['m-0 text-sm font-semibold', headingClass]">{{ profile.profileName || '未命名配置' }}</p>
            <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1">
              <span :class="helperClass">服务器：{{ serverSummary }}</span>
              <span :class="helperClass">用户：{{ usernameSummary }}</span>
              <span :class="helperClass">{{ passwordSummary }}</span>
            </div>
          </div>
          <span :class="badgeClass">{{ profile.enabled ? '已启用校验' : '未启用校验' }}</span>
        </div>
      </div>

      <div v-if="isExpanded" class="grid gap-3 pt-1">
        <div>
          <label :class="labelClass" for="validation-profile-select">Profile</label>
          <select
            id="validation-profile-select"
            :class="inputClass"
            :value="selectedProfileId"
            @change="$emit('update:selectedProfileId', $event.target.value)"
          >
            <option v-for="profileOption in profiles" :key="profileOption.id" :value="profileOption.id">
              {{ profileOption.profileName || '未命名配置' }}
            </option>
          </select>
        </div>

        <div class="flex items-center gap-2">
          <input
            id="validation-enabled"
            :checked="profile.enabled"
            :class="checkboxClass"
            type="checkbox"
            @change="updateField('enabled', $event.target.checked)"
          />
          <label :class="['text-sm', headingClass]" for="validation-enabled">启用 Oracle 校验</label>
        </div>

        <div class="grid gap-3 md:grid-cols-2">
          <div>
            <label :class="labelClass" for="validation-profile-name">名称</label>
            <input
              id="validation-profile-name"
              :class="inputClass"
              :value="profile.profileName"
              type="text"
              @input="updateField('profileName', $event.target.value)"
            />
          </div>

          <div>
            <label :class="labelClass" for="validation-host">Host</label>
            <input
              id="validation-host"
              :class="inputClass"
              :value="profile.host"
              type="text"
              placeholder="例如：10.0.0.10"
              @input="updateField('host', $event.target.value)"
            />
          </div>

          <div>
            <label :class="labelClass" for="validation-port">Port</label>
            <input
              id="validation-port"
              :class="inputClass"
              :value="profile.port"
              type="number"
              min="1"
              @input="updateField('port', $event.target.value)"
            />
          </div>

          <div>
            <label :class="labelClass" for="validation-service">Service</label>
            <input
              id="validation-service"
              :class="inputClass"
              :value="profile.service"
              type="text"
              placeholder="例如：XE"
              @input="updateField('service', $event.target.value)"
            />
          </div>

          <div>
            <label :class="labelClass" for="validation-username">Username</label>
            <input
              id="validation-username"
              :class="inputClass"
              :value="profile.username"
              type="text"
              @input="updateField('username', $event.target.value)"
            />
          </div>

          <div>
            <label :class="labelClass" for="validation-password">Password</label>
            <input
              id="validation-password"
              :class="inputClass"
              :value="profile.password"
              type="password"
              @input="updateField('password', $event.target.value)"
            />
          </div>

          <div>
            <label :class="labelClass" for="validation-timeout">超时（秒）</label>
            <input
              id="validation-timeout"
              :class="inputClass"
              :value="profile.timeoutSeconds"
              type="number"
              min="1"
              @input="updateField('timeoutSeconds', $event.target.value)"
            />
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
