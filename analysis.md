# memlong 專案分析

分析日期：2026-06-25

## 1. 專案定位

`memlong` 是一個 local-first 的 coding agent 長期記憶系統，核心使用 Rust，透過 MCP stdio server 暴露工具，並搭配 OpenCode TypeScript lifecycle shim。目標是讓 coding agent 在跨 session 時仍能保留偏好、決策、錯誤經驗、程式碼模式與專案知識。

目前 README 描述的核心能力：

- Rust 核心。
- MCP server。
- OpenCode lifecycle plugin shim。
- SQLite metadata。
- USearch HNSW vector index。
- Tantivy BM25 text index。
- Hybrid semantic + BM25 + temporal retrieval。
- MCP tools：`add_memory`、`search_memories`、`get_memories`、`delete_memory`、`consolidate_memories`、`get_memory_stats`、`end_session`。

## 2. 目前架構優點

### 2.1 Rust core / TypeScript thin shim 分層正確

記憶抽取、去重、索引、檢索、衰減與儲存都放在 Rust core。TypeScript plugin 只負責 OpenCode lifecycle hook，避免把業務邏輯散到 JS 層。

### 2.2 MCP stdio 對多 agent 生態友善

MCP stdio server 可以同時被 OpenCode、Codex、Claude Code 以本地 process 啟動方式使用。這是跨工具相容的正確方向。

### 2.3 已有 graceful degradation

`MemoryService::add_memory` 對 extraction / embedding / consolidation 失敗有降級處理，避免 agent session 因記憶寫入失敗而中斷。這符合 coding agent 的穩定性需求。

### 2.4 已有 install command 初步處理 OpenCode / Codex

`memory-mcp-server install --json` 會嘗試更新 OpenCode 與 Codex 設定，並移除 legacy MCP server name。這是可延伸到 Claude Code 的基礎。

### 2.5 CI 有 Rust 與 plugin gate

目前 CI 包含：

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- plugin `npm ci && npm test`

## 3. 主要風險與缺口

### R1. 文件與實作名稱不一致

專案中同時出現：

- `memlong`
- `opencode-memory`
- `memory-mcp-server`
- `opencode-memory.exe`
- release archive `opencode-memory-v0.1.0-*`

建議統一：

- repo：`memlong`
- MCP server id：`memlong-memory`
- binary stable name：`memlong-memory` 或 `opencode-memory`
- legacy aliases：保留但只在 install migration 使用。

若短期不改 binary name，至少在 README / install / config 範例中明確標示。

### R2. OpenCode plugin 與 MCP install 責任尚未完全分離

目前 install 主要更新 MCP server 設定。OpenCode 的 TypeScript plugin 仍需要明確安裝、build、載入或說明。改善方向：

- MCP server：跨工具通用能力。
- OpenCode plugin：只負責 lifecycle 自動注入與自動記憶寫入。
- Codex / Claude：不能依賴 OpenCode plugin lifecycle，應使用 AGENTS/CLAUDE 指示讓 agent 主動呼叫 MCP tools。

### R3. Codex / Claude 的相容配置尚未正式文件化

Codex 需要 `~/.codex/config.toml` 的 `[mcp_servers.<id>]` table。Claude Code 可使用 `claude mcp add --transport stdio ...` 或 `.mcp.json`。專案應補上三套官方範例：

- OpenCode：`opencode.jsonc`
- Codex：`config.toml`
- Claude Code：`.mcp.json` 與 `claude mcp add` 命令

### R4. embedding 維度與 mock 維度風險

`LLM_API_KEY=mock` 時 embedding 回傳固定 1536 維。若測試或環境設定 `EMBEDDING_DIM != 1536`，會出現維度不一致。建議 mock embed 根據 `EMBEDDING_DIM` 或 `EmbeddingConfig` 回傳指定維度。

### R5. index consistency 需要更強交易邊界

理想上新增記憶需保證 SQLite、USearch、Tantivy、entity links 一致。若中途失敗，可能產生孤兒 vector 或 text index document。改善方向：

- insert 流程採「先 SQLite transaction 記 pending，再 vector/text/entity commit，最後 mark active」。
- delete 流程採 tombstone + compaction。
- `health` 增加 index consistency check。

### R6. MCP tool schema / output 需要 token 控制

`get_memories` 與 `search_memories` 若回傳大量完整 memory，會增加 context 壓力。建議：

- default `top_k = 5` 或 `10`。
- 回傳欄位分級：`summary` mode / `full` mode。
- `include_metadata` 預設 false。
- 增加 `max_output_chars` 或 page cursor。

### R7. 安裝器目前缺少 Claude Code 設定

`install` 已處理 OpenCode / Codex，但尚未處理 Claude Code。建議：

- 提供 `install --client claude`。
- 生成 project `.mcp.json` 或輸出 `claude mcp add --scope user --transport stdio memlong-memory -- <binary>`。
- 不自動改全域 Claude 設定，除非使用者明確加 `--global`。

### R8. release 沒有 published assets 時 install 會失敗

README / install 預設嘗試下載 release asset，但 GitHub 顯示目前沒有 releases。建議 README 預設使用 `--from-source`，或在 release pipeline 完成前讓 install script 自動 fallback 到 source build，但要清楚提示需要 Rust toolchain。

## 4. 改善優先順序

| 優先級 | 項目 | 原因 |
|---|---|---|
| P0 | 文件與 config 範例統一 | 讓使用者能正確接入三個 agent |
| P0 | baseline test + CI gate 固定 | 避免 agent 改壞 |
| P1 | install script fallback 與 Claude 支援 | 降低使用門檻 |
| P1 | mock embedding 維度修正 | 測試穩定性 |
| P1 | health / doctor / config validate | 快速定位問題 |
| P2 | index transaction / consistency repair | 生產可靠性 |
| P2 | output pagination / summary mode | 控制 context 成本 |
| P3 | benchmark + retrieval quality eval | 驗證效果不是只靠直覺 |

## 5. 建議目標架構

```text
OpenCode / Codex / Claude Code
        │
        │ MCP stdio
        ▼
memlong-memory MCP server
        │
        ▼
memory-core
  ├── extraction: OpenAI-compatible chat + JSON schema validation
  ├── embedding: dimension-aware provider adapter
  ├── consolidation: ADD-only dedup + entity overlap + decay
  ├── retrieval: semantic + BM25 + temporal + scope filters
  ├── storage: SQLite transaction + USearch + Tantivy
  └── admin: health / doctor / repair / stats / export
```

## 6. Agent 使用模式

### OpenCode

- MCP server 提供 tools。
- OpenCode plugin 在 `onChatStart` 自動注入 memory。
- `onMessageComplete` 非阻塞呼叫 `add_memory`。
- `onSessionEnd` 呼叫 `consolidate_memories` 與 `end_session`。

### Codex

- 透過 `mcp_servers.memlong-memory` 啟動 stdio server。
- 使用 `AGENTS.md` 指示：開始任務先 search，完成任務後 add，重大決策 add。
- 不依賴 OpenCode plugin lifecycle。

### Claude Code

- 使用 `.mcp.json` 或 `claude mcp add --transport stdio`。
- 使用 `CLAUDE.md` 指示：任務開始搜尋、任務結束保存、避免保存敏感資料。
- 可使用 skill / hooks 進一步自動化，但第一階段先用 MCP + instructions。
