<p align="center"><img src="src-tauri/icons/icon.png" width="100" alt="MoveDock icon" /></p>
<h1 align="center">MoveDock</h1>
<p align="center">WordPress の同期を、ひとつのワークスペースで。<br />Sync WordPress from one workspace.<br />Rust · Tauri 2 · Vue 3 · macOS Universal</p>

<div align="center">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/annrie/MoveDock.svg" alt="License" /></a>
  <a href="https://github.com/annrie/MoveDock/releases/latest"><img src="https://img.shields.io/github/v/release/annrie/MoveDock.svg" alt="Latest release" /></a>
  <a href="https://github.com/annrie/MoveDock/releases"><img src="https://img.shields.io/github/downloads/annrie/MoveDock/total.svg" alt="Total downloads" /></a>
  <a href="https://github.com/annrie/MoveDock/releases/latest"><img src="https://img.shields.io/github/downloads/annrie/MoveDock/latest/total.svg" alt="Latest release downloads" /></a>
  <a href="https://github.com/annrie/MoveDock/stargazers"><img src="https://img.shields.io/github/stars/annrie/MoveDock.svg" alt="Stars" /></a>
</div>
<p align="center">
  <img src="https://img.shields.io/badge/macOS-Universal-222222?logo=apple&amp;logoColor=white" alt="macOS Universal: Intel and Apple Silicon" />
  <img src="https://img.shields.io/badge/Rust-Tauri%202-24C8D8?logo=tauri&amp;logoColor=white" alt="Rust and Tauri 2" />
  <img src="https://img.shields.io/badge/Vue-3-4FC08D?logo=vuedotjs&amp;logoColor=white" alt="Vue 3" />
</p>
<p align="center"><a href="#日本語">日本語</a> · <a href="#english">English</a> · <a href="https://github.com/annrie/MoveDock/releases/latest">Download / ダウンロード</a></p>

![MoveDock workspace — example with a fictional site / 架空のサイトを使った表示例](docs/screenshots/workspace.png)

## 日本語

Wordmove CLI を操作する、8言語対応の macOS デスクトップアプリです。Intel と Apple Silicon 両方のコードを含むユニバーサルアプリをビルドします。Nuxt や常駐バックエンドサーバーは使用しません。

