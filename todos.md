# memlong 改善任務清單 todos.md

## Gate 0：開始前

- [x] 建立分支：`chore/memlong-compat-plan`。
- [x] 執行 baseline：`scripts/verify.sh` 或 `scripts/verify.ps1`。
- [x] 將 baseline 結果寫入 `docs/baseline.md`。
- [x] 確認目前 `LLM_API_KEY=mock` 測試可跑。

## P0：文件與命名

- [x] 決定正式 server id：`memlong-memory`（MCP server id）。
- [x] README 統一 server id / 三層架構說明 / 多 client 支援。
- [x] 新增 `docs/opencode.md`。
- [x] 新增 `docs/codex.md`。
- [x] 新增 `docs/claude-code.md`。
- [x] 新增 `docs/config-reference.md`。
- [x] 補上「release 尚未發布時請使用 source build」說明。
- [x] 補上 Windows PowerShell 安裝範例。
- [x] 補上 Linux/macOS 安裝範例。
- [x] 補上 uninstall / remove MCP config 說明。

## P0：Agent instructions

- [x] 將 `AGENTS.md` 放到 repo root，供 OpenCode / Codex 讀取。
- [x] 將 `CLAUDE.md` 放到 repo root，供 Claude Code 讀取。
- [x] 指示 agent：任務開始先 `search_memories`。
- [x] 指示 agent：任務完成後只保存 durable memories。
- [x] 指示 agent：不得保存 secrets。
- [x] 指示 agent：stdout clean for MCP。

## P1：Config snippets

- [x] 新增 `examples/opencode/opencode.jsonc`。
- [x] 新增 `examples/codex/config.toml`。
- [x] 新增 `examples/claude/.mcp.json`。
- [x] 新增 `examples/env/local-llm.env.example`。
- [x] 新增 `examples/env/openai-compatible.env.example`。
- [x] 文件說明 `PROJECT_ROOT` 對 `.opencode/` 資料位置的影響（在 docs/config-reference.md 中）。

## P1：Install / Doctor

- [ ] `install.sh` 新增 `--client opencode|codex|claude|all`。
- [ ] `install.ps1` 新增 `-Client opencode|codex|claude|all`。
- [ ] 新增 `--dry-run` / `-DryRun`。
- [ ] 新增 `--print-config` / `-PrintConfig`。
- [ ] release asset 下載失敗時提示 `--from-source`。
- [ ] 可選：`--fallback-source` 自動 source build。
- [ ] `memory-mcp-server install --json` 輸出 warnings 與 skipped clients。
- [ ] 新增 `doctor` command。
- [ ] 新增 `doctor --json`。
- [ ] `doctor` 檢查 OpenCode config。
- [ ] `doctor` 檢查 Codex config。
- [ ] `doctor` 提供 Claude Code 設定建議。

## P1：Mock / Test stability

- [x] Mock embedding 依 `EMBEDDING_DIM` 回傳向量（`LlmClient` 新增 `embedding_dim` 欄位）。
- [x] 增加 test：`EMBEDDING_DIM=8` / 64 / 1536 時 mock 向量長度正確。
- [ ] 增加 test：vector index dimension mismatch 回傳 actionable error。
- [ ] 增加 test：`LLM_API_BASE=mock` + `LLM_API_KEY=local` 可通過。
- [ ] 增加 test：extraction parse failed 不會中斷 session。

## P2：MCP schema 與輸出控制

- [ ] `search_memories` 新增 `output_mode`。
- [ ] `search_memories` 新增 `max_output_chars`。
- [ ] `get_memories` 新增 pagination / cursor。
- [ ] MCP tool descriptions 標明何時使用。
- [ ] 回傳結果加入 `score_breakdown`，但 brief 模式可省略 metadata。

## P2：資料一致性與修復

- [ ] SQLite schema 新增 `status`。
- [ ] SQLite schema 新增 `embedding_model`。
- [ ] SQLite schema 新增 `embedding_dim`。
- [ ] SQLite schema 新增 `content_hash`。
- [ ] 新增 `index_repair_queue`。
- [ ] Insert 流程標記 pending / active。
- [ ] Delete 流程改為 tombstone + compaction，或保留 hard delete 但新增 archive。
- [ ] 新增 `repair_indexes` command。
- [ ] 新增 `repair_indexes` test。
- [ ] `get_memory_stats` 回傳 index health。

## P2：OpenCode plugin

- [ ] plugin build 輸出清楚。
- [ ] README 說明 plugin 如何安裝到 `.opencode/plugins/` 或 global plugins。
- [ ] plugin onSessionEnd 同時呼叫 `end_session`。
- [ ] plugin 對 `projectPath` / `projectId` 使用一致化。
- [ ] plugin test 覆蓋 MCP text content response。
- [ ] plugin test 覆蓋 empty memory response。

## P3：Retrieval eval / benchmark

- [ ] 建立 `tests/fixtures/memories.jsonl`。
- [ ] 建立 retrieval gold queries。
- [ ] benchmark 1K records。
- [ ] benchmark 10K records。
- [ ] benchmark 50K records。
- [ ] 比較 hybrid vs semantic-only vs BM25-only。
- [ ] 將結果寫入 `docs/retrieval-eval.md`。

## P3：Release

- [ ] GitHub Actions release workflow。
- [ ] 產出 Linux x86_64 tar.gz。
- [ ] 產出 Linux aarch64 tar.gz。
- [ ] 產出 Windows x86_64 zip。
- [ ] release artifact name 與 install scripts 一致。
- [ ] `install.sh` 下載 release asset smoke test。
- [ ] `install.ps1` 下載 release asset smoke test。
- [ ] 更新 `CHANGELOG.md`。

## 修復項目 (Phase 0/P1 + 設定修復)

- [x] `${PROJECT_ROOT}` placeholder 防護 — `config.rs` 新增 `is_placeholder()` + fallback
- [x] 清除 `${PROJECT_ROOT}` 字面目錄殘留
- [x] 新增 `from_env` sanitization 測試（4 個新 unit tests）
- [x] Mock embedding 維度固定 1536 bug — `LlmClient` 新增 `embedding_dim` 欄位
- [x] 新增 `mock_embedding_respects_dimension` 測試（8/64/1536 dim）

## Final gate

- [x] `cargo fmt --all -- --check`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo build --release`
- [x] `cd plugin && npm ci && npm test`
- [x] MCP server `health` returns ok。
- [ ] OpenCode config parse smoke test。
- [ ] Codex config parse smoke test。
- [ ] Claude `.mcp.json` parse smoke test。
- [ ] `doctor --json` reports ok or actionable warning。
- [ ] `final.md` 更新實際完成項目。
