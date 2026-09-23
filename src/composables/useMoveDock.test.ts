import { beforeEach, describe, expect, it, vi } from 'vitest'
import { setLocale, t, supportedLocales } from '../i18n'
import { nextTick } from 'vue'
import { useMoveDock } from './useMoveDock'
import type { AppData, History, LogEvent } from '../types'

const mock = vi.hoisted(() => ({ invoke: vi.fn(), confirm: vi.fn(), open: vi.fn(), save: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: mock.invoke, isTauri: () => true, Channel: class { onmessage = (_: LogEvent) => {} } }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ confirm: mock.confirm, open: mock.open, save: mock.save }))
const fixture: AppData = {
  settings: { mode: 'direct', executable: '/tmp/mock-wordmove', gemfile: '', extraPath: '', rubyVersion: '', theme: 'light' },
  sites: [{ id: 'test', name: 'Example', path: '/tmp/Example/Movefile' }], history: [],
}
beforeEach(() => {
  setLocale('ja'); vi.resetAllMocks(); mock.confirm.mockResolvedValue(true)
  mock.invoke.mockImplementation(async (name: string) => {
    if (name === 'get_data') return structuredClone(fixture)
    if (name === 'inspect_site') return { local: 'https://example.local', environments: [{ name: 'staging', vhost: 'https://stage.example' }] }
    if (name === 'preview_run') return 'wordmove pull --themes --simulate'
    if (name === 'read_movefile') return { content: 'local: {}', revision: 'original-hash' }
  })
})
async function loaded() {
  const app = useMoveDock(); await app.initialize(); await app.selectSite(app.data.value.sites[0]); await app.loadEnvironments(); return app
}
describe('sync workflow', () => {
  it('does not evaluate a Movefile on startup or site selection', async () => {
    const app = useMoveDock(); await app.initialize(); await app.selectSite(app.data.value.sites[0])
    expect(app.ready.value).toBe(false)
    expect(mock.invoke.mock.calls.map(c => c[0])).toEqual(['get_data'])
    mock.confirm.mockResolvedValue(false); await app.loadEnvironments()
    expect(mock.invoke).not.toHaveBeenCalledWith('inspect_site', expect.anything())
  })
  it('resets database selection and direction when switching sites', async () => {
    const app = await loaded(); app.selectedTargets.value = ['db']; app.direction.value = 'push'
    await app.selectSite({ id: 'another', name: 'Another', path: '/tmp/Another/Movefile' })
    expect(app.selectedTargets.value).toEqual(['themes']); expect(app.direction.value).toBe('pull')
    expect(app.inspection.value).toBe(null)
  })
  it('cancelling the confirmation never starts synchronization', async () => {
    const app = await loaded(); expect(app.ready.value).toBe(true)
    mock.confirm.mockResolvedValue(false); await app.run(false)
    expect(mock.invoke.mock.calls.some(c => c[0] === 'run_sync')).toBe(false)
    expect(app.running.value).toBe(false)
  })
  it('streams early output, prevents double starts, and keeps simulate explicit', async () => {
    const app = await loaded()
    let complete!: (value: History) => void
    const runHandler = mock.invoke.getMockImplementation()!
    mock.invoke.mockImplementation(async (name, args) => {
      if (name !== 'run_sync') return runHandler(name, args)
      args.output.onmessage({ stream: 'stdout', line: 'first output' })
      return await new Promise<History>(resolve => { complete = resolve })
    })
    const pending = app.run(true)
    for (let i = 0; i < 8; i++) await nextTick()
    expect(app.running.value).toBe(true)
    await vi.waitFor(() => expect(app.logs.value.some(l => l.line === 'first output')).toBe(true))
    await app.run(false)
    expect(mock.invoke.mock.calls.filter(c => c[0] === 'run_sync')).toHaveLength(1)
    const request = mock.invoke.mock.calls.find(c => c[0] === 'run_sync')![1].request
    expect(request).toMatchObject({ simulate: true, direction: 'pull', environment: 'staging', targets: ['themes'] })
    complete({ id: 'done', siteName: 'Example', startedAt: '', finishedAt: '', request, status: 'success', exitCode: 0 })
    await pending; expect(app.running.value).toBe(false)
  })
  it('disables synchronization until edited Movefile is saved and reloaded', async () => {
    const app = await loaded(); await app.editMovefile(); app.document.value!.content += '\nstaging: {}'
    expect(app.dirty.value).toBe(true); expect(app.ready.value).toBe(false)
    mock.invoke.mockResolvedValueOnce({ content: app.document.value!.content, revision: 'new-hash' })
    await app.saveDocument(); expect(app.inspection.value).toBe(null); expect(app.ready.value).toBe(false)
  })
})

