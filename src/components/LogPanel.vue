<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import type { History, LogEvent } from '../types'
import { t, statusText } from '../i18n'
import { formatAppMessage, errorHint } from '../i18n/appMessages'
import AppIcon from './AppIcon.vue'
const props = defineProps<{ logs: LogEvent[]; running: boolean; stopping: boolean; result: History | null; runError: string }>()
defineEmits<{ stop: [] }>()
const output = ref<HTMLTextAreaElement>()
const follow = ref(true)
const copyMessage = ref('')
const copying = ref(false)
const logText = computed(() => props.logs.map(entry => entry.stream === 'system' ? formatAppMessage(entry.line) : entry.line).join('\n'))
const failureHint = computed(() => props.running ? '' : props.runError || props.result?.status === 'failed' || props.result?.status === 'timeout' ? errorHint(props.runError + '\n' + logText.value) : '')
const resultText = computed(() => {
  if (props.running) return props.stopping ? t('log.stopping') : t('log.running')
  if (props.runError) return t('log.error')
  if (!props.result) return t('log.idle')
  const mode = props.result.request.simulate ? t('common.simulation') : t('common.sync')
  const status = props.result.status === 'success' ? t('log.complete') : statusText(props.result.status)
  return t('log.result', { mode, status, code: props.result.exitCode ?? '—' })
})
let frame: number | undefined
watch([logText, follow], () => {
  if (frame !== undefined) cancelAnimationFrame(frame)
  frame = undefined
  if (!follow.value) return
  frame = requestAnimationFrame(() => {
    frame = undefined
    const area = output.value
    if (!area || (document.activeElement === area && area.selectionStart !== area.selectionEnd)) return
    area.scrollTop = area.scrollHeight
  })
}, { flush: 'post' })
onBeforeUnmount(() => { if (frame !== undefined) cancelAnimationFrame(frame) })

async function copyLogs() {
  copying.value = true
  copyMessage.value = ''
  const text = `${resultText.value}\n${logText.value}`
  try {
    if (!navigator.clipboard?.writeText) throw new Error('Clipboard unavailable')
    await navigator.clipboard.writeText(text)
    copyMessage.value = 'log.copied'
  } catch {
    // WKWebView may not provide the Clipboard API. Keep copying tied to this button.
    const field = document.createElement('textarea')
    field.value = text
    field.readOnly = true
    field.style.cssText = 'position:fixed;left:-9999px;top:0'
    const previous = document.activeElement as HTMLElement | null
    document.body.append(field)
    field.select()
    try {
      copyMessage.value = document.execCommand('copy') ? 'log.copied' : 'log.copyFailed'
    } catch {
      copyMessage.value = 'log.copyFailed'
    } finally { field.remove(); previous?.focus() }
  } finally { copying.value = false }
}
</script>
<template>
  <section class="log-panel" aria-labelledby="log-heading">
    <div class="log-toolbar">
      <h2 id="log-heading"><AppIcon name="terminal" :size="16" /> {{ t('log.title') }} <span class="log-count">{{ t('log.count', { count: logs.length }) }}</span></h2>
      <div class="row log-controls">
        <span class="log-status" role="status">{{ resultText }}</span>
        <label class="follow"><input v-model="follow" type="checkbox" /> {{ t('log.follow') }}</label>
        <button class="log-copy" :disabled="!logs.length || copying" @click="copyLogs">{{ t('log.copy') }}</button>
        <button v-if="running" class="stop-button" :disabled="stopping" @click="$emit('stop')"><AppIcon name="stop" :size="13" />{{ stopping ? t('log.stopPending') : t('log.stop') }}</button>
      </div>
    </div>
    <p v-if="failureHint" class="log-copy-message" role="status">{{ failureHint }}</p>
    <p v-if="copyMessage" class="log-copy-message" role="status">{{ t(copyMessage) }}</p>
    <textarea ref="output" class="log-output" :aria-label="t('log.label')" :value="logText" readonly wrap="off" spellcheck="false" :placeholder="t('log.placeholder')" />
  </section>
</template>
