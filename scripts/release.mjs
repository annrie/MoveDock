import { readFileSync, writeFileSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const level = process.argv[2]
if (!['patch', 'minor', 'major'].includes(level)) throw new Error('Usage: node scripts/release.mjs patch|minor|major')
const root = process.cwd()
const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim()
if (git('status', '--porcelain')) throw new Error('先に作業内容をコミットしてください。')
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'))
if (pkg.name !== 'movedock') throw new Error('MoveDock リポジトリで実行してください。')
const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(pkg.version)
if (!match) throw new Error('安定版の x.y.z 形式のバージョンが必要です。')
const [major, minor, patch] = match.slice(1).map(Number)
// Keep normal SemVer behavior even for 0.x (changelogen otherwise remaps minor to patch).
const version = level === 'major' ? `${major + 1}.0.0` : level === 'minor' ? `${major}.${minor + 1}.0` : `${major}.${minor}.${patch + 1}`
if (git('tag', '--list', `v${version}`)) throw new Error(`タグ v${version} は既に存在します。`)

const replacements = [
  ['src/App.vue', /(<span class="version">)[^<]+(<\/span>)/],
  ['src-tauri/Cargo.toml', /(\[package\][\s\S]*?\nversion = ")[^"]+("\n)/],
  ['src-tauri/Cargo.lock', /(\[\[package\]\]\nname = "movedock"\nversion = ")[^"]+("\n)/],
]
// Validate all files before changelogen updates package.json.
const updates = replacements.map(([file, pattern]) => {
  const content = readFileSync(join(root, file), 'utf8')
  if (!pattern.test(content)) throw new Error(`${file} のバージョン欄を確認してください。`)
  return [file, content.replace(pattern, (_, before, after) => `${before}${version}${after}`)]
})
const config = JSON.parse(readFileSync(join(root, 'src-tauri/tauri.conf.json'), 'utf8'))
config.version = version
updates.push(['src-tauri/tauri.conf.json', JSON.stringify(config, null, 2) + '\n'])
const cli = join(dirname(fileURLToPath(import.meta.resolve('changelogen'))), 'cli.mjs')
execFileSync(process.execPath, [cli, '--bump', '-r', version], { cwd: root, stdio: 'inherit' })
for (const [file, contents] of updates) writeFileSync(join(root, file), contents)
const files = ['package.json', 'CHANGELOG.md', ...updates.map(([file]) => file)]
git('add', '--', ...files)
const message = (pkg.changelog?.templates?.commitMessage || 'chore(release): v{{newVersion}}').replaceAll('{{newVersion}}', version)
git('commit', '-m', message)
git('tag', '-a', `v${version}`, '-m', `v${version}`)
console.log(`MoveDock v${version}: changelog、各バージョン、ローカルのコミットとタグを更新しました。`)
