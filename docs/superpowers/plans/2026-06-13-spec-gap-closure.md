# Spec Gap Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close all 5 critical gaps between `opencode-memory-system.md` spec and current codebase: module extraction, auto decay scheduler, AGENTS.md, and index compaction.

**Architecture:** Decompose `RetrievalEngine` into focused sub-modules (`SemanticRetriever`, `Bm25Retriever`) to match spec layout; add background decay scheduling in `memory-mcp-server`; create AGENTS.md; add Tantivy compaction.

**Tech Stack:** Rust 2021, Tokio (async tasks, timers), USearch, Tantivy, rmcp

---

### Task 1: Extract `retrieval/semantic.rs` — USearch HNSW Semantic Retriever

**Files:**
- Create: `crates/memory-core/src/retrieval/semantic.rs`
- Modify: `crates/memory-core/src/retrieval/mod.rs`
- No test yet (functionality is a move, not new logic)

- [ ] **Step 1: Create `semantic.rs`**

```rust
// crates/memory-core/src/retrieval/semantic.rs

use crate::error::Result;
use crate::storage::VectorStore;
use std::sync::Arc;

/// Semantic Retriever — wraps USearch HNSW index for dense vector similarity search.
pub struct SemanticRetriever {
    vector_store: Arc<VectorStore>,
}

impl SemanticRetriever {
    pub fn new(vector_store: Arc<VectorStore>) -> Self {
        Self { vector_store }
    }

    /// Search for top-k semantically similar vectors.
    /// Returns `Vec<(vector_id, cosine_similarity)>`.
    pub fn search(&self, query_vec: &[f32], top_k: usize) -> Result<Vec<(i64, f32)>> {
        self.vector_store.search(query_vec, top_k)
    }
}
```

- [ ] **Step 2: Update `mod.rs` to expose the new module**

```rust
// crates/memory-core/src/retrieval/mod.rs

pub mod engine;
pub mod hybrid;
pub mod semantic;
pub mod bm25;

pub use engine::RetrievalEngine;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p memory-core`
Expected: success

- [ ] **Step 4: Commit**

```bash
git add crates/memory-core/src/retrieval/semantic.rs crates/memory-core/src/retrieval/mod.rs
git commit -m "feat(retrieval): add SemanticRetriever module matching spec layout"
```

---

### Task 2: Extract `retrieval/bm25.rs` — Tantivy BM25 Keyword Retriever

**Files:**
- Create: `crates/memory-core/src/retrieval/bm25.rs`
- No test yet (pure move)

- [ ] **Step 1: Create `bm25.rs`**

```rust
// crates/memory-core/src/retrieval/bm25.rs

use crate::error::Result;
use crate::retrieval::hybrid::normalize_bm25;
use crate::storage::TextIndex;
use std::sync::Arc;

/// BM25 Retriever — wraps Tantivy text index for keyword-based full-text search.
pub struct Bm25Retriever {
    text_index: Arc<TextIndex>,
}

impl Bm25Retriever {
    pub fn new(text_index: Arc<TextIndex>) -> Self {
        Self { text_index }
    }

    /// Search for top-k BM25-scored documents.
    /// Returns raw Tantivy BM25 scores (not normalized).
    pub fn search_raw(&self, query: &str, top_k: usize) -> Result<Vec<(String, f32)>> {
        self.text_index.search(query, top_k)
    }

    /// Search and return min-max normalized BM25 scores in [0.0, 1.0].
    pub fn search_normalized(&self, query: &str, top_k: usize) -> Result<Vec<(String, f32)>> {
        let raw = self.search_raw(query, top_k)?;
        Ok(normalize_bm25(&raw))
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p memory-core`
Expected: success

- [ ] **Step 3: Commit**

```bash
git add crates/memory-core/src/retrieval/bm25.rs
git commit -m "feat(retrieval): add Bm25Retriever module matching spec layout"
```

---

### Task 3: Refactor `RetrievalEngine` to use new sub-modules

**Files:**
- Modify: `crates/memory-core/src/retrieval/engine.rs`

- [ ] **Step 1: Update `RetrievalEngine` to use `SemanticRetriever` and `Bm25Retriever`**

Replace the existing `RetrievalEngine` struct and impl:

