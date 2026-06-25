$ErrorActionPreference = "Stop"

if (-not $env:LLM_API_BASE) { $env:LLM_API_BASE = "mock" }
if (-not $env:LLM_API_KEY) { $env:LLM_API_KEY = "mock" }
if (-not $env:EXTRACTION_MODEL) { $env:EXTRACTION_MODEL = "mock-chat" }
if (-not $env:EMBEDDING_MODEL) { $env:EMBEDDING_MODEL = "mock-embedding" }
if (-not $env:EMBEDDING_DIM) { $env:EMBEDDING_DIM = "1536" }

cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release

$server = Join-Path (Get-Location) "target\release\memory-mcp-server.exe"
if (Test-Path $server) {
  & $server health
}

if (Test-Path "plugin") {
  Push-Location plugin
  npm ci
  npm test
  Pop-Location
}

Write-Host "verify.ps1 completed successfully"
