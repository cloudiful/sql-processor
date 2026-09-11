<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

const props = defineProps({
  modelValue: {
    type: String,
    default: '',
  },
  theme: {
    type: String,
    default: 'sql-processor-night',
  },
  readOnly: {
    type: Boolean,
    default: false,
  },
  ariaLabel: {
    type: String,
    default: 'SQL editor',
  },
})

const emit = defineEmits(['update:modelValue'])

const rootEl = ref(null)
const isReady = ref(false)
let editor
let model
let monaco
let suppressEmit = false

const placeholderClass = computed(() =>
  props.theme === 'sql-processor-night'
    ? 'absolute inset-0 z-10 flex items-center justify-center rounded-md bg-[rgba(6,12,6,0.9)] text-sm tracking-[0.04em] text-[#6ee984]'
    : 'absolute inset-0 z-10 flex items-center justify-center rounded-md bg-[rgba(240,245,240,0.9)] text-sm tracking-[0.04em] text-[#58708c]',
)

onMounted(async () => {
  ;({ monaco } = await import('../monaco'))
  model = monaco.editor.createModel(props.modelValue, 'sql')
  editor = monaco.editor.create(rootEl.value, {
    model,
    theme: props.theme,
    automaticLayout: true,
    minimap: { enabled: false },
    readOnly: props.readOnly,
    roundedSelection: true,
    scrollBeyondLastLine: false,
    fontSize: 14,
    lineHeight: 22,
    fontFamily: '"SF Mono", "JetBrains Mono", "Menlo", monospace',
    wordWrap: 'on',
    tabSize: 2,
    padding: { top: 12, bottom: 12 },
    ariaLabel: props.ariaLabel,
  })

  editor.onDidChangeModelContent(() => {
    if (!suppressEmit) {
      emit('update:modelValue', model.getValue())
    }
  })

  isReady.value = true
})

watch(
  () => props.modelValue,
  (value) => {
    if (!model || value === model.getValue()) {
      return
    }

    suppressEmit = true
    const currentSelection = editor?.getSelection()
    model.setValue(value)
    if (editor && currentSelection && !props.readOnly) {
      editor.setSelection(currentSelection)
    }
    suppressEmit = false
  },
)

watch(
  () => props.theme,
  (value) => {
    if (editor && monaco) {
      monaco.editor.setTheme(value)
    }
  },
)

watch(
  () => props.readOnly,
  (value) => {
    if (editor) {
      editor.updateOptions({ readOnly: value })
    }
  },
)

onBeforeUnmount(() => {
  if (editor) {
    editor.dispose()
  }
  if (model) {
    model.dispose()
  }
})
</script>

<template>
  <div class="relative h-full min-h-[220px]">
    <div v-if="!isReady" :class="placeholderClass">编辑器加载中...</div>
    <div ref="rootEl" class="h-full min-h-[220px] overflow-hidden rounded-md"></div>
  </div>
</template>
