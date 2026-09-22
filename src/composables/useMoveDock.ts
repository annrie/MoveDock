import { computed, ref, watch } from 'vue'
import { Channel, invoke, isTauri } from '@tauri-apps/api/core'
import { confirm, open, save } from '@tauri-apps/plugin-dialog'
import type { AppData, Diagnostic, Document, History, Inspection, LogEvent, RunRequest, Site } from '../types'
import { statusLabel, targets } from '../types'

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
  const logs = ref<LogEvent[]>([])
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
  async function initialize() { if (native) await task('読み込み中', refresh) }
  async function discard(): Promise<boolean> {
    return !dirty.value || await confirm('Movefile の未保存の変更を破棄しますか？', { title: '未保存の変更', kind: 'warning', okLabel: '破棄する', cancelLabel: '戻る' })
  }
  async function selectSite(next: Site) {
    if (locked.value || !await discard()) return
    selectedId.value = next.id; selectedTargets.value = ['themes']; direction.value = 'pull'; lastResult.value = null; inspection.value = null; environment.value = ''; document.value = null; original.value = ''; view.value = 'sync'; error.value = ''; notice.value = ''
  }
  async function addSite(create = false) {
    await task('サイトを追加中', async () => {
      if (!await discard()) return
      const path = create ? await save({ title: 'Movefile を新規作成', defaultPath: 'movefile.yml' }) : await open({ title: 'Movefile を選択', multiple: false, directory: false })
      if (typeof path !== 'string') return
      if (create) await invoke('create_movefile', { path })
      const name = path.split('/').slice(-2, -1)[0] || 'WordPress site'
      const next = await invoke<Site>('add_site', { path, name })
      await refresh(); selectedId.value = next.id; selectedTargets.value = ['themes']; direction.value = 'pull'; lastResult.value = null; inspection.value = null; document.value = null; environment.value = ''; view.value = create ? 'editor' : 'sync'
      if (create) { document.value = await invoke<Document>('read_movefile', { id: next.id }); original.value = document.value.content }
    })
  }
  async function loadEnvironments() {
    await task('環境を読み込み中', async () => {
      if (!site.value || dirty.value) return
      const allowed = await confirm('この Movefile を信頼して環境を読み込みますか？\n\nMovefile に含まれる ERB は Ruby コードとして実行されます。内容が不明な場合は「Movefile」タブで先に確認してください。', { title: 'Movefile の読み込み', kind: 'warning', okLabel: '信頼して読み込む', cancelLabel: 'キャンセル' })
      if (!allowed) return
      inspection.value = null
      inspection.value = await invoke<Inspection>('inspect_site', { id: site.value.id })
      environment.value = inspection.value.environments[0]?.name ?? ''
    })
  }
  async function editMovefile() {
    if (!site.value) return
    await task('Movefile を開いています', async () => {
      view.value = 'editor'
      if (!document.value) { document.value = await invoke<Document>('read_movefile', { id: site.value!.id }); original.value = document.value.content }
    })
  }
  async function reloadDocument() {
    await task('再読み込み中', async () => {
      if (!site.value || !await discard()) return
      document.value = await invoke<Document>('read_movefile', { id: site.value.id }); original.value = document.value.content
      inspection.value = null; environment.value = ''
    })
  }
  async function saveDocument() {
    await task('保存中', async () => {
      if (!site.value || !document.value) return
      document.value = await invoke<Document>('save_movefile', { id: site.value.id, ...document.value })
      original.value = document.value.content; inspection.value = null; environment.value = ''; notice.value = '保存しました。同期前に環境を再読み込みしてください。'
    })
  }
  async function removeSite() {
    await task('登録を解除中', async () => {
      if (!site.value || !await discard()) return
      if (!await confirm(`${site.value.name} の登録を解除しますか？\nMovefile とサイトのファイルは削除されません。`, { title: 'サイトの登録解除', okLabel: '登録解除', cancelLabel: 'キャンセル' })) return
      await invoke('remove_site', { id: site.value.id }); await refresh(); selectedId.value = ''; document.value = null; inspection.value = null; view.value = 'sync'
    })
  }
  async function saveSettings() {
    await task('設定を保存中', async () => { await invoke('save_settings', { settings: { ...data.value.settings } }); savedSettings.value = JSON.stringify(data.value.settings); inspection.value = null; environment.value = ''; diagnostics.value = []; notice.value = '設定を保存しました。' })
  }
  async function checkTools() { await task('実行環境を確認中', async () => { diagnostics.value = await invoke<Diagnostic[]>('diagnostics') }) }
  async function chooseGemfile() { const path = await open({ title: 'Wordmove の Gemfile を選択', multiple: false }); if (typeof path === 'string') data.value.settings.gemfile = path }
  async function run(simulate: boolean) {
    if (!ready.value || !site.value) return
    await task('実行内容を確認中', async () => {
      const request: RunRequest = { siteId: site.value!.id, environment: environment.value, direction: direction.value, targets: [...selectedTargets.value], simulate }
      const preview = await invoke<string>('preview_run', { request })
      const destination = request.direction === 'push' ? `${request.environment}（リモート）` : 'ローカル'
      const targetNames = targets.filter(t => request.targets.includes(t.id)).map(t => t.label).join('、')
      const text = simulate ? `シミュレーションを実行します。\n${request.direction.toUpperCase()} → ${destination}\n対象: ${targetNames}\n\nERB の評価と接続は発生します。同期結果を保証するものではありません。` : `${destination} のデータを上書きする同期を実行します。\n対象: ${targetNames}\n\nファイルの削除や DB の置き換えが発生する場合があります。必要なバックアップを確認してください。`
      if (!await confirm(text, { title: simulate ? 'シミュレーション' : '同期の実行確認', kind: 'warning', okLabel: simulate ? 'シミュレーション実行' : `${request.direction.toUpperCase()} を実行`, cancelLabel: 'キャンセル' })) return
      logs.value = [{ stream: 'system', line: `${site.value!.name} · ${site.value!.path}` }, { stream: 'system', line: preview }]; lastResult.value = null; running.value = true; stopping.value = false
      const output = new Channel<LogEvent>()
      output.onmessage = event => { logs.value.push(event); if (logs.value.length > 2000) logs.value.splice(0, logs.value.length - 2000) }
      try {
        lastResult.value = await invoke<History>('run_sync', { request, output })
        logs.value.push({ stream: 'system', line: `${statusLabel[lastResult.value.status]} / 終了コード: ${lastResult.value.exitCode ?? '—'}` })
        await refresh()
      } finally { running.value = false; stopping.value = false }
    })
  }
  async function stop() {
    if (!running.value || stopping.value) return
    stopping.value = true
    try { await invoke('cancel_run') } catch (e) { error.value = String(e); stopping.value = false }
  }
  return { native, data, settingsDirty, site, view, inspection, environment, direction, selectedTargets, document, dirty, busy, running, stopping, error, notice, logs, diagnostics, lastResult, locked, remote, ready, initialize, selectSite, addSite, loadEnvironments, editMovefile, reloadDocument, saveDocument, removeSite, saveSettings, checkTools, chooseGemfile, run, stop }
}