```rust
// crates/memory-core/src/retrieval/engine.rs

use crate::error::Result;
use crate::extraction::LlmClient;
use crate::models::{HybridWeights, Memory, SearchQuery, SearchResult};
use crate::retrieval::{bm25::Bm25Retriever, hybrid::normalize_bm25, semantic::SemanticRetriever};
use crate::storage::SqliteStore;
use std::sync::Arc;

pub struct RetrievalEngine {
    semantic: SemanticRetriever,
    bm25: Bm25Retriever,
    sqlite: Arc<SqliteStore>,
    llm_client: Arc<LlmClient>,
    embedding_model: String,
    default_weights: HybridWeights,
    decay_mu: f64,
}

impl RetrievalEngine {
    pub fn new(
        sqlite: Arc<SqliteStore>,
        vector_store: Arc<crate::storage::VectorStore>,
        text_index: Arc<crate::storage::TextIndex>,
        llm_client: Arc<LlmClient>,
        embedding_model: &str,
        default_weights: HybridWeights,
        decay_mu: f64,
    ) -> Self {
        Self {
            semantic: SemanticRetriever::new(vector_store),
            bm25: Bm25Retriever::new(text_index),
            sqlite,
            llm_client,
            embedding_model: embedding_model.to_string(),
            default_weights,
            decay_mu,
        }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        query.validate()?;
        let weights = query
            .weights
            .clone()
            .unwrap_or_else(|| self.default_weights.clone());
        let fetch_k = query.top_k * 3;

        // 1. Run Semantic and BM25 searches in parallel
        let query_vec = self
            .llm_client
            .embed(&query.query, &self.embedding_model)
            .await?;

        let sem_results = self.semantic.search(&query_vec, fetch_k)?;
        let bm25_results = self.bm25.search_normalized(&query.query, fetch_k)?;

        // 2. Fetch all candidates from SQLite
        let mut candidate_ids = std::collections::HashSet::new();

        let sem_vector_ids: Vec<i64> = sem_results.iter().map(|(vid, _)| *vid).collect();
        let sem_memories = if !sem_vector_ids.is_empty() {
            let mut ids = Vec::new();
            for vid in sem_vector_ids {
                let mem_opt = self.sqlite.get_memory_by_vector_id(vid).await?;
                if let Some(m) = mem_opt {
                    ids.push(m);
                }
            }
            ids
        } else {
            Vec::new()
        };

        for m in &sem_memories {
            candidate_ids.insert(m.id.clone());
        }
        for (mid, _) in &bm25_results {
            candidate_ids.insert(mid.clone());
        }

        let candidate_ids_vec: Vec<String> = candidate_ids.into_iter().collect();
        let all_memories = self.sqlite.get_by_ids(&candidate_ids_vec).await?;

        // 3. Fusion scoring
        let now_ms = chrono::Utc::now().timestamp_millis();
        let mut scored = Vec::new();

        for mem in all_memories {
            if !self.passes_filters(&mem, query) {
                continue;
            }

            // Semantic score
            let s_sem = sem_results
                .iter()
                .find(|(vid, _)| *vid == mem.vector_id)
                .map(|(_, score)| *score as f64)
                .unwrap_or(0.0);

            // BM25 score
            let s_bm25 = bm25_results
                .iter()
                .find(|(mid, _)| mid == &mem.id)
                .map(|(_, score)| *score as f64)
                .unwrap_or(0.0);

            // Temporal score
            let elapsed_ms = now_ms - mem.last_accessed_at;
            let elapsed_days = elapsed_ms as f64 / 86_400_000.0;
            let s_temp = (-self.decay_mu * elapsed_days).exp();

            // Weighted combination
            let score_final =
                weights.semantic * s_sem + weights.bm25 * s_bm25 + weights.temporal * s_temp;

            scored.push(SearchResult {
                memory: mem,
                score_final,
                score_semantic: s_sem,
                score_bm25: s_bm25,
                score_temporal: s_temp,
            });
        }

        // Sort by final score descending
        scored.sort_by(|a, b| {
            b.score_final
                .partial_cmp(&a.score_final)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(query.top_k);

        // 4. Update access statistics asynchronously for matched memories
        let hit_ids: Vec<String> = scored.iter().map(|r| r.memory.id.clone()).collect();
        if !hit_ids.is_empty() {
            let sqlite = self.sqlite.clone();
            tokio::spawn(async move {
                let _ = sqlite.update_access_stats(&hit_ids).await;
            });
        }

        Ok(scored)
    }

    fn passes_filters(&self, mem: &Memory, query: &SearchQuery) -> bool {
        // Scope filter
        if let Some(ref sc) = query.scope {
            if mem.scope != sc.as_str() {
                return false;
            }
        }

        // Project ID filter
        if let Some(ref pid) = query.project_id {
            if mem.project_id.as_ref() != Some(pid) {
                return false;
            }
        }

        // Categories filter
        if let Some(ref cats) = query.categories {
            if cats.is_empty() {
                // empty list means no filter
            } else {
                let mut found = false;
                for cat in cats {
                    if mem.category == cat.as_str() {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
        }

        // Created after filter
        if let Some(created_after) = query.created_after {
            if mem.created_at < created_after {
                return false;
            }
        }

        // Min importance score filter
        if let Some(min_imp) = query.min_importance {
            if mem.importance_score < min_imp {
                return false;
            }
        }

        // Decayed filter
        if !query.include_decayed {
            if mem.retention_factor < 0.1 {
                return false;
            }
            let meta: serde_json::Value =
                serde_json::from_str(&mem.metadata).unwrap_or_default();
            if meta
                .get("archived")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                return false;
            }
        }

        true
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p memory-core`
Expected: success

