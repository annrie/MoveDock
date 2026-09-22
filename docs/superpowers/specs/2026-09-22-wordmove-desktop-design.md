# MoveDock 設計

状態: ユーザー承認済み。名称 MoveDock、独立リポジトリ work/MoveDock、macOS Universal（Intel / Apple Silicon）対応で実装。

## 目的

YTDown が yt-dlp を GUI から操作するのと同じ構成で、Wordmove を操作するデスクトップアプリを作る。Rust / Tauri 2 / Vue 3 / TypeScript / Vite / pnpm を使用する。初版は macOS を対象とし、既存の Ruby 3 対応 CLI を同期エンジンにする。

## 構成の選択

1. この Wordmove リポジトリの desktop/ に GUI を追加し、導入済み CLI を実行する。 Ruby 本体の修正と GUI の検証を同じリポジトリで進められる。既存 Movefile の意味や同期処理を維持できる。Ruby と外部コマンドの導入は必要。
2. **採用: GUI を別リポジトリとして作成する。**アプリの公開を独立させられるが、CLI との版管理や互換性検証が別途必要。
3. Ruby 実行環境と Wordmove をアプリへ同梱する。利用開始は簡単になるが、Ruby・ネイティブ gem・外部コマンドの配布を含むため、初版の範囲が大きくなる。

ユーザーの追加指示により 2 を採用し、work/MoveDock を独立 Git リポジトリとする。Rust に同期エンジンを移植せず、コマンド実行、状態管理、ファイル操作を担当させる。

## 画面と操作

- 日本語 UI。YTDown のサイドバー、メイン領域、下部ステータスバーの構成を参考にした macOS 向けの落ち着いた外観。ライト・ダーク表示に対応。
- サイドバーに登録サイト、実行履歴、アプリ設定を配置する。登録情報はサイト名と Movefile のパスを中心とする。
- サイト画面で Movefile を選択し、環境、ローカルとリモート、同期方向を表示する。
- Push / Pull と、WordPress 本体・uploads・themes・plugins・mu_plugins・languages・DB の対象を選択する。初期状態で DB を選択せず、対象未選択での実行は不可とする。
- シミュレーションと実行を別ボタンとする。コマンドの表示は実際に構築した引数から生成する。
- 実行直前に方向、環境、対象と上書き先を確認する。シミュレーション済みであることを、実行の安全性の保証として扱わない。
- 実行中は stdout / stderr を逐次表示し、二重実行を防止する。停止要求と停止完了を区別する。
- Movefile の新規テンプレート作成とテキスト編集を提供する。既存の ERB、YAML アンカー、コメントをフォームへの変換で失わない。保存前に外部変更を検出する。
- 設定画面で CLI 実行方法、実行ファイル、Gemfile、追加 PATH、Ruby バージョン指定を扱う。依存コマンドの検出結果と導入案内を表示する。

## Rust と CLI の接続

実行方法は次の 2 種類を扱う。

- インストール済みの `wordmove` を直接実行。
- 指定された Gemfile を `BUNDLE_GEMFILE` に設定し、`bundle exec wordmove` を実行。この互換性フォークを使用する場合の推奨方式。

サイトの作業ディレクトリ、実行ファイル、環境変数、引数配列を分離する。GUI で入力されたコマンド文字列をシェルに渡さない。環境名と対象は検証済みの選択肢から構築する。macOS の GUI 起動時の PATH 差を考慮し、Homebrew / rbenv とユーザー指定のパスを扱う。

環境一覧は既存 `wordmove list` を利用し、ANSI 制御文字を除去した出力を解釈する。列挙できない場合は読み込みエラーとログを表示する。Movefile の ERB は Ruby コードなので、初めて選択したファイルは利用者が内容を確認して読み込みを開始する。サイト一覧表示や起動だけでは未承認ファイルを評価しない。

`doctor` には現行 CLI で `--config` / `--environment` が定義されていないため、そのまま追加して呼び出さない。GUI の依存検出は独立して行い、初版では doctor ボタンを設けず、Wordmove の起動確認と依存検出を提供する。

同期は `push` / `pull`、`--environment`、`--config`、対象フラグを組み合わせ、シミュレーション時だけ `--simulate` を付ける。`--simulate` でも Movefile の ERB 評価と接続が発生し得ることを画面で説明する。

実行管理を一箇所に集約し、一度に一つのジョブを実行する。実行ごとに専用の Tauri Channel を作成してからジョブを起動し、短時間の実行でもログを取りこぼさない。終了結果には履歴 ID を付ける。成功、失敗、停止を区別し、終了コードを保存する。macOS では子プロセス群の停止を扱う。停止はロールバックではなく、同期途中の状態が残る場合がある。

## データとモジュール

- `src/components/`: サイト一覧、同期フォーム、ログ、設定、確認ダイアログ。
- `src/composables/`: Tauri API、実行イベントの購読、画面状態。
- `src/types/`: Rust との通信データの型。
- `src-tauri/src/`: サイト設定の永続化、CLI 検出、引数構築、実行管理、Movefile の読み書き。
- 設定と履歴は Tauri のアプリ用データディレクトリへ保存する。履歴には実行日時、サイト、環境、方向、対象、結果を保存し、ログ本文は初版では永続化しない。
- DB / SSH パスワードをサイト設定へ複製しない。実行ログの既知の機密情報を伏せ、詳細ログで認証情報が露出し得る `--debug` は初版 UI に公開しない。任意のフック出力まで完全に秘匿できるとは表示しない。
- Wordmove の Ruby リポジトリとは完全に分離する。

## エラー処理

Ruby / Bundler / Wordmove の未検出、起動失敗、不正な Movefile、存在しない環境、SSH 認証失敗、非ゼロ終了、停止を区別する。依存検出にはタイムアウトを設ける。起動や描画を同期 CLI の完了待ちでブロックしない。SSH の対話入力を無期限に待たせず、鍵や接続設定の確認を案内する。

## 検証と完了条件

1. Vue の型チェックと本番ビルド、Rust のフォーマットとテスト、Tauri アプリビルドが成功する。
2. 引数生成で空白を含むパス、不正な環境名、対象未選択を検証する。
3. 模擬 CLI で stdout / stderr、失敗終了、二重実行防止、停止と子プロセス回収を検証する。
4. テスト用 Movefile を用いて、ファイル選択、環境選択、実行確認、ログ表示、設定の再読み込みを確認する。
5. Ruby 側の変更が必要になった場合は既存 RSpec / RuboCop を実行する。
6. 実サイトへの Push / Pull は検証に使用しない。実機で検証できた範囲と未検証のリモート同期を README に明記する。
7. 開発起動、ビルド、CLI の導入と設定を日本語で記載する。

## 初版の範囲外

Ruby 自動導入・同梱、Windows 対応、スケジュール同期、自動更新、クラウド同期、ホスティング固有の自動設定、完全なフォーム型 Movefile エディターは初版に含めない。

## 調査した既存コード

- `../YTDown/package.json`、`../YTDown/src-tauri/Cargo.toml`、`../YTDown/src-tauri/tauri.conf.json`
- `../YTDown/src-tauri/src/ytdlp/binary.rs`、`../YTDown/src-tauri/src/ytdlp/process.rs`
- `lib/wordmove/cli.rb`、`lib/wordmove/movefile.rb`、`lib/wordmove/environments_list.rb`、`lib/wordmove/hook.rb`
- `lib/wordmove/generators/movefile.yml`、`README.md`、`wordmove.gemspec`
