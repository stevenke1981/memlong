# AMS 改善規劃包（原 memlong）

目標：針對 `https://github.com/stevenke1981/memlong.git` 做專案分析，提出一套可讓 OpenCode、Codex、Claude Code 都能使用的長期記憶系統改善方案。

本包不是原始碼覆蓋包，而是「給 coding agent 執行的規格與工作包」。可直接丟給 OpenCode / Codex / Claude，要求它依照 `AGENTS.md`、`spec.md`、`todos.md`、`test.md` 逐步修改專案。

## 內容

- `analysis.md`：目前專案分析、優點、風險與改善方向。
- `plan.md`：分階段改善計畫。
- `spec.md`：改版後技術規格。
- `todos.md`：可交給 agent 執行的任務清單。
- `test.md`：測試策略、驗收命令與 CI gate。
- `final.md`：交付摘要與驗收標準。
- `AGENTS.md`：OpenCode / Codex 通用 agent 工作規則。
- `CLAUDE.md`：Claude Code 專用專案記憶與行為規則。
- `configs/opencode/opencode.memlong.jsonc`：OpenCode MCP 設定範例。
- `configs/codex/config.memlong.toml`：Codex MCP 設定範例。
- `configs/claude/.mcp.memlong.json`：Claude Code MCP 設定範例。
- `scripts/verify.sh`、`scripts/verify.ps1`：Linux/macOS 與 Windows 驗證腳本。

## 使用方式

1. 將本包解壓到 `memlong` 專案根目錄旁邊，或把文件複製到專案根目錄。
2. 讓 agent 先讀：
   - `AGENTS.md`
   - `analysis.md`
   - `spec.md`
   - `todos.md`
   - `test.md`
3. 指令範例：

```text
請根據 AGENTS.md、spec.md、todos.md、test.md 對 memlong 專案進行改善。先執行 baseline 測試，再依 todos.md 分批修改，每一批都要跑 test.md 中的 gate。
```

## 建議執行順序

1. Phase 0：baseline 建置與測試。
2. Phase 1：文件與設定相容性修正。
3. Phase 2：安裝器與 MCP 設定可靠化。
4. Phase 3：核心資料一致性、維度檢查與 graceful degradation。
5. Phase 4：跨 OpenCode / Codex / Claude 的測試矩陣。
5. Phase 5：release artifact、版本與 final report。
