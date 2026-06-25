# AMS 改善交付 final.md

## 交付目標

本次改善的最終狀態應達成：

1. AMS 可作為本地 MCP stdio server。
2. OpenCode 可透過 `opencode.jsonc` 使用 MCP tools，並可選用 TypeScript plugin 進行 lifecycle 自動記憶注入。
3. Codex 可透過 `~/.codex/config.toml` 使用 MCP tools。
4. Claude Code 可透過 `.mcp.json` 或 `claude mcp add --transport stdio` 使用 MCP tools。
5. agent instructions 明確規定何時 search、何時 add、何時 consolidate，以及哪些資訊不得保存。
6. 安裝、測試、CI、release artifact 名稱一致。
7. 測試可用 mock LLM 完成，不依賴真實 API。

## 目前建議交付物

- [ ] README 更新。
- [ ] `docs/opencode.md`。
- [ ] `docs/codex.md`。
- [ ] `docs/claude-code.md`。
- [ ] `AGENTS.md`。
- [ ] `CLAUDE.md`。
- [ ] `examples/opencode/opencode.jsonc`。
- [ ] `examples/codex/config.toml`。
- [ ] `examples/claude/.mcp.json`。
- [ ] `scripts/verify.sh`。
- [ ] `scripts/verify.ps1`。
- [ ] `doctor` command。
- [ ] install `--client` / `--dry-run` / `--print-config`。
- [ ] mock embedding dimension fix。
- [ ] index consistency tests。
- [ ] final CI green。

## 最終驗收命令

```bash
export LLM_API_BASE=mock
export LLM_API_KEY=mock
export EXTRACTION_MODEL=mock-chat
export EMBEDDING_MODEL=mock-embedding
export EMBEDDING_DIM=1536

cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release
./target/release/ams health
./target/release/ams doctor --json

cd plugin
npm ci
npm test
```

Windows：

```powershell
$env:LLM_API_BASE="mock"
$env:LLM_API_KEY="mock"
$env:EXTRACTION_MODEL="mock-chat"
$env:EMBEDDING_MODEL="mock-embedding"
$env:EMBEDDING_DIM="1536"

cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release
.\target\release\ams.exe health
.\target\release\ams.exe doctor --json

Set-Location plugin
npm ci
npm test
```

## Agent final report template

```md
# Final Report

## Summary

- Changed:
- Not changed:
- Risk:

## Tests

| Command | Result |
|---|---|
| cargo fmt | pass/fail |
| cargo test | pass/fail |
| cargo clippy | pass/fail |
| cargo build --release | pass/fail |
| server health | pass/fail |
| server doctor | pass/fail |
| npm test | pass/fail |

## Compatibility

| Client | Config | Smoke test |
|---|---|---|
| OpenCode | pass/fail | pass/fail |
| Codex | pass/fail | pass/fail |
| Claude Code | pass/fail | pass/fail |

## Known Issues

- 

## Next Steps

- 
```

## 重要注意

若 release asset 尚未發布，install script 不應讓使用者誤以為可以直接下載成功。文件中應預設使用 source build，或明確標示 release 安裝需要先發布 GitHub Release。
