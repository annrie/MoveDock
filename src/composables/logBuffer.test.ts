import { afterEach, describe, expect, it, vi } from 'vitest'
import { watch } from 'vue'
import { createLogBuffer } from './logBuffer'

afterEach(() => vi.useRealTimers())
describe('log buffering', () => {
  it('publishes a burst once and retains the latest 2000 lines', () => {
    vi.useFakeTimers()
    const buffer = createLogBuffer()
    const updates = vi.fn()
    watch(buffer.logs, updates, { flush: 'sync' })
    for (let i = 0; i < 10000; i++) buffer.append({ stream: 'stdout', line: `file ${i}` })
    expect(updates).not.toHaveBeenCalled()
    vi.advanceTimersByTime(100)
    expect(updates).toHaveBeenCalledTimes(1)
    expect(buffer.logs.value).toHaveLength(2000)
    expect(buffer.logs.value[0].line).toBe('file 8000')
    expect(buffer.logs.value.at(-1)?.line).toBe('file 9999')
  })
  it('flushes the last output immediately on completion and resets pending output', () => {
    vi.useFakeTimers()
    const buffer = createLogBuffer()
    buffer.append({ stream: 'stdout', line: 'last output' })
    buffer.flush()
    expect(buffer.logs.value.at(-1)?.line).toBe('last output')
    buffer.append({ stream: 'stdout', line: 'old run' })
    buffer.reset([{ stream: 'system', line: 'new run' }])
    vi.runAllTimers()
    expect(buffer.logs.value.map(event => event.line)).toEqual(['new run'])
  })
})
