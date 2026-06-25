# CLAUDE.md — memlong Claude Code 專案規則

## 專案背景

本專案是 local-first long-term memory system for coding agents。核心為 Rust MCP stdio server，目標支援 OpenCode、Codex、Claude Code。

## Claude Code 使用規則

1. 每次開始任務時，若 MCP server 可用，先使用 `memlong-memory` 搜尋相關專案記憶。
2. 完成任務時，僅保存 durable lessons，不保存短期訊息。
3. 不保存 secrets、API keys、tokens、passwords、private keys。
4. 修改 storage / install / MCP schema 前先看 `spec.md` 與 `test.md`。
5. 每批修改後執行相關測試。
6. 回覆使用繁體中文。

## MCP server

Claude Code 可透過 `.mcp.json` 或 CLI 加入本地 stdio server：

```bash
claude mcp add --transport stdio --scope user memlong-memory -- /absolute/path/to/memlong-memory
```

或專案根目錄 `.mcp.json`：

```json
{
  "mcpServers": {
    "memlong-memory": {
      "command": "/absolute/path/to/memlong-memory",
      "args": [],
      "env": {
        "LLM_API_BASE": "mock",
        "LLM_API_KEY": "mock"
      },
      "timeout": 120000
    }
  }
}
```

## 測試命令

```bash
export LLM_API_BASE=mock
export LLM_API_KEY=mock
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release
./target/release/memory-mcp-server health
cd plugin && npm ci && npm test
```

## 修改注意

- MCP stdio mode 下 stdout 必須保持 JSON-RPC clean。
- log 請輸出 stderr。
- 不要讓 OpenCode plugin 變厚；核心邏輯留在 Rust。
- Codex / Claude 不依賴 OpenCode lifecycle plugin，因此要靠 instructions 和 MCP tools。
