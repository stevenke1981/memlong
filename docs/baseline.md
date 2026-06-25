# Baseline 驗證報告

建立日期：2026-06-25
分支：`chore/memlong-compat-plan`
環境：Windows PowerShell, mock LLM

## 驗證命令與結果

| 命令 | 結果 | 備註 |
|------|------|------|
| `cargo fmt --all -- --check` | ✅ 通過 | 無格式問題 |
| `cargo test --workspace` | ✅ 通過 | 13 tests passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 通過 | 修復 4 個 clippy 問題後通過 |
| `cargo build --release` | ✅ 通過 | 2m 03s |
| `ams health` | ✅ 通過 | database/vector/text_index 皆 ready |
| `cd plugin && npm ci && npm test` | ✅ 通過 | 2 tests passed |

## Clippy 修復記錄

`crates/agents-memory-core/src/consolidation/engine.rs`:
- `too_many_arguments`：`consolidate_single` 參數 8 個超過預設 7 限制 → 加 `#[allow(clippy::too_many_arguments)]`
- `useless_conversion`（3 處）：`Err(e.into())` 中 e 已是 `MemoryError` → 改為 `Err(e)`

## 環境變數

```powershell
$env:LLM_API_BASE="mock"
$env:LLM_API_KEY="mock"
$env:EXTRACTION_MODEL="mock-chat"
$env:EMBEDDING_MODEL="mock-embedding"
$env:EMBEDDING_DIM="1536"
```

## 已知問題

1. `ams health` 在 Tantivy lock 殘留時會報 `LockBusy`，刪除 `.opencode/tantivy/.tantivy-*.lock` 後恢復正常。
2. `${PROJECT_ROOT}` 字面目錄存在（`D:\memlong\${PROJECT_ROOT}\`），是 config 未正確展開的遺留問題，需在 config 層修復。
