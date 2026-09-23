import { computed } from 'vue'
import { createI18n } from 'vue-i18n'
import ja from '../locales/ja.json'
import en from '../locales/en.json'
import zhTW from '../locales/zh-TW.json'
import fr from '../locales/fr.json'
import es from '../locales/es.json'
import ptBR from '../locales/pt-BR.json'
import de from '../locales/de.json'
import ko from '../locales/ko.json'

export const supportedLocales = [
  { value: 'ja', label: '日本語' }, { value: 'en', label: 'English' },
  { value: 'zh-TW', label: '繁體中文' }, { value: 'fr', label: 'Français' },
  { value: 'es', label: 'Español' }, { value: 'pt-BR', label: 'Português (BR)' },
  { value: 'de', label: 'Deutsch' }, { value: 'ko', label: '한국어' },
] as const
export type Locale = typeof supportedLocales[number]['value']
export type MessageKey = keyof typeof en
export const messages = { ja, en, 'zh-TW': zhTW, fr, es, 'pt-BR': ptBR, de, ko }
const storageKey = 'movedock_language'
export function isLocale(value: unknown): value is Locale {
  return supportedLocales.some(entry => entry.value === value)
}
export function detectLocale(saved: string | null, systemLanguage: string): Locale {
  if (isLocale(saved)) return saved
  const language = systemLanguage.toLowerCase().split(/[-_]/)[0]
  if (language === 'zh') return 'zh-TW'
  if (language === 'pt') return 'pt-BR'
  return isLocale(language) ? language : 'en'
}
function initialLocale(): Locale {
  let saved: string | null = null
  try { saved = localStorage.getItem(storageKey) } catch { /* Storage may be unavailable in previews. */ }
  return detectLocale(saved, navigator.language)
}
export const i18n = createI18n({ legacy: false, locale: initialLocale(), fallbackLocale: 'en', messages: JSON.parse(JSON.stringify(messages)), flatJson: true })
export function t(key: string, values: Record<string, string | number> = {}): string {
  return i18n.global.t(key, values)
}
export function setLocale(value: string) {
  if (!isLocale(value)) return
  i18n.global.locale.value = value
  document.documentElement.lang = value
  try { localStorage.setItem(storageKey, value) } catch { /* Keep the selected language for this session. */ }
}
export const language = computed({ get: () => i18n.global.locale.value, set: setLocale })
export function formatDate(value: string): string {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : new Intl.DateTimeFormat(language.value, { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }).format(date)
}
export function formatList(values: string[]): string {
  return typeof Intl.ListFormat === 'function'
    ? new Intl.ListFormat(language.value, { style: 'short', type: 'conjunction' }).format(values)
    : values.join(', ')
}
export function statusText(status: string): string {
  return ['success', 'failed', 'cancelled', 'timeout'].includes(status) ? t(`status.${status}`) : status
}
