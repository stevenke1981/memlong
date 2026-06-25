# memlong 測試與驗收 test.md

## 1. 測試原則

- 所有 core tests 必須能使用 mock LLM，不依賴真實 API。
- MCP stdout 必須保持 JSON-RPC clean；logs 只能到 stderr。
- 改 install 前先寫 config mutation tests。
- 改 storage 前先寫 consistency tests。
- 改 retrieval 前先寫 ranking regression tests。

## 2. Baseline commands

### Linux / macOS

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
./target/release/memory-mcp-server health

cd plugin
npm ci
npm test
```

### Windows PowerShell

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
.\target\release\memory-mcp-server.exe health

Set-Location plugin
npm ci
npm test
```

## 3. 必要測試分類

### 3.1 Config tests

- `MemoryConfig::from_env` 使用預設值。
- `PROJECT_ROOT` 正確決定 `.opencode/` data path。
- `MEMORY_DB_PATH` 可覆蓋 DB path。
- `MEMORY_VECTOR_PATH` 可覆蓋 vector path。
- `MEMORY_TANTIVY_PATH` 可覆蓋 Tantivy path。
- `EMBEDDING_DIM` parse failed 時回 fallback 或回 actionable error。

### 3.2 Extraction tests

- mock complete 回傳合法 JSON。
- malformed JSON 不造成 session crash。
- empty memories 正常回空陣列。
- low confidence 被過濾。
- low importance 被過濾。
- secret-like memory 被過濾。

### 3.3 Embedding tests

- mock embedding length 等於 `EMBEDDING_DIM`。
- 維度不一致時回 `MemoryError::VectorIndex` 並包含 expected/got。
- embedding API failure 時 add_memory skip chunk。

### 3.4 Storage tests

- SQLite schema migrate。
- insert / get / list / delete。
- WAL mode 可開啟。
- vector id unique。
- entity link / unlink。
- session_stats ensure / update / end。

### 3.5 Vector index tests

- add / search / remove。
- restore existing index。
- compact preserves valid keys。
- dimension mismatch error。
- legacy flat index migration 若仍需支援，要有 fixture。

### 3.6 Text index tests

- add document。
- BM25 search returns expected id。
- delete document。
- flush / reload。
- compact。

### 3.7 Consolidation tests

- cosine >= 0.92 skip duplicate。
- cosine 0.75 ~ 0.92 + entity overlap > 0.5 skip near duplicate。
- cosine < 0.75 insert。
- decay formula 正確。
- retention < 0.1 archive。

### 3.8 Retrieval tests

- semantic-only ranking。
- BM25-only ranking。
- temporal-only ranking。
- hybrid ranking。
- scope filter。
- project_id filter。
- category filter。
- min_importance filter。
- output_mode brief/full。
- max_output_chars truncation。

### 3.9 MCP protocol tests

- server starts in stdio mode。
- `tools/list` includes all expected tools。
- `add_memory` valid input succeeds。
- `add_memory` Project without project_id returns invalid params。
- `search_memories` valid input succeeds。
- `get_memory_stats` returns JSON。
- stdout contains only protocol messages in stdio mode。
- logs go to stderr。

### 3.10 Install tests

- OpenCode JSON config creates `mcp.memlong-memory`。
- OpenCode JSONC comments preserved enough or output JSON is valid。
- Existing unrelated MCP entries preserved。
- Legacy memory server names removed。
- Codex TOML existing unrelated blocks preserved。
- Codex legacy MCP blocks removed。
- Claude `.mcp.json` generated correctly。
- `--dry-run` writes nothing。
- `--print-config` writes nothing。

## 4. 三 client smoke test

### 4.1 OpenCode

Expected config:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "memlong-memory": {
      "type": "local",
      "command": ["/absolute/path/to/memlong-memory"],
      "enabled": true,
      "timeout": 120000,
      "environment": {
        "LLM_API_BASE": "mock",
        "LLM_API_KEY": "mock"
      }
    }
  }
}
```

Smoke prompt:

```text
use memlong-memory to search memories for "Rust project preferences" and then add a memory saying this is a smoke test.
```

### 4.2 Codex

Expected config:

```toml
[mcp_servers.memlong-memory]
command = "/absolute/path/to/memlong-memory"
args = []
enabled = true
startup_timeout_sec = 30
tool_timeout_sec = 120
env = { LLM_API_BASE = "mock", LLM_API_KEY = "mock" }
```

Smoke prompt:

```text
Use the memlong-memory MCP server. Search for memories about this project, then add a durable memory that the smoke test passed.
```

### 4.3 Claude Code

Expected `.mcp.json`:

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

Smoke command:

```bash
claude mcp list
```

Smoke prompt:

```text
Use memlong-memory to search project memories, then save a durable lesson that Claude Code MCP smoke test passed.
```

## 5. CI gate

CI must fail on:

- formatting errors
- clippy warnings
- test failures
- plugin build/test failures
- docs examples invalid JSON/TOML when checked by scripts

## 6. Failure classification

Use this table in PR / final report:

| Class | Meaning | Examples | Required action |
|---|---|---|---|
| compile | Rust/TS cannot build | type error | fix before continuing |
| format | fmt/prettier mismatch | rustfmt | auto-format |
| lint | clippy/tsc warnings | unused imports | fix or justify |
| unit | unit test failure | decay math | fix implementation |
| integration | cross-module failure | storage+retrieval | debug boundary |
| mcp | protocol failure | stdout polluted | fix immediately |
| install | config mutation failure | TOML block broken | add regression test |
| platform | Windows/Linux mismatch | path escaping | add platform-specific test |
| docs | example stale | wrong server id | update docs |
