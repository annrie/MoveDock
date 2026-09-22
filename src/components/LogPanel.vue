<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import type { LogEvent } from '../types'
import AppIcon from './AppIcon.vue'
const props = defineProps<{ logs: LogEvent[]; running: boolean; stopping: boolean }>()
defineEmits<{ stop: [] }>()
const output = ref<HTMLElement>()
const follow = ref(true)
watch(() => props.logs.at(-1), async () => { if (follow.value) { await nextTick(); output.value?.scrollTo({ top: output.value.scrollHeight }) } })
</script>
<template>
  <section class="log-panel" aria-labelledby="log-heading">
    <div class="log-toolbar"><h2 id="log-heading"><AppIcon name="terminal" :size="16" /> 実行ログ <span class="log-count">{{ logs.length }}</span></h2><div class="row"><label class="follow"><input v-model="follow" type="checkbox" /> 自動スクロール</label><button v-if="running" class="stop-button" :disabled="stopping" @click="$emit('stop')"><AppIcon name="stop" :size="13" />{{ stopping ? '停止しています…' : '停止' }}</button><span v-else class="log-idle">待機中</span></div></div>
    <div ref="output" class="log-output" role="region" aria-label="実行ログ。最新 2000 行を表示" tabindex="0"><div v-if="!logs.length" class="log-empty"><span class="prompt">›</span> 実行すると、ここに Wordmove のログが表示されます。<span class="cursor" /></div><div v-for="(entry, i) in logs" :key="i" class="log-line" :class="entry.stream"><span class="line-number">{{ String(i + 1).padStart(3, '0') }}</span><span>{{ entry.line }}</span></div></div>
  </section>
</template>
