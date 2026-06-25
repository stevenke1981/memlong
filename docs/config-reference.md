# 設定參考 (Config Reference)

## 環境變數

| 變數 | 預設值 | 說明 |
|------|--------|------|
| `LLM_API_BASE` | `http://localhost:8080/v1` | OpenAI 相容 API base URL |
| `LLM_API_KEY` | `local` | API 金鑰 |
| `EXTRACTION_MODEL` | `gpt-4o-mini` | 用於記憶抽取的 chat model |
| `EMBEDDING_MODEL` | `text-embedding-3-small` | 用於 embedding 的 model |
| `EMBEDDING_DIM` | `1536` | Embedding 向量維度 |
| `MEMORY_DB_PATH` | `{data_dir}/memories.db` | SQLite 資料庫路徑 |
| `MEMORY_VECTOR_PATH` | `{data_dir}/vectors.usearch` | USearch 向量索引路徑 |
| `MEMORY_TANTIVY_PATH` | `{data_dir}/tantivy` | Tantivy 全文索引目錄 |
| `MEMORY_LOG_LEVEL` | `info` | 日誌等級 (debug/info/warn/error) |
| `MEMORY_TEMPORAL_MU` | `0.05` | 時間衰減係數 |
| `MEMORY_DEDUP_THRESHOLD` | `0.92` | 重複判斷 cosine 閾值 |
| `MEMORY_NEAR_DEDUP_THRESHOLD` | `0.75` | 近似重複 cosine 閾值 |
| `MEMORY_DECAY_LAMBDA` | `0.001` | Ebbinghaus 衰減 lambda |
| `MEMORY_MIN_CONFIDENCE` | `0.60` | 抽取最低 confidence |
| `MEMORY_MIN_IMPORTANCE` | `2` | 抽取最低 importance (1-5) |

## data_dir 決定規則

`data_dir` 依序決定於：

1. `MEMORY_DB_PATH` 有設定 → 使用其目錄
2. `PROJECT_ROOT` 有設定 → `{PROJECT_ROOT}/.opencode`
3. 當前目錄 → `./.opencode`

## OPENCODE / CODEX / CLAUDE 共用設定

### 必要

```text
LLM_API_BASE=http://localhost:8080/v1
LLM_API_KEY=local
```

### 建議

```text
EXTRACTION_MODEL=gpt-4o-mini
EMBEDDING_MODEL=text-embedding-3-small
EMBEDDING_DIM=1536
```

### 本地開發 (mock)

```text
LLM_API_BASE=mock
LLM_API_KEY=mock
EMBEDDING_DIM=1536
```
