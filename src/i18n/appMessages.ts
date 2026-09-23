import { t, statusText } from './index'
import ja from '../locales/ja.json'

// Keep the existing string IPC/storage contract while giving app messages stable identities.
const prefix = 'movedock-message-v1:'
interface AppMessage { code: string; params: Record<string, string | number>; detail: string; cause?: string }
export function appMessage(code: string, params: Record<string, string | number> = {}, detail = ''): string {
  return prefix + JSON.stringify({ code, params, detail })
}
function parse(value: string): AppMessage | undefined {
  if (!value.startsWith(prefix)) return
  try {
    const message = JSON.parse(value.slice(prefix.length))
    if (typeof message.code !== 'string' || !Object.prototype.hasOwnProperty.call(ja, message.code) ||
        !message.params || typeof message.params !== 'object' || Array.isArray(message.params) ||
        !Object.values(message.params).every(v => typeof v === 'string' || typeof v === 'number') ||
        typeof message.detail !== 'string' || (message.cause !== undefined && typeof message.cause !== 'string')) return
    return message
  } catch { return }
}
export function formatAppMessage(value: string, depth = 0): string {
  if (depth > 8) return value
  const message = parse(value)
  if (!message) return value
  const params = message.code === 'log.finished' ? { ...message.params, status: statusText(String(message.params.status)) } : message.params
  const text = t(message.code, params)
  return [text, message.detail, message.cause ? formatAppMessage(message.cause, depth + 1) : ''].filter(Boolean).join('\n\n')
}
// v0.1.0 saved setup explanations as Japanese strings. Do not rewrite users' settings.
const legacyNotes = Object.entries(ja).filter(([key]) => key.startsWith('setup.') && key !== 'setup.executableMissing')
export function formatSetupNote(value: string): string {
  const legacy = legacyNotes.find(([, text]) => text === value)
  if (legacy) return t(legacy[0])
  const missing = value.match(/^(.+) が見つかりません。導入後に「空欄を再検出」を実行してください。$/)
  if (missing) return t('setup.executableMissing', { name: missing[1] })
  return formatAppMessage(value)
}
export function errorHint(value: string): string {
  if (/Permission denied \(publickey|Authentication failed|Host key verification failed/i.test(value)) return t('hint.ssh')
  if (/not parsable|Psych::SyntaxError|did not find expected key/i.test(value)) return t('hint.yaml')
  if (/Connection refused|Could not resolve hostname|No route to host/i.test(value)) return t('hint.connection')
  return ''
}
