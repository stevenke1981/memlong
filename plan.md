# memlong 改善計畫 plan.md

## 目標

將 `memlong` 改造成一個可穩定支援 OpenCode、Codex、Claude Code 的本地優先長期記憶 MCP server，並建立完整的規格、測試、安裝、驗收與 agent 工作流程。

## 非目標

- 不在第一階段改成雲端服務。
- 不把記憶內容上傳 SaaS。
- 不要求 Codex / Claude 擁有 OpenCode lifecycle plugin。
- 不自動保存敏感個資、金鑰、密碼、token、私密憑證。

## Phase 0：Baseline 與保護網

### 工作

1. 建立 baseline 報告：
   - `cargo fmt --all -- --check`
   - `cargo test --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo build --release`
   - `cd plugin && npm ci && npm test`
2. 將目前失敗項目記錄到 `docs/baseline.md`。
3. 確認所有測試可使用 mock LLM，不依賴真實 API。
4. 加入 `scripts/verify.sh`、`scripts/verify.ps1`。

### 驗收

- 任一修改前後都能比較 baseline。
- 測試失敗要分類為：compile、format、clippy、unit、integration、plugin、mcp、platform。

## Phase 1：文件與相容性規格統一

### 工作

1. 更新 README：
   - 明確標示支援 OpenCode / Codex / Claude Code。
   - 分離 MCP server、OpenCode plugin、agent instructions 三層。
   - release 未發布前，預設安裝指令使用 `--from-source`。
2. 新增：
   - `docs/opencode.md`
   - `docs/codex.md`
   - `docs/claude-code.md`
   - `docs/config-reference.md`
3. 統一命名：
   - server id：`memlong-memory` 或保留 `opencode-memory`，但需全文件一致。
   - binary stable path：全文件一致。
4. 補上 config snippets：
   - OpenCode `opencode.jsonc`
   - Codex `config.toml`
   - Claude `.mcp.json`

### 驗收

- 三個 client 都有 copy-paste 可用範例。
- README 不再暗示 release asset 已存在，除非 release workflow 已完成。

## Phase 2：安裝器與設定修復

### 工作

1. `install.sh` / `install.ps1`：
   - release asset 不存在時可選擇 fallback source build。
   - `--client opencode|codex|claude|all`。
   - `--dry-run` 顯示將修改哪些檔案。
   - `--print-config` 只輸出 config snippet，不寫入。
2. `memory-mcp-server install`：
   - OpenCode：寫入 `mcp` local server。
   - Codex：寫入 `[mcp_servers.memlong-memory]`。
   - Claude：支援輸出 `.mcp.json` 或提示 `claude mcp add` 命令。
3. `install --json` 回傳：
   - binary path
   - configured clients
   - skipped clients
   - warnings
   - restart_required
4. 新增 `doctor` command：
   - 檢查 binary 存在。
   - 檢查 config parse。
   - 檢查 MCP server tools/list。
   - 檢查 DB / vector / Tantivy 可初始化。

### 驗收

- Windows / Linux 皆可安裝。
- 不會覆蓋既有 unrelated MCP server。
- 可移除 legacy server names。
- `doctor --json` 可讓 agent 自動判斷問題。

## Phase 3：核心穩定性與資料一致性

### 工作

1. embedding mock 維度改為由 config 控制。
2. extraction JSON parsing 增加 schema validation 與錯誤分類。
3. SQLite 寫入改為更明確的 transaction 流程。
4. vector / text index 失敗時，能 rollback 或標記 pending repair。
5. 新增 `repair_indexes` admin function：
   - SQLite 為 source of truth。
   - 重建 USearch 與 Tantivy。
6. 新增 `export_memories` / `import_memories`：
   - JSONL 格式。
   - 可備份與跨機器搬移。

### 驗收

- 中斷 insert 不會造成不可恢復狀態。
- `doctor` 能偵測 index mismatch。
- `repair_indexes` 可重建索引並通過 search smoke test。

## Phase 4：檢索品質與 context 成本控制

### 工作

1. `search_memories` 支援 output mode：
   - `brief`：content + category + score。
   - `full`：完整 memory。
2. 支援 cursor / pagination。
3. 支援 `max_output_chars`。
4. 建立 retrieval eval dataset：
   - preferences
   - decisions
   - code patterns
   - error lessons
   - project facts
5. benchmark：
   - 1K / 10K / 50K records。
   - add latency。
   - search p50/p95。

### 驗收

- 預設搜尋輸出不超過合理 context。
- 10K records 下 search p95 有明確目標。
- hybrid retrieval 效果需優於單 BM25 或單 semantic baseline。

## Phase 5：Release 與交付

### 工作

1. 建立 release workflow：
   - Linux x86_64 / aarch64。
   - Windows x86_64-msvc。
   - macOS 可排到後續。
2. release artifact 命名與 install script 一致。
3. 建立 `CHANGELOG.md`。
4. 建立 `docs/final-report-template.md`。
5. 更新 `lessons.md`：記錄 agent 修復過程與常見錯誤。

### 驗收

- 使用者可從 release 或 source 成功安裝。
- OpenCode / Codex / Claude 三套 smoke test 都通過。
- README、config、install、CI、release artifact 命名一致。

## 建議分支策略

```text
main/master
  ├── chore/baseline-and-docs
  ├── feat/client-configs
  ├── feat/install-doctor
  ├── feat/index-repair
  ├── feat/retrieval-output-control
  └── chore/release-workflow
```

## 風險控制

- 每個 phase 都要小 PR。
- 修改 install 前先加測試。
- 修改 storage 前先加資料一致性測試。
- 修改 MCP schema 前先跑 OpenCode / Codex / Claude smoke test。
- 不在同一批 PR 同時改 storage、retrieval、install。
