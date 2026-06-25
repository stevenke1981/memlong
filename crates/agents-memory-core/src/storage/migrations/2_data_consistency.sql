-- Data Consistency Migration
-- Adds consistency tracking columns + repair queue

ALTER TABLE memories ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
ALTER TABLE memories ADD COLUMN embedding_model TEXT;
ALTER TABLE memories ADD COLUMN embedding_dim INTEGER;
ALTER TABLE memories ADD COLUMN content_hash TEXT;

-- Index on status for filtering active/deleted/archived memories
CREATE INDEX IF NOT EXISTS idx_mem_status ON memories (status);

-- Index on content_hash for fast duplicate detection
CREATE INDEX IF NOT EXISTS idx_mem_content_hash ON memories (content_hash);

-- Repair Queue: tracks index inconsistencies found during repair operations
CREATE TABLE IF NOT EXISTS index_repair_queue (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    memory_id   TEXT,                    -- UUID of the affected memory (NULL for global issues)
    issue_type  TEXT NOT NULL,           -- e.g. 'orphan_vector', 'missing_vector', 'corrupt_entity', 'content_hash_mismatch'
    details     TEXT NOT NULL DEFAULT '{}',  -- JSON with extra context (expected/found values)
    created_at  INTEGER NOT NULL,        -- Unix timestamp ms
    resolved_at INTEGER,                 -- Unix timestamp ms when resolved (NULL = unresolved)
    resolution  TEXT                     -- e.g. 'repaired', 'deleted_orphan', 'reindexed'
) STRICT;

CREATE INDEX IF NOT EXISTS idx_repair_unresolved
    ON index_repair_queue (resolved_at)
    WHERE resolved_at IS NULL;

-- Update schema version
UPDATE system_config SET value = '2' WHERE key = 'schema_version';
