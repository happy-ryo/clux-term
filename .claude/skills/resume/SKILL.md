---
name: resume
description: 中断した作業を再開する。Git 状態から作業フェーズを推定し、タスクリストを復元して次のステップを案内。新しい会話セッションの最初に実行する。使い方: `/resume`
---

# /resume

中断した作業を再開する。

## 共通原則
- 事前検証 → エラーリカバリ → 状態ガイド → 冪等性
- 詳細は `../rust-dev-workflow/SKILL.md` 参照

## 手順

### 状態推定
1. `git branch --show-current` でブランチ判定
2. **main**: 直近マージ PR 確認 → `/start` 案内
3. **feature ブランチ**:
   - ブランチ名から Issue 番号抽出
   - Issue 内容取得
   - PR 存在/状態確認
   - main との diff 確認
4. 状態→アクション:
   - PR マージ済み → `/done` 案内
   - PR オープン → CI 確認、レビュー待ち or 修正要
   - コミットあり PR なし → 続行 or `/pr`
   - コミットなし → Issue タスク表示、実装開始

### タスク復元
Issue チェックリスト → TaskCreate → git log から完了推定

### REPO の解決
1. CLAUDE.md の `repo:` 設定を確認
2. なければ `git remote get-url origin` から `owner/repo` を抽出
