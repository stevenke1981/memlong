# OpenCode 整合說明

## 整合方式

OpenCode 透過 `opencode.jsonc` 中的 MCP 設定來啟動 `memlong-memory` server。

## 設定範例

`~/.config/opencode/opencode.jsonc` 或專案 `.opencode/opencode.jsonc`：

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
        "PROJECT_ROOT": "/absolute/path/to/your/project",
        "LLM_API_BASE": "http://localhost:8080/v1",
        "LLM_API_KEY": "local",
        "EXTRACTION_MODEL": "your-chat-model",
        "EMBEDDING_MODEL": "your-embedding-model",
        "EMBEDDING_DIM": "1536",
        "MEMORY_LOG_LEVEL": "info"
      }
    }
  },
  "instructions": [
    "AGENTS.md"
  ]
}
```

## 安裝方式

```bash
# 從 source build
cargo build --release
./target/release/memory-mcp-server install --client opencode
```

或使用 install script：

```bash
# Linux/macOS
bash install.sh --client opencode
```

```powershell
# Windows PowerShell
.\install.ps1 -Client opencode
```

## OpenCode Plugin（選用）

`plugin/` 目錄提供 TypeScript lifecycle shim，可自動在對話開始時注入記憶、結束時儲存記憶。

安裝方式：

```bash
cd plugin
npm ci
npm run build
# 複製到 .opencode/plugins/ 或 global plugins 目錄
```

詳見 `plugin/README.md`。

## 驗證

```bash
memory-mcp-server health
```

或在 OpenCode 中輸入：

```text
用 memlong-memory 搜尋關於這個專案的記憶
```
