---
name: rust-rmcp-mcp-server
description: Build, package, install, release, and debug Rust MCP servers implemented with the rmcp crate, especially for OpenCode and Codex on Windows. Use when creating or repairing a Rust stdio MCP server, adding typed Schemars tools, writing install scripts, updating OpenCode mcp config, Codex config.toml, GitHub Release binaries, or diagnosing errors like "failed to get tools", "not connected", locked .exe files, stale target/release paths, or agents compiling instead of downloading release assets.
---

# Rust rmcp MCP Server

Use this skill to keep Rust MCP server work compatible with three real clients at once: rmcp's Rust client, OpenCode's TypeScript MCP SDK path, and Codex config loading. Treat "cargo test passed" as necessary but not sufficient.

## Default Workflow

1. Inspect the repo contract:
   - Confirm binary name, server name, transport, install target paths, and expected tool count.
   - Search for stale names, especially old project names, `target/release` install hints, and mismatched config server names.
   - Check `Cargo.toml`, `src/main.rs`, `src/mcp`, `install.ps1`, `packaging/`, `README.md`, and release workflows.

2. Implement the MCP server with official rmcp:
   - Use stdio transport for local agents.
   - Keep stdout protocol-only in MCP mode; send logs/errors to stderr.
   - Add CLI subcommands such as `install`, `--version`, and a JSON health command.
   - Use typed Schemars router inputs, but validate the public `tools/list` schema against OpenCode too.

3. Package for agents:
   - Default install must download GitHub Release binaries, not compile.
   - Source builds must be opt-in, such as `-FromSource` or `--from-source`.
   - Install to a stable user path like `%USERPROFILE%\.config\<server>\bin\server.exe`.
   - On Windows, if the stable `.exe` is locked by a running agent, install a versioned side-by-side binary and configure agents to that path.

4. Configure OpenCode and Codex:
   - OpenCode local config shape:
     ```json
     {
       "mcp": {
         "server-name": {
           "type": "local",
           "command": ["C:\\Users\\name\\.config\\server\\bin\\server.exe"],
           "enabled": true,
           "timeout": 120000,
           "environment": {}
         }
       }
     }
     ```
   - Codex config shape:
     ```toml
     [mcp_servers.server-name]
     command = "C:/Users/name/.config/server/bin/server.exe"
     args = []
     ```
   - Replace existing blocks atomically and remove legacy server names.

5. Verify with multiple clients:
   - Run `cargo fmt --check`.
   - Run `cargo test --all-targets`.
   - Run `cargo clippy --all-targets -- -D warnings`.
   - Run `cargo build --release`.
   - Run a release binary stdio smoke test.
   - Run OpenCode's own status command: `opencode --pure mcp list`.
   - For `failed to get tools`, run `scripts/opencode_tools_list_smoke.ps1` from this skill.

6. Release:
   - Bump crate/package/manifest/docs versions together.
   - Build release artifacts and checksums.
   - Commit, push main, tag `vX.Y.Z`, and push the tag.
   - Wait for GitHub Release assets before telling another machine to use default install.
   - Re-run the default installer after assets exist to confirm no compile path is used.

## References

Read only the reference needed for the current task:

- `references/rmcp-server.md`: rmcp implementation checklist, stdio, tools, schemas, cancellation, stdout/stderr.
- `references/opencode-codex.md`: OpenCode and Codex config, install registration, and `failed to get tools` debugging.
- `references/release-install.md`: GitHub Release packaging, no-compile installers, Windows locked-binary fallback, versioning.

## High-Risk Checks

Before marking the work done, explicitly check:

- OpenCode uses `@modelcontextprotocol/sdk` validation, not just rmcp's Rust client.
- `tools/list` contains no boolean JSON Schema nodes such as `"field": true`; convert those schema nodes to `{}` while preserving boolean values like `"default": false`.
- Install manifests do not point agents at `target/release`.
- `--version` works.
- The installed binary path in OpenCode config actually exists.
- The default installer can run while an old binary is locked.
- The release tag points at the commit being tested.

