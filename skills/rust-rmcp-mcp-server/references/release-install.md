# Release and Install Reference

## No-Compile Default

Default installers should download release assets. Compilation must be opt-in.

Good:

```powershell
.\install.ps1
.\install.ps1 -Version v0.1.6
.\install.ps1 -FromSource
```

Bad:

- Manifest says `target/release/server.exe`.
- README tells agents to run `cargo build --release` for normal install.
- Installer builds unless a release download fails.

## Windows Paths

Recommended binary path:

```text
%USERPROFILE%\.config\<server-name>\bin\<server-name>.exe
```

When the stable binary is locked by a running agent:

1. Try copying to stable path.
2. If the copy fails with locked/access denied/cannot access file, copy to `<server-name>-<version>.exe`.
3. Run `<installed-binary> install --json`.
4. Configure OpenCode/Codex to the actual installed path.
5. Print restart-required.

This avoids asking users to kill OpenCode just to update config.

## Release Artifacts

Typical assets:

- `<name>-<version>-x86_64-pc-windows-msvc.zip`
- `<name>-<version>-x86_64-unknown-linux-gnu.tar.gz`
- `<name>-<version>-aarch64-apple-darwin.tar.gz`
- `.sha256` per asset or one `SHA256SUMS.txt`

Windows zip should include:

- binary
- README or install hint if useful
- MCP templates or skill files if the project distributes them

## Version Bump Checklist

Update all:

- `Cargo.toml`
- `Cargo.lock`
- `packaging/mcp/manifest.json`
- tool snapshot version
- README pinned examples
- installer comments or docs
- release package names

## Verification Before Tag

Run:

```powershell
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release
.\scripts\package-release.ps1
.\install.ps1 -FromSource -SkipBuild
opencode --pure mcp list
```

Also run the OpenCode SDK smoke script, because rmcp's Rust client can pass while OpenCode's TypeScript MCP SDK rejects the public schema.

## GitHub Release Flow

1. Commit the tested code.
2. Push main.
3. Tag the same commit, for example `git tag v0.1.6`.
4. Push the tag.
5. Wait until GitHub Release assets exist.
6. Run default installer with no source/build flags.
7. Confirm it downloads the new release and does not compile.

Avoid excessive unauthenticated GitHub API polling; it can hit rate limits. If rate-limited, open the release URL or wait and retry.

