#!/usr/bin/env bash
set -euo pipefail

export LLM_API_BASE="${LLM_API_BASE:-mock}"
export LLM_API_KEY="${LLM_API_KEY:-mock}"
export EXTRACTION_MODEL="${EXTRACTION_MODEL:-mock-chat}"
export EMBEDDING_MODEL="${EMBEDDING_MODEL:-mock-embedding}"
export EMBEDDING_DIM="${EMBEDDING_DIM:-1536}"

cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release

if [[ -x "./target/release/memory-mcp-server" ]]; then
  ./target/release/memory-mcp-server health
fi

if [[ -d "plugin" ]]; then
  pushd plugin >/dev/null
  npm ci
  npm test
  popd >/dev/null
fi

echo "verify.sh completed successfully"