Wordmove 本体とは独立したリポジトリです。同期処理には導入済みの CLI を使用し、[annrie/wordmove の Ruby 3 対応フォーク](https://github.com/annrie/wordmove)を Bundler 経由で利用できます。上流 Wordmove の公式アプリではありません。

**日本語・英語・繁体字中国語・フランス語・スペイン語・ポルトガル語（ブラジル）・ドイツ語・韓国語に対応しています。** 初回はシステムの言語を検出し、未対応の言語は英語で表示します。「設定 → 表示言語」で変更すると即時反映・保存されます。Wordmove / SSH / rsync などの実行ログは原文のまま表示し、MoveDock の通知・確認ダイアログ・エラー案内は選択した言語で表示します。言語を切り替えても、編集中の接続設定や読み込み済みの環境は保持されます。

### ダウンロード・インストール

1. [最新の Release](https://github.com/annrie/MoveDock/releases/latest) から `MoveDock_0.2.0_universal.dmg` をダウンロードします。
2. DMG を開き、`MoveDock.app` をアプリケーションフォルダへコピーします。
3. Ruby / Wordmove と必要な外部コマンドを導入し、下の「最初の設定」を行います。

Intel / Apple Silicon 共通のアプリです。Developer ID 署名・公証は未設定で、ad-hoc 署名を使用しています。macOS で起動時のセキュリティ確認が必要になる場合があります。

### 機能

- Movefile を選択して複数サイトを登録、環境を選択
- Push / Pull、対象の個別選択、シミュレーション
- 実行直前の同期先・対象確認、リアルタイム stdout / stderr 表示、ログの選択・コピー
- 実行の停止（子プロセス群を含む）、二重実行の防止
- Movefile の新規作成・編集、外部変更の検出
- 実行履歴（最新 200 件、ログ本文は保存しません）
- Ruby / Bundler / Wordmove / SSH / rsync などの検出
- 実行ファイル、Gemfile、Ruby バージョン、追加 PATH の設定
- ライト・ダーク・システムテーマ

### 必要な環境

- macOS 11 以降（Intel / Apple Silicon）
- Wordmove と、それに対応する Ruby / Bundler
- 同期方式に応じて SSH、rsync、WP-CLI、PHP、MySQL クライアント。FTP は lftp

アプリの Universal 対応と、外部コマンドの導入は別です。Ruby / Wordmove / 外部コマンドは同梱しません。それぞれの Mac に適したものを導入してください。Ruby 環境の準備は [Wordmove README](https://github.com/annrie/wordmove#日本語) も参照してください。

推奨する CLI は [annrie/wordmove v5.3.0.pre.2](https://github.com/annrie/wordmove/releases/tag/v5.3.0.pre.2) です。MoveDock 0.1.0 で検証した Ed25519 鍵と rsync の修正を含みます。新規導入時は Ruby 3.3 / 3.4 と Bundler を用意し、次のように修正版のタグを指定できます（`.ruby-version` は 3.3.12 を指定）。

```sh
mkdir -p ~/work
git clone --branch v5.3.0.pre.2 https://github.com/annrie/wordmove.git ~/work/wordmove
cd ~/work/wordmove
bundle config set --local path vendor/bundle
bundle install
bundle exec wordmove --version
```

### 最初の設定

1. 初回起動時に Wordmove・Gemfile・rbenv の Ruby バージョン・必要な追加 PATH を自動入力して保存します。以前のバージョンで初期状態のままだった設定も対象です。保存済みの手入力は保持します。
2. **設定 → 実行環境を確認**で Wordmove が起動するか確認します。未導入の gem などはここで表示されます。自動入力だけでインストールや接続確認は行いません。
3. 見つからない項目だけ指定してください。導入後の「空欄を再検出」は、入力済みの値を保持して空欄を補完・保存します。未保存の変更がある間は再検出と確認を無効にします。
4. 「Movefile を開く」でサイトを登録します。内容を確認し、「環境を読み込む」で信頼した Movefile を評価します。
5. 環境・方向・対象を選び、まずシミュレーションを実行します。結果を確認してから同期を実行してください。

自動検出はファイルの存在とパスだけを調べ、Gemfile・Movefile・シェルの起動スクリプトを評価しません。Wordmove フォークはホーム配下の `work` / `Projects` / `projects` / `Developer` / `Code` / `src` にある `wordmove` を探索します。複数ある場合は選択を求めます。一つ見つかれば Bundler、見つからず Wordmove コマンドがある場合は直接実行を設定します。

PHP / MySQL が通常の PATH にない場合、Homebrew と Local の導入済みサービスを探索します。Local は導入済みの新しいバージョンを候補にします。サイト固有のバージョンが必要なら追加 PATH を変更してください。追加不要の場合は空欄のままで正常です。

既存 gem を使う場合は「Wordmove を直接実行」を選びます。GUI 起動時にも Homebrew / rbenv の一般的なパスを探索し、追加 PATH を優先します。`~/` は展開されます。実行ファイル欄にシェルコマンドや追加引数は入力しません。

Bundler モードは指定した Gemfile とその横の `.bundle/config` を利用します。rbenv の実行基準ディレクトリも合わせるため、フォークの `.ruby-gemset` を参照できます。Ruby や gem を切り替えた場合は、同じ環境で `bundle install` が成功することを確認してください。

### 実行時の扱い

- Movefile の ERB は Ruby コードです。サイト登録・アプリ起動だけでは評価せず、明示的な読み込み時に実行します。信頼できるファイルを使用してください。
- シミュレーションは Wordmove の `--simulate` を利用します。ERB の評価・接続が発生し得るため、完全に副作用がない操作ではありません。
- 停止はロールバックではありません。停止前に行われたファイル転送や DB 更新が残ることがあります。
- CLI の終了コードが 0 の場合を「正常終了」と表示します。Movefile の `forbid` でスキップした対象もあるため、ログを確認してください。
- SSH の対話入力は使用できません。SSH agent / 鍵認証、known_hosts、必要な接続設定を事前に確認してください。同期は最大 4 時間でタイムアウトします。
- Movefile を外部で変更した場合は再読み込みが必要です。設定保存や Movefile 編集後も環境を再読み込みします。
- 環境一覧には Wordmove の `list` 出力を使用します。リモート環境には `vhost` が必要です。ERB / .env の解決規則は利用する CLI に従います。
- ログは画面上で最新 2000 行を保持します。既知のパスワード表現を伏せますが、任意のフック出力に含まれる機密情報の完全な秘匿は保証しません。
- 新規 Movefile テンプレートは例示の接続先です。編集してから使用してください。初期状態でリモート DB への push を `forbid` で禁止しています。

設定・サイト・実行履歴は `~/Library/Application Support/com.annrie.movedock/settings.json` に保存します。表示言語はアプリの WebView のローカルストレージに別途保存します。Movefile の接続パスワードは設定ファイルへ複製しません。

### 既知の問題

初回の「環境を読み込む」が一度タイムアウトし、再読み込みで成功する事象を確認しています。原因は調査中で、v0.2.0 でも未解決です。今回の再起動・負荷テストでは再現せず、環境取得は約 4〜16 秒で成功しました。失敗した場合は読み込みを再試行してください。詳細は [検証記録](docs/verification.md) にあります。

### 開発

Node.js 22.12 以降、pnpm、Rust と Xcode Command Line Tools が必要です。

```sh
pnpm install
pnpm app:dev
```

ブラウザ上の見た目だけを確認する場合は `pnpm dev`（ポート 1425）。ブラウザではネイティブ操作は無効です。

```sh
pnpm build                # Vue 型チェック + Vite 本番ビルド
pnpm test                 # 同期フローのテスト
pnpm rust:test            # CLI 引数・実行・停止・変更検出のテスト
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

### 変更履歴・リリース

Conventional Commits（`feat:` / `fix:` / `docs:` など）から changelogen で変更履歴を生成します。

```sh
pnpm changelog       # 変更履歴をプレビュー
pnpm release:patch   # 0.1.0 → 0.1.1
pnpm release:minor   # 0.1.0 → 0.2.0
pnpm release:major   # 0.1.0 → 1.0.0
```

リリース前に作業内容をコミットしてください。リリースコマンドは `CHANGELOG.md` を生成し、`package.json`・`Cargo.toml`・`Cargo.lock`・Tauri 設定のバージョンを揃え、ローカルのリリースコミットとタグを作成します。画面のバージョン表示も同じリリース処理で同期します。push・GitHub Release の公開は手動です。README と GitHub Release の説明は日本語・英語の両方を用意します。changelogen の生成結果を確認し、公開するリリース説明を日英併記に整えてください。

### Universal ビルド

[Tauri の Universal ビルド](https://v2.tauri.app/distribute/app-store/)を利用します。

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm app:build
```

出力:

- `src-tauri/target/universal-apple-darwin/release/bundle/macos/MoveDock.app`
- `src-tauri/target/universal-apple-darwin/release/bundle/dmg/MoveDock_0.2.0_universal.dmg`

```sh
lipo -archs src-tauri/target/universal-apple-darwin/release/bundle/macos/MoveDock.app/Contents/MacOS/movedock
# x86_64 arm64
```

既定はローカル利用向けの ad-hoc 署名です。公開配布用の Developer ID 署名・公証は未設定です。

### 検証範囲

**Local.app の2サイトで実環境の同期を確認済みです。** DB同期とファイルの Pull を確認しています。

CLI の引数、標準出力・エラー出力、失敗終了、停止、タイムアウト、Movefile の変更検出とフロントエンドの確認フローを模擬 CLI / テストデータで検証します。実サイトへの Push / Pull は自動検証しません。ステージング環境でバックアップを取って確認してください。

### ライセンス

MIT。Wordmove 本体の著作権・ライセンスはそれぞれのリポジトリに従います。アプリの構成は、yt-dlp のデスクトップクライアント [YTDown](https://github.com/annrie/YTDown) を参考にしています。

## English

MoveDock is a macOS desktop interface for the Wordmove CLI, built with Rust, Tauri 2, Vue 3 and TypeScript. Its Universal app contains both Intel and Apple Silicon binaries. It does not use Nuxt or a persistent backend server.

The app is maintained separately from Wordmove and runs an installed CLI. It supports the [annrie/wordmove Ruby 3 compatibility fork](https://github.com/annrie/wordmove) through Bundler. It is not an official upstream Wordmove application.

**Available in Japanese, English, Traditional Chinese, French, Spanish, Brazilian Portuguese, German and Korean.** The first launch detects the system language, falling back to English for unsupported languages. Changes under **Settings → Display language** apply and save immediately. Wordmove / SSH / rsync output remains in its original language; MoveDock notifications, confirmations and error guidance use the selected language. Switching languages preserves unsaved connection settings and loaded environments.

### Download and install

1. Download `MoveDock_0.2.0_universal.dmg` from the [latest release](https://github.com/annrie/MoveDock/releases/latest).
2. Open the DMG and copy `MoveDock.app` to your Applications folder.
3. Install Ruby, Wordmove and the required external commands, then follow “First-time setup” below.

The same app supports Intel and Apple Silicon Macs. It uses an ad-hoc signature; Developer ID signing and notarization are not configured. macOS may require a security confirmation when opening it.

### Features

- Register multiple sites from Movefiles and select remote environments
- Push / Pull, individual sync targets and simulation
- Confirm the destination and targets before running; view, select and copy stdout / stderr logs
- Stop a running operation, including its child processes, and prevent concurrent runs
- Create and edit Movefiles, with detection of external changes
- Keep the latest 200 history entries without persisting log contents
- Detect Ruby, Bundler, Wordmove, SSH, rsync and related tools
- Configure the executable, Gemfile, Ruby version and additional PATH entries
- Light, dark and system themes

### Requirements

- macOS 11 or later, on Intel or Apple Silicon
- Wordmove and a compatible Ruby / Bundler environment
- SSH, rsync, WP-CLI, PHP and MySQL client tools as required by your sync method; lftp for FTP

Ruby, Wordmove and external tools are not bundled with the Universal app. Install versions suitable for your Mac. See the [Wordmove README](https://github.com/annrie/wordmove#english) for Ruby setup information.

The recommended CLI is [annrie/wordmove v5.3.0.pre.2](https://github.com/annrie/wordmove/releases/tag/v5.3.0.pre.2), which includes the Ed25519 key and rsync fixes verified with MoveDock 0.1.0. For a new installation, prepare Ruby 3.3 / 3.4 and Bundler, then select the maintenance release tag (`.ruby-version` specifies 3.3.12):

```sh
mkdir -p ~/work
git clone --branch v5.3.0.pre.2 https://github.com/annrie/wordmove.git ~/work/wordmove
cd ~/work/wordmove
bundle config set --local path vendor/bundle
bundle install
bundle exec wordmove --version
```

### First-time setup

1. On first launch, MoveDock detects and saves the Wordmove executable, Gemfile, rbenv Ruby version and additional PATH entries where possible. It also fills settings left at their original defaults by earlier versions, preserving saved manual values.
2. Open **設定 → 実行環境を確認** (Settings → Check runtime) to verify that Wordmove starts. Missing gems are reported here. Detection alone does not install dependencies or test connections.
3. Fill in only the missing values. **空欄を再検出** (Detect empty fields again) fills and saves empty fields while preserving existing values. Detection and checks are disabled while there are unsaved edits.
4. Use **Movefile を開く** (Open Movefile) to register a site. Review the contents and explicitly load the environments from a trusted Movefile.
5. Select the environment, direction and targets, run a simulation, and review its result before syncing.

Automatic detection inspects file existence and paths without evaluating Gemfiles, Movefiles or shell startup scripts. It searches for a `wordmove` directory under `work`, `Projects`, `projects`, `Developer`, `Code` and `src` in your home directory. Multiple matches require manual selection. A single match selects Bundler mode; otherwise, an available Wordmove executable selects direct mode.

If PHP or MySQL is not on the usual PATH, detection checks Homebrew and installed Local services, preferring newer installed Local versions. Adjust the additional PATH entries when a site needs a specific version. An empty additional PATH is valid when no additions are needed.

To use an existing gem, select **Wordmove を直接実行** (Run Wordmove directly). Common Homebrew and rbenv paths are searched even when launched as a GUI, and additional PATH entries take priority. `~/` is expanded. Enter an executable path, not a shell command or additional arguments.

Bundler mode uses the selected Gemfile and its adjacent `.bundle/config`. It also sets rbenv's working directory to the fork so that `.ruby-gemset` is respected. After changing Ruby or gems, ensure that `bundle install` succeeds in the same environment.

### Runtime behavior

- Movefile ERB is Ruby code. Registering a site or starting the app does not evaluate it; explicit environment loading does. Use trusted files.
- Simulation uses Wordmove's `--simulate`. It may still evaluate ERB and establish connections, so it is not guaranteed to be free of side effects.
- Stopping is not a rollback. File transfers and database changes already performed may remain.
- A CLI exit code of 0 is shown as a successful exit. Check the logs, as `forbid` rules may have skipped some targets.
- Interactive SSH input is unavailable. Prepare SSH agent / key authentication, known_hosts and connection settings in advance. Sync commands have a four-hour timeout.
- Reload environments after external Movefile changes, settings changes or Movefile edits.
- Environment discovery uses Wordmove's `list` output. Remote environments need a `vhost`. ERB and `.env` resolution follow the selected CLI's behavior.
- The UI retains the latest 2,000 log lines. Known password patterns are redacted, but arbitrary hook output may still contain sensitive information.
- The new Movefile template contains example connection details. Edit it before use; remote database pushes are forbidden by default.

Settings, registered sites and history are stored in `~/Library/Application Support/com.annrie.movedock/settings.json`. Connection passwords from Movefiles are not copied into this settings file. The display language is stored separately in the app WebView’s local storage.

### Known issue

The first environment load has occasionally timed out, with a retry succeeding. The cause is under investigation and remains unresolved in v0.2.0. Recent restart and load tests did not reproduce the timeout; environment discovery completed in approximately 4–16 seconds. Retry loading if it fails. See the [verification record](docs/verification.md) for details (Japanese).

### Development

Requires Node.js 22.12 or later, pnpm, Rust and Xcode Command Line Tools.

```sh
pnpm install
pnpm app:dev
```

Use `pnpm dev` to preview the UI in a browser on port 1425. Native operations are disabled in browser previews.

```sh
pnpm build                # Vue type checks and Vite production build
pnpm test                 # Frontend sync-flow tests
pnpm rust:test            # CLI arguments, execution, cancellation and change detection
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

### Changelog and releases

Changelogen generates the changelog from Conventional Commits such as `feat:`, `fix:` and `docs:`.

```sh
pnpm changelog       # Preview the changelog
pnpm release:patch   # 0.1.0 → 0.1.1
pnpm release:minor   # 0.1.0 → 0.2.0
pnpm release:major   # 0.1.0 → 1.0.0
```

Commit your work before running a release command. The command generates `CHANGELOG.md`, synchronizes versions in `package.json`, `Cargo.toml`, `Cargo.lock`, the Tauri configuration and the app's version label, then creates a local release commit and tag. Pushing and publishing a GitHub Release are separate manual steps. Maintain both Japanese and English README and release descriptions; review generated entries and prepare bilingual release notes before publishing.

### Universal build

Uses [Tauri's Universal build](https://v2.tauri.app/distribute/app-store/).

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm app:build
```

Outputs:

- `src-tauri/target/universal-apple-darwin/release/bundle/macos/MoveDock.app`
- `src-tauri/target/universal-apple-darwin/release/bundle/dmg/MoveDock_0.2.0_universal.dmg`

```sh
lipo -archs src-tauri/target/universal-apple-darwin/release/bundle/macos/MoveDock.app/Contents/MacOS/movedock
# x86_64 arm64
```

The default build uses an ad-hoc signature for local use. Developer ID signing and notarization for distribution are not configured.

### Verification scope

**Real-world synchronization has been verified on two Local.app sites using MoveDock.** Verification includes database synchronization and file pulls.

Mock CLI and test data cover arguments, stdout / stderr, failures, cancellation, timeouts, Movefile change detection and frontend confirmation flows. Automated tests do not perform real-site Push / Pull operations. Back up your data and verify your setup in staging before syncing.

### License

MIT. Wordmove and its dependencies retain their respective copyrights and licenses. The app structure draws on [YTDown](https://github.com/annrie/YTDown), a desktop client for yt-dlp.
