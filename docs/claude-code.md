# Claude Code 整合說明

## 整合方式

Claude Code 支援兩種方式啟動 MCP server：

1. **專案 `.mcp.json`** — 放在專案根目錄
2. **`claude mcp add` 命令** — 全域或專案範圍

## 設定範例

### 方式一：專案 `.mcp.json`

在專案根目錄建立 `.mcp.json`：

```json
{
  "mcpServers": {
    "ams-memory": {
      "command": "/absolute/path/to/ams-memory",
      "args": [],
      "env": {
        "PROJECT_ROOT": "/absolute/path/to/your/project",
        "LLM_API_BASE": "http://localhost:8080/v1",
        "LLM_API_KEY": "local",
        "EXTRACTION_MODEL": "your-chat-model",
        "EMBEDDING_MODEL": "your-embedding-model",
        "EMBEDDING_DIM": "1536",
        "MEMORY_LOG_LEVEL": "info"
      },
      "timeout": 120000
    }
  }
}
```

### 方式二：claude mcp add 命令

```bash
claude mcp add --transport stdio --scope user ams-memory -- /absolute/path/to/ams-memory
```

## 安裝方式

install script 支援 Claude Code：

```bash
# Linux/macOS
bash install.sh --client claude
```

```powershell
# Windows PowerShell
.\install.ps1 -Client claude
```

或手動安裝後執行：

```bash
ams install --client claude
```

## 使用方式

Claude Code **不依賴 OpenCode plugin**。請透過 `CLAUDE.md` 指示 agent 主動呼叫 MCP tools。

### 任務開始時搜尋

```
用 ams-memory 搜尋關於這個專案的記憶
```

### 任務結束時儲存

```
用 ams-memory 儲存一個持久記憶：這個專案的測試通過了。
```

## 驗證

```bash
ams health
claude mcp list
```
