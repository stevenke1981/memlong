# rmcp Server Reference

## Cargo Setup

Use the official Rust MCP SDK:

```toml
[dependencies]
rmcp = { version = "1", features = ["server", "transport-io", "macros"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

Prefer the repo's exact current rmcp version. Do not silently downgrade unless a specific compatibility bug requires it.

## Main Entrypoint

Pattern:

```rust
#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        // CLI commands: install, --version, workflow --json, etc.
        return;
    }

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    if let Err(error) = MyServer::new().serve_stdio().await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
```

Rules:

- No stdout logs in MCP mode. Stdio stdout is for JSON-RPC frames only.
- CLI mode may print JSON or text to stdout.
- Support `--version`, `-V`, and optionally `version`; many agents use this as a cheap health check.

## Tool Router

Use typed router inputs:

```rust
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SearchInput {
    query: String,
    limit: Option<u64>,
}

#[tool_router(router = tool_router)]
impl MyServer {
    #[tool(name = "search", description = "Search indexed content.")]
    async fn search(
        &self,
        cancellation: tokio_util::sync::CancellationToken,
        Parameters(input): Parameters<SearchInput>,
    ) -> Result<CallToolResult, ErrorData> {
        // Check cancellation before expensive work and while waiting.
    }
}
```

## Cancellation

Long-running tools should take `CancellationToken` and wire it through `tokio::select!`. For blocking domain code, run it in `spawn_blocking` and return a cancellation error if the token fires before the worker returns.

## Schema Compatibility

Schemars can emit JSON Schema boolean nodes for `serde_json::Value`, for example:

```json
{
  "properties": {
    "output": true
  }
}
```

OpenCode's MCP SDK validation rejects boolean schema nodes inside tool input schemas. Normalize only schema nodes:

- `properties.<name>: true` -> `properties.<name>: {}`
- `items: true` -> `items: {}`
- `$defs.<name>: true` -> `$defs.<name>: {}`
- Preserve normal boolean values such as `default: false`, `enum: [true]`, and user data.

If using rmcp macros, overriding a helper like `rmcp_tool_definitions()` may not affect the actual server `tools/list`. Override `ServerHandler::list_tools()` and `get_tool()` or normalize the router entries before they are served.

## Tests

Minimum contracts:

- Server info name and tools capability.
- `tools/list` returns expected count and names.
- Tool list snapshot.
- No boolean JSON Schema nodes in schema positions.
- Official rmcp client can initialize, list tools, and call at least one tool.
- Release binary can initialize and list tools through stdio.