describe('automatic setup', () => {
  it('loads detected settings on startup and preserves them as saved values', async () => {
    const app = useMoveDock(); await app.initialize()
    expect(app.data.value.settings.executable).toBe('/tmp/mock-wordmove')
    expect(app.settingsDirty.value).toBe(false)
  })
  it('does not replace unsaved edits or diagnose different saved settings', async () => {
    const app = await loaded(); app.data.value.settings.executable = '/custom/wordmove'
    mock.invoke.mockClear(); await app.autofillSettings(); await app.checkTools()
    expect(mock.invoke).not.toHaveBeenCalled()
    expect(app.data.value.settings.executable).toBe('/custom/wordmove')
  })
  it('loads and saves newly detected fields, invalidating the old environment', async () => {
    const app = await loaded()
    mock.invoke.mockResolvedValueOnce({ ...structuredClone(fixture), settings: { ...fixture.settings, rubyVersion: '3.3.12' }, setupComplete: true, setupNotes: ['detected'] })
    await app.autofillSettings()
    expect(mock.invoke).toHaveBeenCalledWith('autofill_settings')
    expect(app.data.value.settings.rubyVersion).toBe('3.3.12')
    expect(app.settingsDirty.value).toBe(false); expect(app.inspection.value).toBe(null)
  })
})


describe('localized confirmations', () => {
  it('uses the active language for trust and overwrite confirmations in all eight locales', async () => {
    const app = useMoveDock(); await app.initialize(); await app.selectSite(app.data.value.sites[0])
    for (const locale of supportedLocales) {
      setLocale(locale.value); mock.confirm.mockResolvedValue(true)
      await app.loadEnvironments()
      expect(mock.confirm).toHaveBeenLastCalledWith(t('dialog.trustBody'), expect.objectContaining({ title: t('dialog.trustTitle'), cancelLabel: t('common.cancel') }))
      app.direction.value = 'push'; app.selectedTargets.value = ['db']
      mock.confirm.mockResolvedValue(false)
      await app.run(false)
      expect(mock.confirm).toHaveBeenLastCalledWith(t('dialog.syncBody', { destination: t('dialog.remote', { environment: 'staging' }), targets: t('target.db') }), expect.objectContaining({ title: t('dialog.syncTitle'), okLabel: t('common.execute', { direction: 'PUSH' }) }))
      expect(mock.invoke.mock.calls.some(call => call[0] === 'run_sync')).toBe(false)
    }
  })
  it('keeps loaded environments and unsaved connection settings when language changes', async () => {
    const app = await loaded(); const inspection = app.inspection.value
    const calls = mock.invoke.mock.calls.length
    setLocale('fr'); await nextTick()
    expect(app.ready.value).toBe(true); expect(app.settingsDirty.value).toBe(false)
    app.data.value.settings.executable = '/unsaved/custom-wordmove'
    setLocale('ko'); await nextTick()
    expect(app.inspection.value).toBe(inspection)
    expect(app.data.value.settings.executable).toBe('/unsaved/custom-wordmove')
    expect(app.settingsDirty.value).toBe(true)
    expect(mock.invoke.mock.calls).toHaveLength(calls)
  })
})
