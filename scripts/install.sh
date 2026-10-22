#!/bin/sh
# Build this checkout; no upstream binary or auto-update service is used.
set -eu
command -v go >/dev/null 2>&1 || { echo "Go 1.25.5 or later is required." >&2; exit 1; }
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
DEST_DIR=${NOVEL_TRANSLATE_INSTALL_DIR:-"$HOME/.local/bin"}
mkdir -p "$DEST_DIR"
cd "$REPO_DIR"
GOWORK=off go build -trimpath -o "$DEST_DIR/novel-translate-cli" ./cmd/novel-translate-cli
echo "Installed $DEST_DIR/novel-translate-cli"
echo "Run: \"$DEST_DIR/novel-translate-cli\" tui"
