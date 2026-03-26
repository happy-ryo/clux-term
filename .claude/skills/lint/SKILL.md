---
name: lint
description: clippy + fmt + deny チェックを実行。失敗時に修正提案を行う。使い方: `/lint`
---

# /lint

clippy + fmt + deny チェック。

## チェック順序
1. `cargo +nightly fmt --all -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo deny check` (deny.toml 存在時のみ)

## 失敗時
- fmt: `cargo +nightly fmt --all` で自動修正を提案
- clippy: `cargo clippy --fix --allow-dirty` を試行
- deny: エラー内容を表示し対処法を提示
