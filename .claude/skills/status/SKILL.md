---
name: status
description: プロジェクト全体の進捗を表示。Milestone 進捗バー、Issue 一覧（Phase別）、PR の CI 状態を統合表示。使い方: `/status`
---

# /status

プロジェクト全体の進捗を表示。

## 共通原則
- 詳細は `../rust-dev-workflow/SKILL.md` 参照

## 収集情報
1. Git 状態 (ブランチ、未コミット変更、リモート差分)
2. `gh api repos/<REPO>/milestones` → Milestone 進捗バー
3. `gh issue list --state open` → Phase 別 Issue
4. `gh pr list` → オープン PR + CI 状態

## 表示
プログレスバー付き Milestone 進捗 + Issue 一覧 + 推奨アクション

## REPO の解決
1. CLAUDE.md の `repo:` 設定を確認
2. なければ `git remote get-url origin` から `owner/repo` を抽出