- [ ] **Step 3: Run existing retrieval-related tests**

Run: `cargo test -p memory-core`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/memory-core/src/retrieval/engine.rs
git commit -m "refactor(retrieval): use SemanticRetriever and Bm25Retriever sub-modules"
```

---

### Task 4: Implement Auto Decay Scheduler

**Files:**
- Create: `crates/memory-core/src/consolidation/scheduler.rs`
- Modify: `crates/memory-core/src/consolidation/mod.rs`
- Modify: `crates/memory-mcp-server/src/main.rs`

- [ ] **Step 1: Create `scheduler.rs` — background decay scheduler**

```rust
// crates/memory-core/src/consolidation/scheduler.rs

use crate::consolidation::ConsolidationEngine;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

/// Background decay scheduler that runs Ebbinghaus decay on a timer.
/// Spawned as a Tokio task, runs every `interval` (default 24 hours).
pub struct DecayScheduler {
    engine: Arc<ConsolidationEngine>,
    interval: Duration,
}

impl DecayScheduler {
    pub fn new(engine: Arc<ConsolidationEngine>, interval: Duration) -> Self {
        Self { engine, interval }
    }

    /// Start the background decay loop. This never returns (runs forever).
    pub async fn run(self) {
        loop {
            tokio::time::sleep(self.interval).await;

            info!("DecayScheduler: starting batch consolidation");
            match self.engine.batch_consolidate(None, None).await {
                Ok(_) => info!("DecayScheduler: batch consolidation completed"),
                Err(e) => error!("DecayScheduler: batch consolidation failed: {e}"),
            }
        }
    }
}
```

- [ ] **Step 2: Update `consolidation/mod.rs` to export the scheduler**

```rust
// crates/memory-core/src/consolidation/mod.rs

pub mod decay;
pub mod dedup;
pub mod engine;
pub mod entity;
pub mod scheduler;

pub use engine::ConsolidationEngine;
pub use scheduler::DecayScheduler;
```

- [ ] **Step 3: Update `memory-mcp-server/src/main.rs` to spawn the scheduler**

Add after `let service = Arc::new(MemoryService::new(config).await?);`:

```rust
    // Spawn background decay scheduler (runs every 24 hours)
    let decay_service = service.clone();
    let scheduler = memory_core::consolidation::DecayScheduler::new(
        decay_service.consolidation_engine(),
        std::time::Duration::from_secs(24 * 60 * 60),
    );
    tokio::spawn(async move {
        scheduler.run().await;
    });
```

Also add a method on `MemoryService` to expose the consolidation engine:

```rust
// In crates/memory-core/src/service.rs, add:

