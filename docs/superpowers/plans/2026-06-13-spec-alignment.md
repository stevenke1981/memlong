# OpenCode Memory Specification Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the existing implementation into behavioral alignment with `opencode-memory-system.md`, focusing on durable USearch storage, ADD-only consolidation boundaries, scoped operations, index consistency, and plugin interoperability.

**Architecture:** Preserve the current `MemoryService` orchestration and storage boundaries. Replace the flat vector fallback behind `VectorStore`, add missing SQLite helpers so engines no longer query its pool directly, and keep cross-store cleanup explicit at the service/consolidation boundary.

**Tech Stack:** Rust 2021, Tokio, sqlx/SQLite, USearch, Tantivy, rmcp, TypeScript 5.

---

### Task 1: Lock Down ADD-only and Vector Lifecycle Behavior

**Files:**
- Modify: `crates/memory-core/tests/lifecycle_test.rs`
- Create: `crates/memory-core/tests/vector_store_test.rs`

- [ ] Add an integration test with `dedup_threshold = 1.0` proving cosine similarity exactly at the threshold is deduplicated.
- [ ] Add an integration test proving identical embeddings in different scopes/projects do not deduplicate each other.
- [ ] Add a vector-store test proving `remove` excludes an ID immediately and after reopening the persisted index.
- [ ] Run `cargo test -p memory-core --tests` and confirm the new assertions fail against the current implementation.

### Task 2: Replace the Flat Vector Fallback with USearch

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/memory-core/Cargo.toml`
- Modify: `crates/memory-core/src/storage/vector.rs`
- Modify: `crates/memory-core/src/error.rs`

- [ ] Add the `usearch = "2"` workspace dependency.
- [ ] Construct a cosine/F32 USearch index using the configured embedding dimensions.
- [ ] Load an existing index when present; otherwise reserve capacity and persist an empty index.
- [ ] Implement add, search, remove, save, and size with dimension validation and error mapping.
- [ ] Run the vector-store test until removal and reopen persistence pass.

### Task 3: Correct Consolidation Boundaries and Scope Isolation

**Files:**
- Modify: `crates/memory-core/src/storage/sqlite.rs`
- Modify: `crates/memory-core/src/consolidation/engine.rs`

- [ ] Add `get_memory_by_vector_id` to `SqliteStore` and stop reaching into its pool from engines.
- [ ] Treat similarity `>= dedup_threshold` as exact duplicate and `>= near_dedup_threshold` as near duplicate.
- [ ] Compare duplicate candidates only when their scope/project identity matches the incoming memory.
- [ ] Run the threshold and scope tests until both pass.

### Task 4: Keep SQLite, Vector, Text, and Entity State Consistent

**Files:**
- Modify: `crates/memory-core/src/storage/sqlite.rs`
- Modify: `crates/memory-core/src/storage/text_index.rs`
- Modify: `crates/memory-core/src/service.rs`
- Test: `crates/memory-core/tests/lifecycle_test.rs`

- [ ] Fetch the memory before deleting its SQLite row.
- [ ] Remove its vector ID and Tantivy document when deletion succeeds.
- [ ] Remove the memory ID from linked entities and delete empty entity records.
- [ ] Add rollback cleanup when a later index write fails during insertion.
- [ ] Add and run a delete/re-add lifecycle test proving stale vectors do not remain.

### Task 5: Honor Scoped Consolidation and Validate Retrieval Inputs

**Files:**
- Modify: `crates/memory-core/src/storage/sqlite.rs`
- Modify: `crates/memory-core/src/consolidation/engine.rs`
- Modify: `crates/memory-core/src/service.rs`
- Modify: `crates/memory-mcp-server/src/server.rs`
- Modify: `crates/memory-core/src/models/query.rs`
- Test: `crates/memory-core/tests/lifecycle_test.rs`

- [ ] Pass optional scope/project filters from the MCP tool through `MemoryService` into batch consolidation.
- [ ] Query only matching memories for decay updates.
- [ ] Reject `top_k = 0`, non-finite/negative weights, and weights whose sum is not 1.0.
- [ ] Add tests for scoped consolidation and invalid hybrid weights.

### Task 6: Make the TypeScript Shim Accept Real MCP Result Shapes

**Files:**
- Modify: `plugin/src/index.ts`
- Modify: `plugin/package.json`
- Create: `plugin/src/index.test.ts`

- [ ] Export a parser/formatter that accepts a direct array, `{ results: [...] }`, or rmcp text content containing JSON.
- [ ] Ignore malformed entries rather than injecting `undefined` into the system prompt.
- [ ] Add a Node test script and verify all supported response shapes.
- [ ] Run `npm ci`, `npm test`, and `npm run build`.

### Task 7: Full Verification

**Files:**
- Modify only files required by failures found during verification.

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test --workspace`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Run `cargo build --release` and record the server binary size.
- [ ] Run Plugin tests/build and inspect `git diff --check` plus `git status --short`.
