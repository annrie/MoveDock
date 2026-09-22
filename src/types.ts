export interface Settings { mode: 'bundler' | 'direct'; executable: string; gemfile: string; extraPath: string; rubyVersion: string; theme: 'system' | 'light' | 'dark' }
export interface Site { id: string; name: string; path: string }
export interface RunRequest { siteId: string; environment: string; direction: 'push' | 'pull'; targets: string[]; simulate: boolean }
export interface History { id: string; siteName: string; startedAt: string; finishedAt: string; request: RunRequest; status: string; exitCode: number | null }
export interface AppData { settings: Settings; sites: Site[]; history: History[] }
export interface Inspection { local: string; environments: { name: string; vhost: string }[] }
export interface Document { content: string; revision: string }
export interface LogEvent { stream: string; line: string }
export interface Diagnostic { name: string; available: boolean; detail: string }
export const targets = [
  { id: 'themes', label: 'テーマ', detail: 'themes' }, { id: 'plugins', label: 'プラグイン', detail: 'plugins' },
  { id: 'uploads', label: 'メディア', detail: 'uploads' }, { id: 'mu_plugins', label: '必須プラグイン', detail: 'mu-plugins' },
  { id: 'languages', label: '言語ファイル', detail: 'languages' }, { id: 'wordpress', label: 'WordPress 本体', detail: 'core files' },
  { id: 'db', label: 'データベース', detail: 'database' },
]
export const statusLabel: Record<string, string> = { success: '正常終了', failed: '失敗', cancelled: '停止', timeout: 'タイムアウト' }
