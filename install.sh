#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# memlong MCP Server — Unix Install Script
# Supports Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
# ─────────────────────────────────────────────────────────────────────────────

SERVER_NAME="opencode-memory"
BINARY_NAME="memory-mcp-server"
VERSION="v0.1.0"
FROM_SOURCE=false
CLIENT="all"
DRY_RUN=false
PRINT_CONFIG=false

# ── Argument Parsing ──────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version|-V)
            echo "$SERVER_NAME $VERSION"
            exit 0
            ;;
        --from-source)
            FROM_SOURCE=true
            shift
            ;;
        --client|-c)
            CLIENT="$2"
            shift 2
            ;;
        --dry-run|-n)
            DRY_RUN=true
            shift
            ;;
        --print-config|-p)
            PRINT_CONFIG=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --version               Show version"
            echo "  --from-source           Build from source instead of downloading release"
            echo "  --client, -c <name>     Target client: opencode, codex, claude, or all (default)"
            echo "  --dry-run, -n           Preview changes without modifying files"
            echo "  --print-config, -p      Print example config snippets"
            echo "  --help, -h              Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--from-source] [--client opencode|codex|claude|all] [--dry-run] [--print-config]"
            exit 1
            ;;
    esac
done

# ── Detect Platform ───────────────────────────────────────────────────────────

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux)
        TARGET_TRIPLE="unknown-linux-gnu"
        ;;
    darwin)
        TARGET_TRIPLE="apple-darwin"
        ;;
    *)
        echo "Error: Unsupported OS: $OS"
        echo "This script supports Linux and macOS. Windows users should use install.ps1."
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_FOLDER="x86_64"
        ;;
    aarch64|arm64)
        ARCH_FOLDER="aarch64"
        ;;
    *)
        echo "Warning: Unrecognized architecture '$ARCH', defaulting to x86_64"
        ARCH_FOLDER="x86_64"
        ;;
esac

# ── Resolve Paths ────────────────────────────────────────────────────────────

HOME_DIR="${HOME:-$(echo ~)}"
TARGET_DIR="$HOME_DIR/.config/$SERVER_NAME/bin"
STABLE_EXE="$TARGET_DIR/$BINARY_NAME"

# ── Print Config Mode ────────────────────────────────────────────────────────

if $PRINT_CONFIG; then
    echo "Printing example configuration for: $CLIENT"
    echo ""
    case "$CLIENT" in
        all|opencode)
            echo "--- OpenCode (opencode.jsonc) ---"
            echo "Add to ~/.config/opencode/opencode.jsonc:"
            echo '{'
            echo '  "mcp": {'
            echo '    "opencode-memory": {'
            echo '      "type": "local",'
            echo '      "command": ["'"$STABLE_EXE"'"],'
            echo '      "enabled": true,'
            echo '      "timeout": 120000,'
            echo '      "environment": {}'
            echo '    }'
            echo '  }'
            echo '}'
            echo ""
            ;;
    esac
    case "$CLIENT" in
        all|codex)
            echo "--- Codex (config.toml) ---"
            echo "Add to ~/.codex/config.toml:"
            echo "[mcp_servers.opencode-memory]"
            echo "command = \"$STABLE_EXE\""
            echo "args = []"
            echo ""
            ;;
    esac
    case "$CLIENT" in
        all|claude)
            echo "--- Claude (.mcp.json) ---"
            echo "Add to ~/.claude/.mcp.json:"
            echo '{'
            echo '  "mcpServers": {'
            echo '    "opencode-memory": {'
            echo '      "command": "'"$STABLE_EXE"'",'
            echo '      "args": [],'
            echo '      "disabled": false,'
            echo '      "autoApprove": []'
            echo '    }'
            echo '  }'
            echo '}'
            echo ""
            ;;
    esac
    echo "Copy these snippets into your agent config files."
    exit 0
fi

# ── Dry Run Mode ─────────────────────────────────────────────────────────────

