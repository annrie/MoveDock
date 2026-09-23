import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { appMessage } from '../i18n/appMessages'
import { setLocale } from '../i18n'
import LogPanel from './LogPanel.vue'
import type { History } from '../types'

const props = { logs: [{ stream: 'stdout', line: 'last file.php' }], running: false, stopping: false, result: null, runError: '' }
const result: History = { id: 'done', siteName: 'Example', startedAt: '', finishedAt: '', request: { siteId: 'site', environment: 'staging', direction: 'pull', targets: ['themes'], simulate: true }, status: 'success', exitCode: 0 }
beforeEach(() => setLocale('ja'))
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks() })

describe('execution log', () => {
  it('renders many long log lines in one selectable field', () => {
    const logs = Array.from({ length: 2000 }, (_, i) => ({ stream: 'stdout', line: `${i} ${'path/'.repeat(100)}` }))
    const wrapper = mount(LogPanel, { props: { ...props, logs } })
    expect(wrapper.findAll('textarea')).toHaveLength(1)
    expect(wrapper.findAll('.log-line')).toHaveLength(0)
    expect(wrapper.get('textarea').element.value.split('\n')).toHaveLength(2000)
    expect(wrapper.get('textarea').attributes('readonly')).toBeDefined()
    expect(wrapper.get('textarea').attributes('wrap')).toBe('off')
    wrapper.unmount()
  })
  it('shows completion independently from the last filename', async () => {
    const wrapper = mount(LogPanel, { props: { ...props, running: true } })
    expect(wrapper.get('.log-status').text()).toBe('実行中')
    await wrapper.setProps({ running: false, result })
    expect(wrapper.get('.log-status').text()).toBe('シミュレーション完了 · 終了コード 0')
    await wrapper.setProps({ result: { ...result, status: 'failed', exitCode: 23 } })
    expect(wrapper.get('.log-status').text()).toContain('失敗 · 終了コード 23')
    wrapper.unmount()
  })
  it('copies the result and log text without line numbers', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    const wrapper = mount(LogPanel, { props: { ...props, result } })
    await wrapper.get('.log-copy').trigger('click')
    await flushPromises()
    expect(writeText).toHaveBeenCalledWith('シミュレーション完了 · 終了コード 0\nlast file.php')
    expect(wrapper.text()).toContain('コピーしました')
    wrapper.unmount()
  })
  it('falls back to a user-initiated copy in WebViews without the Clipboard API', async () => {
    vi.stubGlobal('navigator', {})
    const copy = vi.fn().mockReturnValue(true)
    Object.defineProperty(document, 'execCommand', { configurable: true, value: copy })
    const wrapper = mount(LogPanel, { props })
    await wrapper.get('.log-copy').trigger('click')
    await flushPromises()
    expect(copy).toHaveBeenCalledWith('copy')
    expect(wrapper.text()).toContain('コピーしました')
    expect(document.querySelectorAll('body > textarea')).toHaveLength(0)
    wrapper.unmount()
  })
  it('reports a copy failure and keeps the log selectable', async () => {
    vi.stubGlobal('navigator', {})
    Object.defineProperty(document, 'execCommand', { configurable: true, value: () => false })
    const wrapper = mount(LogPanel, { props })
    await wrapper.get('.log-copy').trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('コピーできませんでした')
    expect(wrapper.get('textarea').element.value).toBe('last file.php')
    wrapper.unmount()
  })
})


describe('localized logs', () => {
  it('updates controls and app messages while leaving stdout and stderr untouched', async () => {
    const raw = 'Permission denied (publickey). 日本語のファイル.php'
    const envelope = appMessage('backend.timeout')
    const wrapper = mount(LogPanel, { props: { ...props, result: { ...result, status: 'failed', exitCode: 23 }, logs: [{ stream: 'stderr', line: raw }, { stream: 'stdout', line: envelope }, { stream: 'system', line: envelope }] } })
    setLocale('en'); await flushPromises()
    expect(wrapper.get('.log-copy').text()).toBe('Copy log')
    expect(wrapper.get('textarea').element.value).toBe(raw + '\n' + envelope + '\nThe command timed out.')
    expect(wrapper.text()).toContain('SSH connection failed.')
    setLocale('de'); await flushPromises()
    expect(wrapper.get('.log-copy').text()).toBe('Protokoll kopieren')
    expect(wrapper.get('textarea').element.value).toContain(raw)
    expect(wrapper.get('.log-status').text()).toContain('Fehlgeschlagen')
    wrapper.unmount()
  })
})
