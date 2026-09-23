import { computed, ref, watch } from 'vue'
import { Channel, invoke, isTauri } from '@tauri-apps/api/core'
import { confirm, open, save } from '@tauri-apps/plugin-dialog'
import type { AppData, Diagnostic, Document, History, Inspection, LogEvent, RunRequest, Site } from '../types'
import { targets } from '../types'
import { t, formatList } from '../i18n'
import { appMessage } from '../i18n/appMessages'
import { createLogBuffer } from './logBuffer'

export function useMoveDock() {
  const native = isTauri()
  const data = ref<AppData>({ settings: { mode: 'bundler', executable: '', gemfile: '', extraPath: '', rubyVersion: '', theme: 'system' }, sites: [], history: [] })
  const savedSettings = ref(JSON.stringify(data.value.settings))
  const settingsDirty = computed(() => JSON.stringify(data.value.settings) !== savedSettings.value)
  const selectedId = ref('')
  const site = computed(() => data.value.sites.find(s => s.id === selectedId.value))
  const view = ref<'sync' | 'editor' | 'history' | 'settings'>('sync')
  const inspection = ref<Inspection | null>(null)
  const environment = ref('')
  const direction = ref<'pull' | 'push'>('pull')
  const selectedTargets = ref<string[]>(['themes'])
  const document = ref<Document | null>(null)
  const original = ref('')
  const dirty = computed(() => document.value !== null && document.value.content !== original.value)
  const busy = ref('')
  const running = ref(false)
  const stopping = ref(false)
  const error = ref('')
  const notice = ref('')
  const logBuffer = createLogBuffer()
  const { logs } = logBuffer
  const runError = ref('')
  const diagnostics = ref<Diagnostic[]>([])
  const lastResult = ref<History | null>(null)
  const locked = computed(() => !!busy.value || running.value)
  const remote = computed(() => inspection.value?.environments.find(e => e.name === environment.value))
  const ready = computed(() => native && !!site.value && !!remote.value && selectedTargets.value.length > 0 && !locked.value && !dirty.value && !settingsDirty.value)

  watch(() => data.value.settings.theme, theme => { globalThis.document.documentElement.dataset.theme = theme }, { immediate: true })
  async function task<T>(label: string, action: () => Promise<T>): Promise<T | undefined> {
    if (locked.value) return
    error.value = ''; notice.value = ''; busy.value = label
    try { return await action() } catch (e) { error.value = String(e); return undefined } finally { busy.value = '' }
  }
  async function refresh() { data.value = await invoke<AppData>('get_data'); savedSettings.value = JSON.stringify(data.value.settings) }
  async function initialize() { if (native) await task('busy.loading', refresh) }
  async function discard(): Promise<boolean> {
    return !dirty.value || await confirm(t('dialog.discardBody'), { title: t('dialog.unsaved'), kind: 'warning', okLabel: t('dialog.discard'), cancelLabel: t('common.back') })
  }
  async function selectSite(next: Site) {
    if (locked.value || !await discard()) return
    selectedId.value = next.id; selectedTargets.value = ['themes']; direction.value = 'pull'; lastResult.value = null; inspection.value = null; environment.value = ''; document.value = null; original.value = ''; view.value = 'sync'; error.value = ''; notice.value = ''
  }
  async function addSite(create = false) {
    await task('busy.adding', async () => {
      if (!await discard()) return
      const path = create ? await save({ title: t('dialog.create'), defaultPath: 'movefile.yml' }) : await open({ title: t('dialog.open'), multiple: false, directory: false })
      if (typeof path !== 'string') return
      if (create) await invoke('create_movefile', { path })
      const name = path.split('/').slice(-2, -1)[0] || 'WordPress site'
      const next = await invoke<Site>('add_site', { path, name })
      await refresh(); selectedId.value = next.id; selectedTargets.value = ['themes']; direction.value = 'pull'; lastResult.value = null; inspection.value = null; document.value = null; environment.value = ''; view.value = create ? 'editor' : 'sync'
      if (create) { document.value = await invoke<Document>('read_movefile', { id: next.id }); original.value = document.value.content }
    })
  }
  async function loadEnvironments() {
    await task('busy.environments', async () => {
      if (!site.value || dirty.value) return
      const allowed = await confirm(t('dialog.trustBody'), { title: t('dialog.trustTitle'), kind: 'warning', okLabel: t('dialog.trust'), cancelLabel: t('common.cancel') })
      if (!allowed) return
      inspection.value = null
      inspection.value = await invoke<Inspection>('inspect_site', { id: site.value.id })
      environment.value = inspection.value.environments[0]?.name ?? ''
    })
  }
  async function editMovefile() {
    if (!site.value) return
    await task('busy.opening', async () => {
      view.value = 'editor'
      if (!document.value) { document.value = await invoke<Document>('read_movefile', { id: site.value!.id }); original.value = document.value.content }
    })
  }
  async function reloadDocument() {
    await task('busy.reloading', async () => {
      if (!site.value || !await discard()) return
      document.value = await invoke<Document>('read_movefile', { id: site.value.id }); original.value = document.value.content
      inspection.value = null; environment.value = ''
    })
  }
  async function saveDocument() {
    await task('busy.saving', async () => {
      if (!site.value || !document.value) return
      document.value = await invoke<Document>('save_movefile', { id: site.value.id, ...document.value })
      original.value = document.value.content; inspection.value = null; environment.value = ''; notice.value = 'notice.movefileSaved'
    })
  }
  async function removeSite() {
    await task('busy.removing', async () => {
      if (!site.value || !await discard()) return
      if (!await confirm(t('dialog.removeBody', { name: site.value.name }), { title: t('dialog.removeTitle'), okLabel: t('dialog.remove'), cancelLabel: t('common.cancel') })) return
      await invoke('remove_site', { id: site.value.id }); await refresh(); selectedId.value = ''; document.value = null; inspection.value = null; view.value = 'sync'
    })
  }
  async function saveSettings() {
    await task('busy.settings', async () => { await invoke('save_settings', { settings: { ...data.value.settings } }); savedSettings.value = JSON.stringify(data.value.settings); inspection.value = null; environment.value = ''; diagnostics.value = []; data.value.setupNotes = []; notice.value = 'notice.settingsSaved' })
  }
  async function autofillSettings() {
    if (settingsDirty.value) return
    await task('busy.detecting', async () => {
      data.value = await invoke<AppData>('autofill_settings')
      savedSettings.value = JSON.stringify(data.value.settings)
      inspection.value = null; environment.value = ''; diagnostics.value = []
      notice.value = 'notice.detected'
    })
  }
  async function checkTools() { if (settingsDirty.value) return; await task('busy.checking', async () => { diagnostics.value = await invoke<Diagnostic[]>('diagnostics') }) }
  async function chooseGemfile() { const path = await open({ title: t('dialog.gemfile'), multiple: false }); if (typeof path === 'string') data.value.settings.gemfile = path }
  async function run(simulate: boolean) {
    if (!ready.value || !site.value) return
    await task('busy.confirming', async () => {
      const request: RunRequest = { siteId: site.value!.id, environment: environment.value, direction: direction.value, targets: [...selectedTargets.value], simulate }
      const preview = await invoke<string>('preview_run', { request })
      const destination = request.direction === 'push' ? t('dialog.remote', { environment: request.environment }) : t('common.local')
      const targetNames = formatList(targets.filter(target => request.targets.includes(target.id)).map(target => t(`target.${target.id}`)))
      const text = t(simulate ? 'dialog.simulateBody' : 'dialog.syncBody', { direction: request.direction.toUpperCase(), destination, targets: targetNames })
      if (!await confirm(text, { title: simulate ? t('common.simulation') : t('dialog.syncTitle'), kind: 'warning', okLabel: simulate ? t('dialog.simulate') : t('common.execute', { direction: request.direction.toUpperCase() }), cancelLabel: t('common.cancel') })) return
      logBuffer.reset([{ stream: 'system', line: `${site.value!.name} · ${site.value!.path}` }, { stream: 'system', line: preview }]); lastResult.value = null; runError.value = ''; running.value = true; stopping.value = false
      const output = new Channel<LogEvent>()
      output.onmessage = event => logBuffer.append(event)
      try {
        lastResult.value = await invoke<History>('run_sync', { request, output })
        logBuffer.append({ stream: 'system', line: appMessage('log.finished', { status: lastResult.value.status, code: lastResult.value.exitCode ?? '—' }) })
        await refresh()
      } catch (e) {
        runError.value = String(e)
        logBuffer.append({ stream: 'system', line: runError.value })
        throw e
      } finally { logBuffer.flush(); running.value = false; stopping.value = false }
    })
  }
  async function stop() {
    if (!running.value || stopping.value) return
    stopping.value = true
    try { await invoke('cancel_run') } catch (e) { error.value = String(e); stopping.value = false }
  }
  return { native, data, settingsDirty, site, view, inspection, environment, direction, selectedTargets, document, dirty, busy, running, stopping, error, notice, logs, diagnostics, lastResult, runError, locked, remote, ready, initialize, selectSite, addSite, loadEnvironments, editMovefile, reloadDocument, saveDocument, removeSite, saveSettings, autofillSettings, checkTools, chooseGemfile, run, stop }
}
