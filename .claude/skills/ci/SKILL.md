---
name: ci
description: ローカル CI パイプラインを一括実行。fmt → clippy → test → build → deny の順でチェックし、結果サマリーをテーブル表示。全通過で `/pr` を案内。使い方: `/ci`
---

# /ci

ローカル CI パイプラインを実行。

## 共通原則
- 事前検証 → エラーリカバリ → 状態ガイド → 冪等性
- 詳細は `../rust-dev-workflow/SKILL.md` 参照

## チェック順序
1. `cargo +nightly fmt --all -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --all`
4. `cargo build`
5. `cargo deny check` (deny.toml 存在時のみ)

## 結果表示
テーブル形式で全結果サマリー。全通過 → `/pr` 案内。失敗 → 修正案内。

## 自動修正モード
「修正して」で fmt/clippy の自動修正→コミット→再チェック。

## CI ステップの解決
1. CLAUDE.md の `ci_steps:` 設定を確認
2. なければデフォルト: fmt → clippy → test → build → deny (deny.toml 存在時)
