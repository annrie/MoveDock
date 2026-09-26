# 変更履歴 / Changelog


## v0.2.1

[compare changes](https://github.com/annrie/MoveDock/compare/v0.2.0...v0.2.1)

### 📖 ドキュメント / Documentation

- Record v0.2.0 verification in Japanese and English ([c4d18b5](https://github.com/annrie/MoveDock/commit/c4d18b5))

### 📦 ビルド / Build

- 🔧 共通リリーススクリプトへ移行 / Harden release script and share it with the other Tauri apps ([#1](https://github.com/annrie/MoveDock/pull/1))
- **deps:** ⬆️ tauri 2.11.6(セキュリティ修正)・vite 8.3.1・vue 3.5.43 ほか minor/patch 一括更新 / Bump tauri to 2.11.6 (security fix), vite, vue and other minor/patch deps ([fadd25d](https://github.com/annrie/MoveDock/commit/fadd25d))

### 🧹 ビルドプロセスまたは補助ツールの変更 / Maintenance

- 🔧 Codex アプリのプロジェクト設定を追跡 / Track Codex app project settings (.codex) ([a893723](https://github.com/annrie/MoveDock/commit/a893723))

### ❤️ Contributors

- Annrie ([@annrie](https://github.com/annrie))

## v0.2.0

[compare changes](https://github.com/annrie/MoveDock/compare/v0.1.0...v0.2.0)

### 🚀 新機能 / Features

- 8言語の画面・エラー案内に対応 / Add eight-language UI and localized error guidance ([df95735](https://github.com/annrie/MoveDock/commit/df95735))

### 📖 ドキュメント / Documentation

- 推奨 Wordmove を v5.3.0.pre.2 に更新 / Recommend Wordmove v5.3.0.pre.2 ([881594c](https://github.com/annrie/MoveDock/commit/881594c))
- 実環境での同期確認を明記 / Document real-world sync verification ([4f057b2](https://github.com/annrie/MoveDock/commit/4f057b2))

### ❤️ Contributors

- Annrie ([@annrie](https://github.com/annrie))

## v0.1.0 — 2026-09-23

[リリース説明 / Release notes](docs/releases/v0.1.0.md)


### 🚀 新機能 / Features

- Wordmove のデスクトップアプリ MoveDock を追加 / Add MoveDock Wordmove desktop app ([972d5d1](https://github.com/annrie/MoveDock/commit/972d5d1))
- Wordmove の自動設定と文字の読みやすさを改善 / Auto-configure Wordmove and improve text readability ([b03a781](https://github.com/annrie/MoveDock/commit/b03a781))

### 🐛 バグ修正 / Bug fixes

- 見出し以外の文字サイズを拡大 / Increase non-heading text size by one point ([9ea925d](https://github.com/annrie/MoveDock/commit/9ea925d))
- 環境読み込みの起動待ち上限を延長（初回タイムアウトは調査中） / Allow longer Wordmove startup during environment loading (initial timeout still under investigation) ([e7dbe13](https://github.com/annrie/MoveDock/commit/e7dbe13))
- ログ描画の停止を防止し、実行結果のコピーを追加 / Prevent log rendering stalls and add copyable run results ([5519560](https://github.com/annrie/MoveDock/commit/5519560))
- 初期画面の見出しとサイドバー注記の文字を調整 / Balance onboarding headings and keep sidebar note on one line ([09452bf](https://github.com/annrie/MoveDock/commit/09452bf))

