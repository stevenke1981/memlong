# memlong 改善任務清單 todos.md

## Gate 0：開始前

- [ ] 建立分支：`chore/memlong-compat-plan`。
- [ ] 執行 baseline：`scripts/verify.sh` 或 `scripts/verify.ps1`。
- [ ] 將 baseline 結果寫入 `docs/baseline.md`。
- [ ] 確認目前 `LLM_API_KEY=mock` 測試可跑。

## P0：文件與命名

- [ ] 決定正式 server id：`memlong-memory` 或 `opencode-memory`。
- [ ] README 統一 server id / binary name / release artifact name。
- [ ] 新增 `docs/opencode.md`。
- [ ] 新增 `docs/codex.md`。
- [ ] 新增 `docs/claude-code.md`。
- [ ] 新增 `docs/config-reference.md`。
- [ ] 補上「release 尚未發布時請使用 source build」說明。
- [ ] 補上 Windows PowerShell 安裝範例。
- [ ] 補上 Linux/macOS 安裝範例。
- [ ] 補上 uninstall / remove MCP config 說明。

## P0：Agent instructions

- [ ] 將 `AGENTS.md` 放到 repo root，供 OpenCode / Codex 讀取。
- [ ] 將 `CLAUDE.md` 放到 repo root，供 Claude Code 讀取。
- [ ] 指示 agent：任務開始先 `search_memories`。
- [ ] 指示 agent：任務完成後只保存 durable memories。
- [ ] 指示 agent：不得保存 secrets。
- [ ] 指示 agent：stdout clean for MCP。

## P1：Config snippets

- [ ] 新增 `examples/opencode/opencode.jsonc`。
- [ ] 新增 `examples/codex/config.toml`。
- [ ] 新增 `examples/claude/.mcp.json`。
- [ ] 新增 `examples/env/local-llm.env.example`。
- [ ] 新增 `examples/env/openai-compatible.env.example`。
- [ ] 文件說明 `PROJECT_ROOT` 對 `.opencode/` 資料位置的影響。

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

- [ ] Mock embedding 依 `EMBEDDING_DIM` 回傳向量。
- [ ] 增加 test：`EMBEDDING_DIM=8` 時 mock add/search 可通過。
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

## Final gate

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo build --release`
- [ ] `cd plugin && npm ci && npm test`
- [ ] MCP server `health` returns ok。
- [ ] OpenCode config parse smoke test。
- [ ] Codex config parse smoke test。
- [ ] Claude `.mcp.json` parse smoke test。
- [ ] `doctor --json` reports ok or actionable warning。
- [ ] `final.md` 更新實際完成項目。
