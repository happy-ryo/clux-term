# clux-coord (Rust版)

Claude Code ワーカーエージェントを協調させるための軽量 MCP サーバー。

## セットアップ

### 1. ビルド

```bash
git checkout feature/coord-server-rust
cargo build -p clux-coord
```

### 2. サーバー起動

```bash
# ポート指定（省略時はOS自動割り当て）
./target/debug/clux-coord-server.exe 19850
```

起動すると `MCP server listening on http://127.0.0.1:19850/mcp` と表示される。

### 3. Claude Code に接続

方法A: 起動時に `--mcp-config` で指定

```bash
# 設定ファイルを作成
cat > clux-mcp.json << 'EOF'
{
  "mcpServers": {
    "clux-coord": {
      "type": "http",
      "url": "http://127.0.0.1:19850/mcp"
    }
  }
}
EOF

# Claude Code を起動
claude --mcp-config clux-mcp.json
```

方法B: プロジェクトの `.claude/settings.local.json` に追加

```json
{
  "mcpServers": {
    "clux-coord": {
      "type": "http",
      "url": "http://127.0.0.1:19850/mcp"
    }
  }
}
```

### 4. 動作確認

Claude Code 内で以下を試す:

```
clux_list_workers ツールを呼んで
```

空の配列 `[]` が返ってくれば接続成功。

## ワーカー起動の例

セントラル Claude Code から別ペインでワーカーを起動する:

```bash
# 別ターミナル or ペインで実行
claude --print \
  --mcp-config clux-mcp.json \
  --dangerously-skip-permissions \
  "clux_register_worker で登録して (worker_id: worker-1, task_description: cargo test 実行, cwd: /path/to/project)。
   cargo test を実行して、結果を clux_report_result で報告して (worker_id: worker-1)。"
```

セントラル側で結果を確認:

```
clux_list_workers ツールで結果を確認して
```

## 提供ツール

| ツール名 | 用途 | 主な引数 |
|---------|------|---------|
| `clux_register_worker` | ワーカー自己登録 | `worker_id`, `task_description`, `cwd` |
| `clux_report_result` | 結果報告 | `worker_id`, `status` (completed/failed), `summary` |
| `clux_list_workers` | 全ワーカー状態確認 | なし |
| `clux_check_permissions` | 権限プロンプト待ち確認 | なし |

## テスト

```bash
cargo test -p clux-coord
```
