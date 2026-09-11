<script setup>
import { computed, ref, watch } from 'vue'
import { useValidationProfiles } from './composables/useValidationProfiles.js'
import ControlPanel from './components/ControlPanel.vue'
import InputPanel from './components/InputPanel.vue'
import MatrixRainCanvas from './components/MatrixRainCanvas.vue'
import OracleValidationSection from './components/OracleValidationSection.vue'
import OutputPanel from './components/OutputPanel.vue'
import ProcessingFeedbackPanel from './components/ProcessingFeedbackPanel.vue'
import ThemeFooterToggle from './components/ThemeFooterToggle.vue'
import { processSqlRequest } from './lib/process-api.ts'
import { buildValidationConfig } from './lib/validation-profiles.js'

const exampleSQL = `CREATE TABLE ORDERS (
  ID NUMBER PRIMARY KEY,
  ORDER_NO VARCHAR2(50) NOT NULL
);

INSERT INTO ORDERS (ID, ORDER_NO) VALUES (1, 'SO-20260403');`

const schema = ref('')
const sql = ref(exampleSQL)
const processedSql = ref('')
const messages = ref([])
const validation = ref(null)
const errorMessage = ref('')
const isLoading = ref(false)
const nightMode = ref(true)
const {
  validationProfiles,
  selectedValidationProfileId,
  validationProfile,
  isValidationConfigExpanded,
  missingValidationFields,
  isValidationConfigComplete,
  saveProfile,
  createProfile,
  deleteProfile,
} = useValidationProfiles()

const inputStats = computed(() => `${sql.value.length} chars`)
const outputStats = computed(() => `${processedSql.value.length} chars`)
const editorTheme = computed(() => (nightMode.value ? 'sql-processor-night' : 'sql-processor-day'))
const shellClass = computed(() =>
  nightMode.value
    ? 'relative flex min-h-screen flex-col overflow-hidden bg-[#050805] text-[#dce6d8] transition-colors duration-200'
    : 'relative flex min-h-screen flex-col bg-[#edf2ed] text-[#132238] transition-colors duration-200',
)

watch(
  validationProfile,
  () => {
    validation.value = null
  },
  { deep: true },
)

async function processSQL() {
  errorMessage.value = ''
  validation.value = null
  isLoading.value = true

  try {
    const payload = await processSqlRequest({
      sql: sql.value,
      schema: schema.value,
      validationConfig: buildValidationConfig(validationProfile.value),
    })

    processedSql.value = payload.processedSql || ''
    messages.value = payload.messages || []
    validation.value = payload.validation || null
  } catch (error) {
    processedSql.value = ''
    messages.value = []
    errorMessage.value = error instanceof Error ? error.message : '处理失败'
  } finally {
    isLoading.value = false
  }
}

function saveValidationProfile() {
  const draft = saveProfile()
  messages.value = [`Oracle 校验配置“${draft.profileName || '未命名配置'}”已保存。`, ...messages.value]
}

function newValidationProfile() {
  createProfile()
}

function deleteValidationProfile() {
  deleteProfile()
}

function fillExample() {
  sql.value = exampleSQL
  if (!processedSql.value) {
    messages.value = ['已填充示例 SQL，可以直接点击处理。']
  }
}

function clearAll() {
  sql.value = ''
  processedSql.value = ''
  messages.value = []
  validation.value = null
  errorMessage.value = ''
}

function downloadResult() {
  if (!processedSql.value) {
    return
  }

  const blob = new Blob([processedSql.value], { type: 'text/sql;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = 'processed.sql'
  link.click()
  URL.revokeObjectURL(url)
}
</script>

<template>
  <main :class="shellClass">
    <MatrixRainCanvas v-if="nightMode" />
    <section class="relative z-10 mx-auto flex w-[min(1680px,calc(100%-20px))] flex-col gap-3 py-3 xl:gap-4 xl:py-4">
      <ControlPanel
        v-model:schema="schema"
        :validation-profiles="validationProfiles"
        :selected-validation-profile-id="selectedValidationProfileId"
        :validation-profile="validationProfile"
        :is-validation-config-expanded="isValidationConfigExpanded"
        :input-stats="inputStats"
        :is-loading="isLoading"
        :can-process="Boolean(sql.trim())"
        :night-mode="nightMode"
        @process="processSQL"
        @fill-example="fillExample"
        @clear-all="clearAll"
        @update:selected-validation-profile-id="selectedValidationProfileId = $event"
        @update:validation-profile="validationProfile = $event"
        @new-validation-profile="newValidationProfile"
        @save-validation-profile="saveValidationProfile"
        @delete-validation-profile="deleteValidationProfile"
        @toggle-validation-config-expanded="isValidationConfigExpanded = !isValidationConfigExpanded"
      />

      <div class="grid gap-3 xl:grid-cols-2 xl:items-stretch">
        <InputPanel v-model:sql="sql" :input-stats="inputStats" :night-mode="nightMode" :editor-theme="editorTheme" />

        <OutputPanel
          :processed-sql="processedSql"
          :output-stats="outputStats"
          :night-mode="nightMode"
          :editor-theme="editorTheme"
          @download="downloadResult"
        />
      </div>

      <div class="grid gap-3 xl:grid-cols-[minmax(0,0.86fr)_minmax(0,1.14fr)] xl:items-start">
        <ProcessingFeedbackPanel :messages="messages" :error-message="errorMessage" :night-mode="nightMode" />

        <OracleValidationSection
          :validation="validation"
          :validation-profile="validationProfile"
          :is-validation-config-complete="isValidationConfigComplete"
          :missing-validation-fields="missingValidationFields"
          :is-loading="isLoading"
          :night-mode="nightMode"
        />
      </div>
    </section>
    <ThemeFooterToggle :night-mode="nightMode" @toggle="nightMode = !nightMode" />
  </main>
</template>
