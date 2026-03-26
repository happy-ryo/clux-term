# clux - Development Guide

## Project

WezTermベースのAIエージェント協調型ターミナルマルチプレクサ (Rust製)

- リポジトリ: https://github.com/happy-ryo/clux-term
- 上流: https://github.com/wezterm/wezterm

## Development Workflow

**mainブランチへの直接push禁止。必ず以下のフローで進める。**

```
/start <Issue番号>  →  実装  →  /ci  →  /pr  →  レビュー/マージ  →  /done
```

新しい会話セッションで作業を再開する場合は `/resume` を実行する。

### Slash Commands

| コマンド | 説明 |
|---------|------|
| `/start <番号>` | Issueの実装を開始 (事前検証→ブランチ作成→タスク計画) |
| `/pr` | PR作成 (事前検証→CIチェック→自動修正→PR作成) |
| `/done` | 後片付け (マージ確認→ブランチ削除→次Issue提案) |
| `/resume` | 中断した作業の再開 (状態推定→タスク復元→次ステップ案内) |
| `/status` | プロジェクト進捗表示 (Milestone進捗→Issue→PR状態) |
| `/ci` | 全CIチェック実行 (fmt→clippy→test→build→deny) |
| `/lint` | clippy + fmt + deny チェック |
| `/build` | cargo build |
| `/test` | cargo test |

### コマンドの設計原則

各コマンドは以下の原則に従う:
- **事前検証**: 実行前に前提条件を全てチェックし、問題があれば是正策を提示して停止
- **エラーリカバリ**: 失敗時に自動修正を試み、不可能な場合は具体的な対処法を提示
- **状態ガイド**: 完了後に次にやるべきことを明示 (例: `/ci` 通過後 → `/pr` を案内)
- **冪等性**: 同じコマンドを再実行しても安全 (既存ブランチ/PRがあれば検出して対応)

### Git Hooks

- `scripts/pre-commit`: コミット前にfmt + clippy + build を自動実行
- hooks設定: `git config core.hooksPath scripts`

### Branch Naming

- `feature/issue-<番号>-<短い説明>` (例: `feature/issue-6-dev-workflow`)

### Commit Message

- 英語で簡潔に (why > what)
- `Closes #<番号>` でIssue自動クローズ
- `Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>`

## Tech Stack

- Rust (edition 2021)
- WezTerm (GPU-accelerated terminal emulator)
- termwiz (terminal primitives, escape sequence handling)
- FreeType / HarfBuzz / fontconfig (font rendering & shaping)
- OpenGL (GPU rendering)

## CI

- rustfmt -- `cargo +nightly fmt --all -- --check` (`.github/workflows/fmt.yml`)
- clippy -- `cargo clippy --all-targets -- -D warnings`
- cargo-deny -- ライセンス/セキュリティ監査 (`deny.toml`)
- プラットフォーム別ビルド+テスト (`.github/workflows/gen_*.yml`)

## Workspace Structure

```
wezterm/                # メインバイナリ (CLI)
wezterm-gui/            # GUIフロントエンド
wezterm-mux-server/     # マルチプレクササーバー
wezterm-ssh/            # SSH実装
wezterm-surface/        # サーフェス抽象化
wezterm-cell/           # セル・テキスト表現
wezterm-escape-parser/  # エスケープシーケンスパーサー
wezterm-dynamic/        # 動的型付け
wezterm-blob-leases/    # BLOBリース管理
wezterm-open-url/       # URL処理
wezterm-uds/            # Unixドメインソケット
strip-ansi-escapes/     # ANSIエスケープ除去
sync-color-schemes/     # カラースキーム同期
bidi/                   # 双方向テキスト処理
deps/cairo/             # Cairo描画バインディング
```
