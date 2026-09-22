# 実装計画

設計は承認済み。Nuxt は使わず Vue + Vite とする。writing-plans スキルは環境に存在しなかったため、この計画で実装を管理する。

1. 独立リポジトリ work/MoveDock に Vue / TypeScript / Tauri 2 の構成と Universal ビルドスクリプトを追加する。
2. Rust に設定保存、Movefile 管理、CLI の引数構築、環境列挙、出力ストリーム、停止を実装する。
3. 日本語のサイト管理・同期・編集・設定・履歴画面を実装する。
4. 模擬 CLI で実行・失敗・停止をテストし、フロントエンドの型チェックと主要な操作を検証する。
5. MoveDock の macOS Universal .app / .dmg をビルドし、lipo で両アーキテクチャを確認する。
6. 開発・利用手順、依存ソフトと検証範囲を記録する。
