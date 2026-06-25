# Memlong

Memlong is a local-first long-term memory system for coding agents. It stores durable facts, preferences, decisions, code patterns, and project knowledge across sessions, then retrieves relevant memories through hybrid semantic, keyword, and temporal ranking.

The core is implemented in Rust and exposed as an MCP server (`memlong-memory`). A small TypeScript shim provides optional OpenCode lifecycle hooks for automatic retrieval and capture.

**Supported clients:** OpenCode, Codex, Claude Code — all through MCP stdio.

## What It Provides

- Single-pass LLM memory extraction with confidence and importance filtering
- ADD-only consolidation with exact and near-duplicate detection
- Project, global, session, and agent memory scopes
- USearch HNSW vector search with persistent local indexes
- Tantivy BM25 full-text search
- Ebbinghaus-inspired retention decay and access reinforcement
- MCP tools for adding, searching, listing, deleting, consolidating, and inspecting memories
- A thin OpenCode plugin for automatic session injection and capture

## Architecture

```text
OpenCode / Codex / Claude Code
          |
          | MCP stdio JSON-RPC
          v
memory-mcp-server (memlong-memory)
          |
          v
memory-core
  |-- SQLite metadata and entities
  |-- USearch HNSW vectors
  |-- Tantivy BM25 index
  `-- extraction, consolidation, retrieval, decay
```

### 三層架構

| 層 | 說明 | 適用 client |
|---|---|---|
| **MCP server** (`memory-mcp-server`) | 跨工具通用能力，提供 `search_memories`、`add_memory` 等 tools | OpenCode, Codex, Claude Code |
| **OpenCode plugin** (`plugin/`) | 僅 OpenCode lifecycle 自動注入與自動記憶寫入 | OpenCode |
| **Agent instructions** (`AGENTS.md`, `CLAUDE.md`) | 指示 Codex / Claude agent 主動呼叫 MCP tools | Codex, Claude Code |

Codex 與 Claude Code **不依賴 OpenCode plugin**，而是透過專案 `AGENTS.md` / `CLAUDE.md` 指示 agent 主動使用 MCP tools。

Default project-local data is stored under `.opencode/`:

```text
.opencode/
|-- memory.db
|-- vectors.usearch
`-- tantivy/
```

## Human Quick Start

### Requirements

- Windows 10 or 11
- Rust stable with the MSVC toolchain
- Visual Studio Build Tools with Desktop development with C++
- Node.js 18+ only when building or testing the OpenCode plugin
- An OpenAI-compatible chat completions and embeddings endpoint

### Build From Source

```powershell
git clone https://github.com/stevenke1981/memlong.git
cd memlong
cargo build --release
```

The MCP server is created at:

```text
target\release\memory-mcp-server.exe
```

### Install On Windows

> **Note:** No release assets have been published yet. Use `-FromSource` (source build) which requires Rust toolchain.

Build and install from the checkout:

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1 -FromSource
```

Install a published release when available:

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1 -Version v0.1.0
```

### Install On Linux / macOS

```bash
# Source build (recommended until first release)
bash install.sh --from-source

# With specific client config
bash install.sh --from-source --client all
```

The installer places the executable under `~/.config/memlong-memory/bin` (Linux/macOS) or `%USERPROFILE%\.config\memlong-memory\bin` (Windows) and invokes its `install` command to configure supported MCP clients. Restart the client after installation.

### Uninstall

```powershell
# Windows
.\uninstall.ps1
```

```bash
# Linux / macOS
bash uninstall.sh
```

