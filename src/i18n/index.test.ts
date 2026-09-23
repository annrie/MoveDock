import { afterEach, describe, expect, it, vi } from 'vitest'
import { readFileSync } from 'node:fs'
import { detectLocale, formatDate, i18n, language, messages, setLocale, supportedLocales, t } from './index'
import { appMessage, errorHint, formatAppMessage, formatSetupNote } from './appMessages'
import type { Locale } from './index'

afterEach(() => { vi.restoreAllMocks(); setLocale('ja') })
const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map(match => match[1]).sort()
describe('locale catalogs', () => {
  it('covers exactly the eight YTDown locales, with complete keys and interpolation parameters', () => {
    expect(supportedLocales.map(l => l.value)).toEqual(['ja', 'en', 'zh-TW', 'fr', 'es', 'pt-BR', 'de', 'ko'])
    const keys = Object.keys(messages.en).sort()
    for (const locale of supportedLocales) {
      const catalog = messages[locale.value]
      expect(Object.keys(catalog).sort()).toEqual(keys)
      for (const key of keys as (keyof typeof catalog)[]) {
        expect(catalog[key].trim(), `${locale.value}: ${key}`).not.toBe('')
        expect(placeholders(catalog[key]), `${locale.value}: ${key}`).toEqual(placeholders(messages.en[key]))
        setLocale(locale.value)
        const args = Object.fromEntries(placeholders(catalog[key]).map(name => [name, `TEST_${name}`]))
        const rendered = t(key, args)
        expect(rendered).not.toBe(key)
        expect(rendered).not.toMatch(/\{\w+\}/)
        for (const name of placeholders(catalog[key])) expect(rendered).toContain(`TEST_${name}`)
      }
    }
  })
  it('has translations for every backend message code', () => {
    for (const file of ['cli', 'commands', 'discovery', 'process', 'store']) {
      const source = readFileSync(`src-tauri/src/${file}.rs`, 'utf8')
      for (const match of source.matchAll(/"((?:backend|setup)\.[\w]+)"/g)) expect(Object.hasOwn(messages.en, match[1]), match[1]).toBe(true)
    }
  })
})
describe('language preference', () => {
  it('respects saved choice, maps regional languages and falls back to English', () => {
    for (const locale of supportedLocales) expect(detectLocale(locale.value, 'ja-JP')).toBe(locale.value)
    for (const [system, expected] of [['ja-JP','ja'],['en-GB','en'],['zh-Hant-TW','zh-TW'],['zh-CN','zh-TW'],['pt-PT','pt-BR'],['de-AT','de'],['ko-KR','ko'],['fr-CA','fr'],['es-MX','es'],['ar','en']]) expect(detectLocale('invalid', system)).toBe(expected)
  })
  it('persists immediately, updates document language and ignores unsupported values', () => {
    setLocale('de'); expect(localStorage.getItem('movedock_language')).toBe('de')
    expect(document.documentElement.lang).toBe('de'); expect(t('nav.settings')).toBe('Einstellungen')
    setLocale('invalid'); expect(language.value).toBe('de')
    expect(formatDate('2026-09-23T13:00:00Z')).toBe(new Intl.DateTimeFormat('de', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }).format(new Date('2026-09-23T13:00:00Z')))
  })
  it('uses the English fallback for a missing translation', () => {
    const existing = i18n.global.getLocaleMessage('fr')
    i18n.global.setLocaleMessage('fr', {} as typeof existing)
    setLocale('fr'); expect(t('nav.settings')).toBe('Settings')
    i18n.global.setLocaleMessage('fr', existing)
  })
})
describe('application messages and original details', () => {
  it('changes the explanation live and preserves original command output', () => {
    const raw = 'Permission denied (publickey).\n/path/日本語/file.php'
    const error = appMessage('backend.commandFailed', { code: 23 }, raw)
    for (const locale of ['ja', 'en', 'de', 'ko'] as Locale[]) {
      setLocale(locale)
      expect(formatAppMessage(error)).toBe(t('backend.commandFailed', { code: 23 }) + '\n\n' + raw)
      expect(errorHint(error)).toBe(t('hint.ssh'))
    }
  })

  it('does not interpret raw command details that resemble an application message', () => {
    const raw = appMessage('backend.timeout')
    setLocale('en')
    expect(formatAppMessage(appMessage('backend.commandFailed', { code: 1 }, raw))).toBe('The command failed (exit code 1).\n\n' + raw)
    const nested = 'movedock-message-v1:' + JSON.stringify({ code: 'backend.bundleHint', params: {}, detail: '', cause: appMessage('backend.timeout') })
    expect(formatAppMessage(nested)).toContain('The command timed out.')
  })
  it('preserves unknown messages and malformed envelopes verbatim', () => {
    for (const raw of ['Original error', 'movedock-message-v1:broken', appMessage('unknown.code'), 'movedock-message-v1:{"code":"backend.timeout","params":null}']) expect(formatAppMessage(raw)).toBe(raw)
  })
  it('translates legacy persisted setup notes without replacing arbitrary text', () => {
    setLocale('en')
    expect(formatSetupNote(messages.ja['setup.detected'])).toBe(messages.en['setup.detected'])
    expect(formatSetupNote('bundle が見つかりません。導入後に「空欄を再検出」を実行してください。')).toBe(t('setup.executableMissing', { name: 'bundle' }))
    expect(formatSetupNote('/path/日本語')).toBe('/path/日本語')
  })
})
