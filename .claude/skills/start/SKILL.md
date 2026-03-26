---
name: start
description: GitHub Issue の実装を開始する。事前検証（未コミット変更・ブランチ状態・main最新確認）→ featureブランチ作成 → タスク計画。引数なしでオープンIssue一覧表示。使い方: `/start <Issue番号>` または `/start`
---

# /start <Issue番号>

Issue の実装を開始する。

## 共通原則
- 事前検証 → エラーリカバリ → 状態ガイド → 冪等性
- 詳細は `../rust-dev-workflow/SKILL.md` 参照

## 手順

### 事前検証
1. `git status --porcelain` → 未コミット変更があれば停止
2. 現在 main 以外 → 「既にブランチにいます。続行or切り替え？」と確認
3. `git fetch origin` → main が最新か確認、遅れていれば pull

### 実行
1. `gh issue view <番号> --repo <REPO>` で Issue 取得 (存在/クローズ確認)
2. `git checkout main && git pull origin main`
3. Issue タイトルからブランチ名生成 → 同名あれば切り替え確認
4. `git checkout -b feature/issue-<N>-<slug>`
5. Issue のチェックリストから TaskCreate でタスク作成
6. 報告: Issue 要約、ブランチ名、タスク一覧、次ステップ

### 引数なし
`gh issue list --repo <REPO> --state open` → Milestone 別にグルーピング表示

### REPO の解決
1. CLAUDE.md の `repo:` 設定を確認
2. なければ `git remote get-url origin` から `owner/repo` を抽出

### エラーリカバリ
- gh 失敗 → `gh auth status` 確認案内
- ブランチ作成失敗 → dirty working / 同名ブランチ確認
- 再実行 → 既存ブランチがあれば再開提案 (冪等)
