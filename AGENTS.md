# AGENTS.md — memlong Agent 工作規則

本文件給 OpenCode、Codex 與其他 coding agents 使用。請在修改 `memlong` 專案前先讀完。

## 1. 工作語言

- 對使用者回覆使用繁體中文。
- 程式碼、commit message、API schema、錯誤訊息可使用英文。
- 文件可中英並列，但以繁體中文說明為主。

## 2. 專案目標

`memlong` 是 local-first long-term memory MCP server。核心原則：

1. Rust core contains business logic。
2. MCP stdio server exposes tools。
3. OpenCode plugin is lifecycle-only thin shim。
4. Codex / Claude Code use MCP + project instructions, not OpenCode lifecycle plugin。
5. Memory data is local by default under `.opencode/` or configured paths。

## 3. 任務開始流程

每次開始修改前：

1. 讀 `analysis.md`、`spec.md`、`todos.md`、`test.md`。
2. 檢查目前 git 狀態。
3. 執行或至少嘗試 baseline tests。
4. 若 MCP server 已可用，先呼叫 `search_memories` 查詢：
   - project path
   - current task summary
   - recent build/test lessons
5. 不要直接大改 storage / install / MCP schema；先加測試或記錄風險。

## 4. 修改原則

### 4.1 小步提交

每次只處理一類問題：

- docs
- config examples
- install scripts
- core config
- storage
- retrieval
- MCP schema
- plugin
- tests

不要在同一輪同時重構多個子系統。

### 4.2 Gate-driven

每批修改後至少跑相關 gate：

- Rust：`cargo fmt --all -- --check`
- Rust tests：`cargo test --workspace`
- Rust lint：`cargo clippy --workspace --all-targets -- -D warnings`
- Plugin：`cd plugin && npm test`
- MCP：`memory-mcp-server health`

### 4.3 MCP stdout 規則

MCP stdio mode 下：

- stdout 只允許 JSON-RPC protocol output。
- logs、debug、warnings 全部到 stderr。
- health / install / doctor command 可以輸出 JSON 到 stdout，因為不是 MCP stdio mode。

## 5. 記憶使用規則

### 5.1 何時 search

任務開始時搜尋：

```json
{
  "query": "project path + user task summary",
  "scope": "Project",
  "top_k": 8,
  "output_mode": "brief"
}
```

### 5.2 何時 add

只保存 durable information：

- 使用者明確偏好。
- 專案架構決策。
- 能重複使用的 build/test 命令。
- 重要錯誤與修復方式。
- 相容性限制。
- agent 工作流程規則。

### 5.3 不要保存

不得保存：

- API keys、tokens、passwords。
- 私鑰、cookie、憑證。
- 敏感個資。
- 短期暫存訊息。
- 大段原始碼或完整 private conversation。

## 6. Failure class

遇到錯誤要分類：

| Class | 說明 |
|---|---|
| compile | 無法編譯 |
| format | 格式錯誤 |
| lint | clippy/tsc 警告 |
| unit | 單元測試失敗 |
| integration | 跨模組測試失敗 |
| mcp | protocol / schema / stdout 問題 |
| install | config mutation / path escaping 問題 |
| platform | Windows/Linux/macOS 差異 |
| docs | 文件與實作不一致 |

## 7. Boundary contract

### memory-core

- 不知道 OpenCode / Codex / Claude 的設定檔格式。
- 只提供 Rust API。
- 負責資料一致性與檢索品質。

### memory-mcp-server

- 負責 MCP tool schema 與 handler。
- 可提供 `health`、`doctor`、`install` CLI command。
- 不把 logs 寫 stdout。

### plugin

- 僅 OpenCode lifecycle bridge。
- 不實作核心記憶邏輯。
- 不直接碰 SQLite / vector / Tantivy。

### install scripts

- 可 build / copy binary / call install command。
- 不破壞既有 unrelated config。
- 需要 dry-run 與 print-config 模式。

## 8. Done definition

一個任務完成需滿足：

- 相關 tests 通過。
- 文件更新。
- config example 更新。
- 若改 MCP schema，OpenCode / Codex / Claude 範例都同步。
- 若改 install，Windows 與 Unix 都同步。
- final report 說明完成、未完成、測試、風險。