pub fn consolidation_engine(&self) -> Arc<ConsolidationEngine> {
    self.consolidation.clone()
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p memory-core -p memory-mcp-server`
Expected: success

- [ ] **Step 5: Commit**

```bash
git add crates/memory-core/src/consolidation/scheduler.rs crates/memory-core/src/consolidation/mod.rs crates/memory-core/src/service.rs crates/memory-mcp-server/src/main.rs
git commit -m "feat(consolidation): add DecayScheduler with 24h background decay loop"
```

---

### Task 5: Implement Tantivy Index Compaction

**Files:**
- Modify: `crates/memory-core/src/storage/text_index.rs`

- [ ] **Step 1: Add `compact()` method to `TextIndex`**

```rust
// Add to crates/memory-core/src/storage/text_index.rs, inside impl TextIndex:

    /// Compact the Tantivy index by merging segments into a single segment.
    /// This improves search performance and reduces disk usage.
    /// Should be called periodically (e.g., during batch consolidation).
    pub fn compact(&self) -> Result<()> {
        let mut writer_guard = self.writer.lock().map_err(|e| {
            MemoryError::Other(format!("Failed to acquire text index lock: {:?}", e))
        })?;

        let segment_ids = self.index.searchable_segments()?;
        if segment_ids.len() > 1 {
            writer_guard.merge(&segment_ids).map_err(|e| {
                MemoryError::Other(format!("Failed to merge segments: {:?}", e))
            })?;
            writer_guard.commit()?;
            self.reader.reload()?;
        }
        Ok(())
    }
```

- [ ] **Step 2: Update `ConsolidationEngine::batch_consolidate` to call compaction**

```rust
// In crates/memory-core/src/consolidation/engine.rs, add to the end of batch_consolidate:

        // Compact Tantivy index after decay updates
        if let Err(e) = self.text_index.compact() {
            tracing::warn!("Failed to compact text index: {e}");
        }

        Ok(())
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p memory-core`
Expected: success

- [ ] **Step 4: Run all tests**

Run: `cargo test -p memory-core`
Expected: all pass

- [ ] **Step 5: Commit**

```bash
git add crates/memory-core/src/storage/text_index.rs crates/memory-core/src/consolidation/engine.rs
git commit -m "feat(storage): add Tantivy index compaction in batch consolidation"
```

---

### Task 6: Create `docs/AGENTS.md`

**Files:**
- Create: `docs/AGENTS.md`

- [ ] **Step 1: Create `docs/AGENTS.md` — AI Implementation Guide**

```markdown
# OpenCode Memory System — AI Agent Implementation Guide

> This document is an AGENTS.md for the memory system itself, describing how to build,
> modify, and debug the system. It is designed for AI coding agents working on this codebase.

## Architecture Overview

```
memory-mcp-server (bin)          → MCP stdio server, entry point
  └── memory-core (lib)          → All core logic
        ├── extraction/          → LLM Single-Pass memory extraction
        ├── consolidation/       → ADD-only dedup, entity linking, decay
        ├── retrieval/           → Hybrid semantic+BM25+temporal search
        ├── storage/             → SQLite, USearch (HNSW), Tantivy (BM25)
        └── models/              → Data types (Memory, SearchQuery, etc.)
memory-cli (bin)                 → Debug CLI tool
```

## Key Design Decisions

### ADD-only Immutability
Memory content is never mutated after creation. Only access statistics (`access_count`, `last_accessed_at`)
and decay parameters (`importance_score`, `retention_factor`, `metadata.archived`) are updated.
This ensures traceability and simplifies concurrency.

### Single-Pass LLM Extraction
One LLM call extracts all memories from a conversation turn. The extraction prompt
returns structured JSON array. Quality filters (`min_confidence >= 0.6`, `min_importance >= 2`)
remove low-signal memories before storage.

### Dedup Strategy
- **cosine > 0.92**: Exact duplicate → skip, increment access_count on existing
- **0.75 to 0.92**: Near duplicate → entity overlap check → if >50% overlap, treat as synonym
- **< 0.75**: New memory → insert

### Ebbinghaus Decay
Formula: `R(t) = e^(-t/S)` where `S = importance_score * 30` days.
Accessed memories get `S *= 1.2` (reinforcement). `retention_factor < 0.1` → archived.

## Common Tasks

### Adding a New SQLite Migration
1. Create file in `crates/memory-core/src/storage/migrations/V{N}__{description}.sql`
2. `sqlx::migrate!()` in `SqliteStore::new()` auto-runs pending migrations
3. Follow existing naming convention: idempotent (`IF NOT EXISTS` / `OR IGNORE`)

### Adding a New MCP Tool
1. Define input struct with `#[derive(Deserialize, JsonSchema)]` in `server.rs`
2. Add `#[tool(name = "...", description = "...")]` async method on `MemoryMcpServer`
3. Delegate to `MemoryService` method
4. Tools are auto-discovered via `#[tool(tool_box)]` macro

### Testing with Mock LLM
Set `LLM_API_KEY=mock` or `LLM_API_BASE=mock` — the `LlmClient` returns canned responses
for both chat completions and embeddings. No real API needed.

## Debugging

- Log output goes to **stderr** (MCP stdio protocol requires stdout clean for JSON-RPC)
- CLI tool: `cargo run -p memory-cli -- stats` (quick health check)
- MCP Server: `cargo run -p memory-mcp-server -- health` (test initialization)
- Use `MEMORY_LOG_LEVEL=debug` for verbose tracing

## Configuration Reference

| Env Var | Default | Purpose |
|---------|---------|---------|
| `MEMORY_DB_PATH` | `.opencode/memory.db` | SQLite path |
| `MEMORY_VECTOR_PATH` | `.opencode/vectors.usearch` | USearch HNSW path |
| `MEMORY_TANTIVY_PATH` | `.opencode/tantivy` | Tantivy index dir |
| `LLM_API_BASE` | `https://api.anthropic.com/v1` | OpenAI-compatible API |
| `LLM_API_KEY` | `local` | API key (`mock` for testing) |
| `EMBEDDING_DIM` | `1536` | Vector dimensions |

## Project Structure Map

```
crates/memory-core/src/
├── lib.rs              — Public API exports
├── config.rs           — MemoryConfig from env
├── error.rs            — MemoryError enum
├── service.rs          — MemoryService orchestrator
├── models/
│   ├── mod.rs
│   ├── memory.rs       — Memory, MemoryCategory, MemoryScope
│   └── query.rs        — SearchQuery, HybridWeights, SearchResult
├── extraction/
│   ├── mod.rs
│   ├── engine.rs       — ExtractionEngine (LLM call + JSON parse)
│   ├── prompt.rs       — System prompt templates
│   └── llm_client.rs   — HTTP client for LLM/embedding APIs
├── consolidation/
│   ├── mod.rs
│   ├── engine.rs       — ConsolidationEngine (ADD-only insert)
│   ├── dedup.rs        — Duplicate detection logic
│   ├── entity.rs       — Entity linking (entities table)
│   ├── decay.rs        — Ebbinghaus decay math
│   └── scheduler.rs    — Background decay loop
├── retrieval/
│   ├── mod.rs
│   ├── engine.rs       — RetrievalEngine (orchestration)
│   ├── semantic.rs     — SemanticRetriever (USearch HNSW)
│   ├── bm25.rs         — Bm25Retriever (Tantivy)
│   └── hybrid.rs       — BM25 score normalization
└── storage/
    ├── mod.rs
    ├── sqlite.rs       — SqliteStore (CRUD + entity + stats)
    ├── vector.rs       — VectorStore (USearch wrapper)
    ├── text_index.rs   — TextIndex (Tantivy wrapper)
    └── migrations/
        └── 1_init.sql  — Schema init
```

## Version Compatibility

| Dependency | Version | Notes |
|-----------|---------|-------|
| Rust edition | 2021 | Stable |
| sqlx | 0.8 | SQLite + Tokio |
| tantivy | 0.22 | BM25 full-text |
| usearch | 2.x | HNSW vectors |
| rmcp | 0.1 | MCP protocol |
```

- [ ] **Step 2: Commit**

```bash
git add docs/AGENTS.md
git commit -m "docs: add AGENTS.md AI implementation guide"
```

---

### Task 7: Full Integration Verification

**Files:** No file changes

- [ ] **Step 1: Run full test suite**

Run: `cargo test -p memory-core`
Expected: all 5 test files pass

- [ ] **Step 2: Run compilation check for all workspace members**

Run: `cargo check --workspace`
Expected: success

- [ ] **Step 3: Run MCP server health check (smoke test)**

Run: `cargo run -p memory-mcp-server -- health`
Expected: `{"status": "ok", ...}`

- [ ] **Step 4: Final commit**

```bash
git add Cargo.lock
git commit -m "chore: update lockfile after spec gap closure"
```
