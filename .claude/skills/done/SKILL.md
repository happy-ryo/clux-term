---
name: done
description: 作業完了の後片付け。PR のマージ状態を確認し、ブランチ削除と次 Issue の提案を行う。使い方: `/done`
---

# /done

作業完了の後片付け。

## 共通原則
- 事前検証 → エラーリカバリ → 状態ガイド → 冪等性
- 詳細は `../rust-dev-workflow/SKILL.md` 参照

## 手順

### 事前検証
1. featureブランチか確認 (main なら直近マージ PR を確認)
2. 未コミット変更なし確認

### 分岐
- **PR マージ済み**: main checkout → ブランチ削除 (`-d` のみ、`-D` 禁止) → リモート削除 → タスク完了 → Issue クローズ確認
- **PR オープン**: CI ステータス報告、レビュー待ちか修正要かを案内
- **PR 未作成**: `/pr` を案内

### 次ステップ提案
同 Phase の残り Issue を表示 → `/start <番号>` を提案

### REPO の解決
1. CLAUDE.md の `repo:` 設定を確認
2. なければ `git remote get-url origin` から `owner/repo` を抽出
