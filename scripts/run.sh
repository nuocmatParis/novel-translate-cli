#!/bin/sh
# Run the current source; .env credentials are read by the app itself.
set -eu
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
command -v go >/dev/null 2>&1 || { echo "Go 1.25.5+ is required to run from source." >&2; exit 1; }
if [ "$#" -eq 0 ]; then
    set -- tui
fi
cd "$REPO_DIR"
GOWORK=off exec go run -trimpath -ldflags="-s -w" ./cmd/novel-translate-cli "$@"