if $DRY_RUN; then
    echo "DRY RUN — no changes will be made"
    echo "Binary: $STABLE_EXE"
    echo "Would configure client(s): $CLIENT"
    echo ""
    echo "Run without --dry-run to apply changes."
    exit 0
fi

# ── Locate or Build Binary ──────────────────────────────────────────────────

SOURCE_EXE=""

if ! $FROM_SOURCE; then
    # Normalize version tag
    TAG_VERSION="$VERSION"
    if [[ "$TAG_VERSION" != v* ]]; then
        TAG_VERSION="v$TAG_VERSION"
    fi

    ARCHIVE_NAME="$SERVER_NAME-$TAG_VERSION-$ARCH_FOLDER-$TARGET_TRIPLE.tar.gz"
    URL="https://github.com/stevenke1981/memlong/releases/download/$TAG_VERSION/$ARCHIVE_NAME"

    echo "Attempting to download release $TAG_VERSION from GitHub..."
    echo "URL: $URL"

    TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'memlong-install')"
    ARCHIVE_PATH="$TMP_DIR/$ARCHIVE_NAME"

    if command -v curl &>/dev/null; then
        curl -sL -o "$ARCHIVE_PATH" "$URL" || true
    elif command -v wget &>/dev/null; then
        wget -q -O "$ARCHIVE_PATH" "$URL" || true
    else
        echo "Warning: Neither curl nor wget found. Cannot download release."
    fi

    if [[ -f "$ARCHIVE_PATH" && -s "$ARCHIVE_PATH" ]]; then
        echo "Extracting archive..."
        tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR" 2>/dev/null || true
        # Find the binary
        FOUND_EXE="$(find "$TMP_DIR" -type f -name "$BINARY_NAME" 2>/dev/null | head -1)"
        if [[ -n "$FOUND_EXE" ]]; then
            SOURCE_EXE="$FOUND_EXE"
            echo "Successfully downloaded and extracted binary."
        else
            echo "Warning: Binary not found in downloaded archive."
        fi
    else
        echo "Warning: Failed to download release asset."
    fi

    if [[ -z "$SOURCE_EXE" ]]; then
        echo "Falling back to source compilation."
        FROM_SOURCE=true
    fi

    # Cleanup temp (will be reused if FROM_SOURCE)
    rm -rf "$TMP_DIR" 2>/dev/null || true
fi

if $FROM_SOURCE; then
    echo "Building $SERVER_NAME from source..."
    if ! command -v cargo &>/dev/null; then
        echo "Error: cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi

    cargo build --release
    echo "Source build complete."

    BUILT_EXE="target/release/$BINARY_NAME"
    if [[ ! -f "$BUILT_EXE" ]]; then
        echo "Error: Built binary not found at $BUILT_EXE"
        exit 1
    fi
    SOURCE_EXE="$(cd "$(dirname "$BUILT_EXE")" && pwd)/$(basename "$BUILT_EXE")"
    echo "Using compiled binary at $SOURCE_EXE"
fi

# ── Ensure target directory ──────────────────────────────────────────────────

mkdir -p "$TARGET_DIR"

# ── Install Binary ───────────────────────────────────────────────────────────

INSTALLED_EXE="$STABLE_EXE"
echo "Installing executable to $INSTALLED_EXE..."
cp -f "$SOURCE_EXE" "$INSTALLED_EXE"
chmod +x "$INSTALLED_EXE"
echo "Binary installed."

# ── Run Configuration Installer ──────────────────────────────────────────────

INSTALL_ARGS=("install")
if [[ "$CLIENT" != "all" ]]; then
    INSTALL_ARGS+=("--client" "$CLIENT")
fi

echo "Running configuration installer from $INSTALLED_EXE..."
"$INSTALLED_EXE" "${INSTALL_ARGS[@]}"

echo ""
echo "========================================================"
echo "$SERVER_NAME has been installed successfully!"
echo "Installed Executable Path: $INSTALLED_EXE"
echo "Please restart your agent to reload changes."
echo "========================================================"
