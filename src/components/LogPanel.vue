<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import type { History, LogEvent } from '../types'
import { statusLabel } from '../types'
import AppIcon from './AppIcon.vue'
const props = defineProps<{ logs: LogEvent[]; running: boolean; stopping: boolean; result: History | null; runError: string }>()
defineEmits<{ stop: [] }>()
const output = ref<HTMLTextAreaElement>()
const follow = ref(true)
const copyMessage = ref('')
const copying = ref(false)
const logText = computed(() => props.logs.map(entry => entry.line).join('\n'))
const resultText = computed(() => {
  if (props.running) return props.stopping ? '停止処理中' : '実行中'
  if (props.runError) return '実行エラー'
  if (!props.result) return '待機中'
  const mode = props.result.request.simulate ? 'シミュレーション' : '同期'
  const status = props.result.status === 'success' ? '完了' : statusLabel[props.result.status]
  return `${mode}${status} · 終了コード ${props.result.exitCode ?? '—'}`
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
    copyMessage.value = 'コピーしました'
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
      copyMessage.value = document.execCommand('copy') ? 'コピーしました' : 'コピーできませんでした。ログを選択してコピーしてください。'
    } catch {
      copyMessage.value = 'コピーできませんでした。ログを選択してコピーしてください。'
    } finally { field.remove(); previous?.focus() }
  } finally { copying.value = false }
}
</script>
<template>
  <section class="log-panel" aria-labelledby="log-heading">
    <div class="log-toolbar">
      <h2 id="log-heading"><AppIcon name="terminal" :size="16" /> 実行ログ <span class="log-count">{{ logs.length }} 行</span></h2>
      <div class="row log-controls">
        <span class="log-status" role="status">{{ resultText }}</span>
        <label class="follow"><input v-model="follow" type="checkbox" /> 自動スクロール</label>
        <button class="log-copy" :disabled="!logs.length || copying" @click="copyLogs">ログをコピー</button>
        <button v-if="running" class="stop-button" :disabled="stopping" @click="$emit('stop')"><AppIcon name="stop" :size="13" />{{ stopping ? '停止しています…' : '停止' }}</button>
      </div>
    </div>
    <p v-if="copyMessage" class="log-copy-message" role="status">{{ copyMessage }}</p>
    <textarea ref="output" class="log-output" aria-label="実行ログ。最新 2000 行。選択してコピーできます" :value="logText" readonly wrap="off" spellcheck="false" placeholder="実行すると、ここに Wordmove のログが表示されます。" />
  </section>
</template>
