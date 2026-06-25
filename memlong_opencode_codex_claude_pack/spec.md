# memlong 改版技術規格 spec.md

## 1. 系統名稱與相容性

建議名稱：`memlong-memory`

相容目標：

| Client | 整合方式 | 自動 lifecycle | 說明 |
|---|---|---:|---|
| OpenCode | `opencode.jsonc` MCP + optional plugin | 是 | plugin 可自動 search/add/consolidate |
| Codex | `~/.codex/config.toml` MCP | 否 | 透過 `AGENTS.md` 指示 agent 主動呼叫 |
| Claude Code | `.mcp.json` 或 `claude mcp add` | 否/可用 hooks 擴充 | 透過 `CLAUDE.md` 指示 agent 主動呼叫 |

## 2. 架構

```text
┌──────────────────────────────────────────────┐
│ OpenCode / Codex / Claude Code               │
│ - agent loop                                 │
│ - project instructions                       │
│ - MCP client                                 │
└───────────────────────┬──────────────────────┘
                        │ stdio JSON-RPC / MCP
                        ▼
┌──────────────────────────────────────────────┐
│ memlong-memory MCP Server                    │
│ - tools/list                                 │
│ - tools/call                                 │
│ - health / doctor / install                  │
└───────────────────────┬──────────────────────┘
                        │ Rust API
                        ▼
┌──────────────────────────────────────────────┐
│ memory-core                                  │
│ extraction / embedding / consolidation       │
│ retrieval / storage / admin                  │
└───────────────────────┬──────────────────────┘
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
 SQLite metadata   USearch vectors   Tantivy BM25
```

## 3. MCP Tools

### 3.1 add_memory

Input:

```json
{
  "content": "string",
  "scope": "Global|Project|Session|Agent",
  "project_id": "optional string",
  "agent_id": "optional string",
  "session_id": "optional string",
  "metadata": { }
}
```

Rules:

- `Project` scope requires `project_id`.
- `Agent` scope requires `agent_id`.
- Must not throw on transient extraction failure; return empty list with warning in stderr/log.
- Must not store secrets by default.

### 3.2 search_memories

Input:

```json
{
  "query": "string",
  "top_k": 10,
  "scope": "optional",
  "project_id": "optional",
  "session_id": "optional",
  "categories": ["Preference", "Decision"],
  "min_importance": 0.3,
  "weights": { "semantic": 0.6, "bm25": 0.3, "temporal": 0.1 },
  "output_mode": "brief|full",
  "max_output_chars": 12000
}
```

Default:

- `top_k = 10`
- `output_mode = brief`
- `include_decayed = false`

### 3.3 get_memories

Input:

```json
{
  "ids": ["uuid"],
  "scope": "optional",
  "project_id": "optional",
  "limit": 20,
  "cursor": "optional"
}
```

### 3.4 delete_memory

Hard delete should remain available, but agent instructions must prefer archive / decay unless user explicitly asks to delete.

### 3.5 consolidate_memories

Runs:

- retention decay update
- duplicate check
- archive marking
- index compaction

### 3.6 get_memory_stats

Returns:

- total memories
- active / archived count
- category counts
- scope counts
- vector count
- text index doc count if available
- pending repair count

### 3.7 end_session

Marks `ended_at` and optionally triggers session-level consolidation.

### 3.8 doctor

New recommended tool/command.

Checks:

- config parse
- DB open
- vector index open
- Tantivy open
- dimensions match
- tools/list works
- read/write smoke test in temp scope

### 3.9 repair_indexes

New recommended admin tool/command.

Behavior:

- SQLite is source of truth.
- Rebuild USearch and Tantivy from active memories.
- Verify counts.

## 4. Storage schema additions

Recommended additions:

```sql
ALTER TABLE memories ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
ALTER TABLE memories ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE memories ADD COLUMN content_hash TEXT;
ALTER TABLE memories ADD COLUMN embedding_model TEXT;
ALTER TABLE memories ADD COLUMN embedding_dim INTEGER;
```

New table:

```sql
CREATE TABLE IF NOT EXISTS index_repair_queue (
  id TEXT PRIMARY KEY,
  memory_id TEXT NOT NULL,
  reason TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT
) STRICT;
```

