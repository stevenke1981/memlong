# OpenCode / Codex / Claude 相容性設計

## OpenCode

OpenCode 使用 JSON/JSONC config，MCP server 放在 `mcp` 物件下。local server 需要：

```jsonc
{
  "type": "local",
  "command": ["/path/to/server"],
  "enabled": true,
  "environment": {}
}
```

OpenCode 額外可載入 plugin，適合做 lifecycle 自動化。

## Codex

Codex 使用 TOML：

```toml
[mcp_servers.ams-memory]
command = "/path/to/server"
args = []
env = { LLM_API_KEY = "mock" }
```

Codex 不應依賴 OpenCode plugin。請用 `AGENTS.md` 指示 agent 主動呼叫 MCP tools。

## Claude Code

Claude Code 可以用：

```bash
claude mcp add --transport stdio --scope user ams-memory -- /path/to/server
```

或 project `.mcp.json`：

```json
{
  "mcpServers": {
    "ams-memory": {
      "command": "/path/to/server",
      "args": [],
      "env": {}
    }
  }
}
```

Claude Code 使用 `CLAUDE.md` 作為專案記憶與行為規則。
