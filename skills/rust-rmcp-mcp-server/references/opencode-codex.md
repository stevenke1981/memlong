# OpenCode and Codex Reference

## OpenCode Config

OpenCode v1 local MCP config:

```json
{
  "mcp": {
    "server-name": {
      "type": "local",
      "command": ["C:\\Users\\name\\.config\\server-name\\bin\\server-name.exe"],
      "enabled": true,
      "timeout": 120000,
      "environment": {}
    }
  }
}
```

Notes:

- `command` is an array. Put the executable as the first element and arguments after it.
- `environment` is valid for MCP entries.
- Write both `opencode.json` and `opencode.jsonc` if they exist. If neither exists, create `opencode.json`.
- Remove legacy server names so OpenCode does not load stale entries.
- Restart OpenCode or open a new session after install; MCP tools are loaded at session start.

Useful commands:

```powershell
opencode --version
opencode --pure mcp list
opencode --pure --print-logs --log-level DEBUG mcp list
```

`--pure` removes plugin noise while still loading MCP config.

## Codex Config

Codex config:

```toml
[mcp_servers.server-name]
command = "C:/Users/name/.config/server-name/bin/server-name.exe"
args = []
```

Rules:

- Use forward slashes or escaped backslashes in TOML.
- Remove old `[mcp_servers.<legacy>]` blocks and their env sub-blocks.
- Preserve unrelated config.
- Keep server name stable; tool names should also be stable.

## Debugging `Failed to get tools`

OpenCode source path:

- `packages/opencode/src/mcp/index.ts` calls `McpCatalog.defs()`.
- `packages/opencode/src/mcp/catalog.ts` calls `client.listTools()` using `@modelcontextprotocol/sdk`.
- If validation fails, OpenCode UI often only shows `Failed to get tools`.

Use the OpenCode SDK smoke script in this skill:

```powershell
powershell -ExecutionPolicy Bypass -File C:\Users\steven\.codex\skills\rust-rmcp-mcp-server\scripts\opencode_tools_list_smoke.ps1 -Binary C:\path\server.exe
```

Common root causes:

- Boolean JSON Schema nodes in `inputSchema`, especially from `serde_json::Value`.
- stdout pollution before or during MCP JSON-RPC frames.
- Server exits after initialize because main thinks no args means CLI help instead of MCP mode.
- `tools/list` timeout from expensive schema generation or slow startup.
- Config command points to `target/release`, a deleted build artifact, or a locked/stale exe.
- OpenCode session was not restarted after install.

## What To Log

When diagnosing a user machine:

- Exact OpenCode version.
- Exact `opencode mcp list` output.
- Exact configured binary path.
- `server.exe --version` output.
- OpenCode SDK `listTools()` smoke output.
- Whether the path is stable or versioned side-by-side.