Or remove manually:
1. Delete the binary directory (`~/.config/memlong-memory/` or `%USERPROFILE%\.config\memlong-memory\`)
2. Remove the MCP server entry from your client config (opencode.jsonc, config.toml, or .mcp.json)

### Configure The Models

Set an OpenAI-compatible endpoint before starting the MCP server:

```powershell
$env:LLM_API_BASE = "http://localhost:8080/v1"
$env:LLM_API_KEY = "local"
$env:EXTRACTION_MODEL = "your-chat-model"
$env:EMBEDDING_MODEL = "your-embedding-model"
$env:EMBEDDING_DIM = "1536"
```

Important optional settings:

| Variable | Default | Purpose |
| --- | --- | --- |
| `PROJECT_ROOT` | current directory | Root used for the `.opencode` data directory |
| `MEMORY_DB_PATH` | `.opencode/memory.db` | SQLite database path |
| `MEMORY_VECTOR_PATH` | `.opencode/vectors.usearch` | USearch index path |
| `MEMORY_TANTIVY_PATH` | `.opencode/tantivy` | Tantivy index directory |
| `MEMORY_DEDUP_THRESHOLD` | `0.92` | Exact duplicate cosine threshold |
| `MEMORY_NEAR_DEDUP_THRESHOLD` | `0.75` | Near-duplicate cosine threshold |
| `MEMORY_MIN_CONFIDENCE` | `0.60` | Minimum extraction confidence |
| `MEMORY_MIN_IMPORTANCE` | `2` | Minimum LLM importance from 1 to 5 |
| `MEMORY_DECAY_LAMBDA` | `0.001` | Importance recency decay rate |
| `MEMORY_DECAY_MU` | `0.05` | Retrieval temporal decay rate |

`EMBEDDING_DIM` must match the embedding model. Existing vector indexes are dimension-specific.

### Health Check

```powershell
.\target\release\memory-mcp-server.exe health
```

### Debug CLI

```powershell
cargo run -p memory-cli -- add --content "User prefers Rust for core services"
cargo run -p memory-cli -- search --query "preferred implementation language"
cargo run -p memory-cli -- list
cargo run -p memory-cli -- stats
cargo run -p memory-cli -- consolidate
```

### OpenCode Plugin

The plugin is a thin lifecycle adapter; memory behavior remains in Rust.

```powershell
cd plugin
npm ci
npm run build
```

The built entry point is `plugin/dist/index.js`. It supports direct arrays, `{ results: [...] }`, and MCP text-content responses.

## MCP Tools

| Tool | Purpose |
| --- | --- |
| `add_memory` | Extract and store memories from text |
| `search_memories` | Hybrid semantic, BM25, and temporal retrieval |
| `get_memories` | Fetch memories by ID or filters |
| `delete_memory` | Delete a memory and clean all indexes |
| `consolidate_memories` | Apply scoped decay and consolidation |
| `get_memory_stats` | Return counts and index health data |

Search weights must be finite, non-negative, and sum to `1.0`.

## Agent Guide

Agents working in this repository should treat `spec.md` and `AGENTS.md` as the authoritative specifications and preserve these contracts:

1. Core memory behavior belongs in Rust. TypeScript remains a thin lifecycle adapter.
2. Memory content is ADD-only. Access statistics, retention, importance, and archival metadata may be updated.
3. Duplicate detection must respect scope and project boundaries.
4. SQLite, USearch, Tantivy, and entity links must remain consistent after insertion or deletion.
5. MCP stdout is reserved for protocol messages; diagnostics go to stderr.
6. Tests use temporary isolated databases and indexes. They must not call real LLM endpoints.

### Code Discovery

This repository is indexed by `codebase-memory-mcp` as `cbrlm+D-memlong`. Prefer graph tools before text search when exploring code:

1. `search_graph` or `rlm_filter`
2. `trace_path`
3. `rlm_read_symbol` or `get_code_snippet`
4. `query_graph`
5. `get_architecture`

Use grep or file search for configuration, documentation, literal error messages, and other non-code content. Re-run `index_repository` after structural changes when the graph is stale.

### Main Code Paths

| Path | Responsibility |
| --- | --- |
| `crates/memory-core/src/service.rs` | High-level orchestration |
| `crates/memory-core/src/extraction/` | LLM extraction and embeddings |
| `crates/memory-core/src/consolidation/` | Deduplication, entity linking, decay |
| `crates/memory-core/src/retrieval/` | Hybrid ranking and filtering |
| `crates/memory-core/src/storage/` | SQLite, USearch, and Tantivy adapters |
| `crates/memory-mcp-server/src/server.rs` | MCP schemas and handlers |
| `plugin/src/index.ts` | OpenCode lifecycle bridge |

### Required Verification

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release
cd plugin
npm ci
npm test
```

The release server should remain below the 20 MB target documented in the specification.

## Packaging

Create the Windows release archive and SHA256 file:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\package-release.ps1 -Version 0.1.0
```

Artifacts are written under `target/`.

## Documentation

- Agent rules: [`AGENTS.md`](AGENTS.md) (OpenCode / Codex), [`CLAUDE.md`](CLAUDE.md) (Claude Code)
- Technical specification: [`spec.md`](spec.md)
- Test & acceptance plan: [`test.md`](test.md)
- Implementation plan: [`plan.md`](plan.md)
- Task list: [`todos.md`](todos.md)
- Client config examples: [`examples/`](examples/)
- OpenCode integration: [`docs/opencode.md`](docs/opencode.md)
- Codex integration: [`docs/codex.md`](docs/codex.md)
- Claude Code integration: [`docs/claude-code.md`](docs/claude-code.md)
- Config reference: [`docs/config-reference.md`](docs/config-reference.md)
- Full product specification: [`opencode-memory-system.md`](opencode-memory-system.md)
- Baseline report: [`docs/baseline.md`](docs/baseline.md)

## License

MIT
