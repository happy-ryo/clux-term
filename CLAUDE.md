# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

WezTermベースのAIエージェント協調型ターミナルマルチプレクサ (Rust製)

- リポジトリ: https://github.com/happy-ryo/clux-term
- 上流: https://github.com/wezterm/wezterm

clux adds Claude Code coordination features via an embedded MCP server on top of WezTerm's terminal emulation, font rendering, and multiplexer functionality.

## Build & Test Commands

```bash
# ビルド
cargo build

# テスト (全体)
cargo test --workspace

# テスト (単一クレート)
cargo test -p wezterm-term
cargo test -p termwiz

# テスト (単一テスト関数)
cargo test -p wezterm-term -- test_function_name

# フォーマット (nightly toolchain 必須)
cargo +nightly fmt --all -- --check    # チェックのみ
cargo +nightly fmt --all               # 自動修正

# ライセンス/セキュリティ監査
cargo deny check

# 全CIチェック一括 (スラッシュコマンド)
/ci
```

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
| `/ci` | 全CIチェック実行 (fmt→test→build→deny) |
| `/lint` | fmt + deny チェック |
| `/build` | cargo build |
| `/test` | cargo test |

### コマンドの設計原則

各コマンドは以下の原則に従う:
- **事前検証**: 実行前に前提条件を全てチェックし、問題があれば是正策を提示して停止
- **エラーリカバリ**: 失敗時に自動修正を試み、不可能な場合は具体的な対処法を提示
- **状態ガイド**: 完了後に次にやるべきことを明示 (例: `/ci` 通過後 → `/pr` を案内)
- **冪等性**: 同じコマンドを再実行しても安全 (既存ブランチ/PRがあれば検出して対応)

### Git Hooks

- `scripts/pre-commit`: コミット前にfmt + build を自動実行
- hooks設定: `git config core.hooksPath scripts`

### Branch Naming

- `feature/issue-<番号>-<短い説明>` (例: `feature/issue-6-dev-workflow`)

### Commit Message

- 英語で簡潔に (why > what)
- `Closes #<番号>` でIssue自動クローズ
- `Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>`

## Architecture

### レイヤー構成

```
┌─────────────────────────────────────────────────┐
│  Entry Points                                    │
│  wezterm/ (CLI)  wezterm-gui/  wezterm-mux-server/ │
├─────────────────────────────────────────────────┤
│  Rendering: window/ (X11/Wayland/macOS/WGPU)    │
│  wezterm-font/ (FreeType/HarfBuzz glyph shaping)│
├─────────────────────────────────────────────────┤
│  Configuration: config/ (Lua 5.4 via mlua)       │
│  lua-api-crates/ (Rust→Lua API バインディング群)    │
├─────────────────────────────────────────────────┤
│  Multiplexer: mux/ (Pane/Tab/Window/Domain管理)  │
│  portable-pty (PTYプロセス管理)                    │
├─────────────────────────────────────────────────┤
│  Terminal Core: term/ (VTEパーサー, セルグリッド)   │
│  termwiz/ (エスケープシーケンス, ターミナルプリミティブ) │
└─────────────────────────────────────────────────┘
```

### レンダリングパイプライン

PTY → `mux::Pane` (localpane.rs) → `TermWindow::paint()` → HarfBuzz text shaping → GlyphCache (テクスチャアトラス) → WGPU/GL シェーダー → 画面

### 主要クレート間の依存

- `wezterm-gui` → `window`, `mux`, `config`, `wezterm-font`
- `mux` → `term`, `config`, `portable-pty`
- `config` → `mlua` (Lua 5.4), `wezterm-dynamic` (Lua⇔Rust型変換)
- `term` → `termwiz`, `wezterm-cell`, `wezterm-escape-parser`

### Lua設定システム

- 起動時に `wezterm.lua` をロード (`config::configuration()`)
- `lua-api-crates/` 配下の各クレートがRust関数をLuaに公開 (mux-lua, window-funcs, spawn-funcs 等)
- `wezterm-dynamic` クレートによるLua→Rust型マーシャリング

## CI

- rustfmt -- `cargo +nightly fmt --all -- --check` (`.github/workflows/fmt.yml`)
- cargo-deny -- ライセンス/セキュリティ監査 (`deny.toml`)
- プラットフォーム別ビルド+テスト (`.github/workflows/gen_*.yml`)
