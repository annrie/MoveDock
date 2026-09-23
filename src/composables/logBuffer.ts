import { shallowRef } from 'vue'
import type { LogEvent } from '../types'

export const LOG_LIMIT = 2000

export function createLogBuffer() {
  const logs = shallowRef<LogEvent[]>([])
  let pending: LogEvent[] = []
  let timer: ReturnType<typeof setTimeout> | undefined

  function flush() {
    if (timer !== undefined) clearTimeout(timer)
    timer = undefined
    if (!pending.length) return
    logs.value = logs.value.concat(pending).slice(-LOG_LIMIT)
    pending = []
  }

  function append(event: LogEvent) {
    pending.push(event)
    if (pending.length > LOG_LIMIT) pending.splice(0, pending.length - LOG_LIMIT)
    if (timer === undefined) timer = setTimeout(flush, 100)
  }

  function reset(initial: LogEvent[] = []) {
    if (timer !== undefined) clearTimeout(timer)
    timer = undefined
    pending = []
    logs.value = initial.slice(-LOG_LIMIT)
  }

  return { logs, append, flush, reset }
}
