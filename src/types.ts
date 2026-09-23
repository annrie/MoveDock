export interface Settings { mode: 'bundler' | 'direct'; executable: string; gemfile: string; extraPath: string; rubyVersion: string; theme: 'system' | 'light' | 'dark' }
export interface Site { id: string; name: string; path: string }
export interface RunRequest { siteId: string; environment: string; direction: 'push' | 'pull'; targets: string[]; simulate: boolean }
export interface History { id: string; siteName: string; startedAt: string; finishedAt: string; request: RunRequest; status: string; exitCode: number | null }
export interface AppData { settings: Settings; sites: Site[]; history: History[]; setupComplete?: boolean; setupNotes?: string[] }
export interface Inspection { local: string; environments: { name: string; vhost: string }[] }
export interface Document { content: string; revision: string }
export interface LogEvent { stream: string; line: string }
export interface Diagnostic { name: string; available: boolean; detail: string }
export const targets = [
  { id: 'themes', detail: 'themes' }, { id: 'plugins', detail: 'plugins' },
  { id: 'uploads', detail: 'uploads' }, { id: 'mu_plugins', detail: 'mu-plugins' },
  { id: 'languages', detail: 'languages' }, { id: 'wordpress', detail: 'core files' },
  { id: 'db', detail: 'database' },
]
