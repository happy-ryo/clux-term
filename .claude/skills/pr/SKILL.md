---
name: pr
description: 現在のブランチから PR を作成する。事前に全 CI チェック (fmt→clippy→test→build→deny) を実行し、fmt/clippy は自動修正を試みる。全通過後のみ PR を作成。使い方: `/pr`
---

# /pr

現在のブランチから PR を作成する。

## 共通原則
- 事前検証 → エラーリカバリ → 状態ガイド → 冪等性
- 詳細は `../rust-dev-workflow/SKILL.md` 参照

## 手順

### 事前検証
1. main でないこと確認 (main なら `/start` を案内)
2. `git status --porcelain` → 未コミット変更なし確認
3. `git log main..HEAD --oneline` → 差分コミット存在確認
4. `gh pr list --head <branch>` → 既存 PR あれば push のみ提案

### CI チェック (自動修正付き)
順番に実行。失敗時:
- **fmt**: `cargo fmt` で自動修正 → 修正コミット → 再チェック
- **clippy**: `cargo clippy --fix --allow-dirty` を試行 → 不可なら停止
- **test**: 失敗テスト表示 → **PR 作成しない**
- **build**: エラー表示 → **PR 作成しない**

全通過後のみ続行。

### PR 作成
1. ブランチ名から Issue 番号抽出
2. `git push -u origin <branch>`
3. `gh pr create --title "<タイトル>" --body "<テンプレート>"`

テンプレート:
```
## Summary
- <箇条書き>

## Test plan
- [ ] <テスト項目>

Closes #<N>

🤖 Generated with [Claude Code](https://claude.ai/code)
```

4. 報告: PR URL、次ステップ (マージ後 `/done`)

### REPO の解決
1. CLAUDE.md の `repo:` 設定を確認
2. なければ `git remote get-url origin` から `owner/repo` を抽出

### エラーリカバリ
- push 認証失敗 → `gh auth status` 案内
- push リモート差分 → `git pull --rebase` 提案
- PR 作成失敗 → エラー内容 + 権限確認
