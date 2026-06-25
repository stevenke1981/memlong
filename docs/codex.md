# Codex 整合說明

## 整合方式

Codex 使用 `~/.codex/config.toml` 中的 `[mcp_servers]` 設定來啟動 `ams-memory` server。

## 設定範例

`~/.codex/config.toml`：

```toml
[mcp_servers.ams-memory]
command = "/absolute/path/to/ams-memory"
args = []
enabled = true
startup_timeout_sec = 30
tool_timeout_sec = 120
required = false
env = {
  PROJECT_ROOT = "/absolute/path/to/your/project",
  LLM_API_BASE = "http://localhost:8080/v1",
  LLM_API_KEY = "local",
  EXTRACTION_MODEL = "your-chat-model",
  EMBEDDING_MODEL = "your-embedding-model",
  EMBEDDING_DIM = "1536",
  MEMORY_LOG_LEVEL = "info"
}
```

## 安裝方式

```bash
# 從 source build
cargo build --release
./target/release/ams install --client codex
```

或使用 install script：

```bash
# Linux/macOS
bash install.sh --client codex
```

```powershell
# Windows PowerShell
.\install.ps1 -Client codex
```

## 使用方式

Codex **不依賴 OpenCode plugin**。請透過 `AGENTS.md` 指示 agent 主動呼叫 MCP tools：

- 任務開始：`search_memories` 以取得相關記憶
- 任務中：`add_memory` 保存持久資訊
- 任務結束：`consolidate_memories` 與 `end_session`

## 驗證

```bash
ams health
```