## 5. Embedding behavior

- Each stored memory records `embedding_model` and `embedding_dim`.
- Existing index must reject mismatched dimensions with actionable error.
- Mock mode must generate vectors using configured `EMBEDDING_DIM`.
- Changing embedding dimension requires `repair_indexes --reembed` or a new vector index path.

## 6. Extraction behavior

Extraction prompt must return JSON object:

```json
{
  "memories": [
    {
      "content": "third-person standalone statement",
      "category": "Fact|Preference|Decision|ProjectKnowledge|CodePattern|ErrorLesson|Workflow",
      "entities": ["string"],
      "importance": 1,
      "confidence": 0.0,
      "sensitivity": "normal|secret|personal|unsafe"
    }
  ]
}
```

Filter rules:

- `confidence < MEMORY_MIN_CONFIDENCE` => skip.
- `importance < MEMORY_MIN_IMPORTANCE` => skip.
- `sensitivity in secret|unsafe` => skip.
- `personal` => store only if user explicitly asked to remember, or project-level non-sensitive preference.

## 7. Retrieval scoring

Final score:

```text
score = semantic_weight * semantic_score
      + bm25_weight     * bm25_score
      + temporal_weight * temporal_score
```

Defaults:

```text
semantic = 0.60
bm25     = 0.30
temporal = 0.10
```

Temporal:

```text
temporal_score = exp(-MEMORY_TEMPORAL_MU * days_since_last_access)
```

## 8. Agent protocol

### 8.1 Start of task

Agent should search relevant memories:

```json
{
  "query": "project path + user task summary",
  "scope": "Project",
  "project_id": "absolute project path",
  "top_k": 8,
  "output_mode": "brief"
}
```

### 8.2 During task

Agent should add memory only for durable information:

- user preferences
- project architecture decisions
- build/test commands that actually work
- recurring errors and fixes
- important constraints

### 8.3 End of task

Agent should call:

1. `add_memory` for durable lesson.
2. `consolidate_memories` for project scope.
3. `end_session` if session id exists.

## 9. OpenCode integration spec

OpenCode config:

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
        "LLM_API_BASE": "http://localhost:8080/v1",
        "LLM_API_KEY": "local",
        "EXTRACTION_MODEL": "your-chat-model",
        "EMBEDDING_MODEL": "your-embedding-model",
        "EMBEDDING_DIM": "1536"
      }
    }
  }
}
```

## 10. Codex integration spec

Codex config:

```toml
[mcp_servers.memlong-memory]
command = "/absolute/path/to/memlong-memory"
args = []
enabled = true
startup_timeout_sec = 30
tool_timeout_sec = 120
env = { LLM_API_BASE = "http://localhost:8080/v1", LLM_API_KEY = "local", EXTRACTION_MODEL = "your-chat-model", EMBEDDING_MODEL = "your-embedding-model", EMBEDDING_DIM = "1536" }
```

## 11. Claude Code integration spec

Project `.mcp.json`:

```json
{
  "mcpServers": {
    "memlong-memory": {
      "command": "/absolute/path/to/memlong-memory",
      "args": [],
      "env": {
        "LLM_API_BASE": "http://localhost:8080/v1",
        "LLM_API_KEY": "local",
        "EXTRACTION_MODEL": "your-chat-model",
        "EMBEDDING_MODEL": "your-embedding-model",
        "EMBEDDING_DIM": "1536"
      },
      "timeout": 120000
    }
  }
}
```

CLI alternative:

```bash
claude mcp add --transport stdio --scope user memlong-memory -- /absolute/path/to/memlong-memory
```

## 12. Logging

- stdout must be reserved for MCP JSON-RPC.
- logs must go to stderr.
- `MEMORY_LOG_LEVEL=debug` enables verbose logs.
- health / install / doctor command may print JSON to stdout because they are not MCP stdio mode.

## 13. Security

Do not store by default:

- passwords
- API keys
- tokens
- private SSH keys
- full credential files
- sensitive health, identity, political, religious, sexual, precise location information
- private messages unless necessary and user explicitly asks

Before saving ambiguous personal information, prefer not saving.
