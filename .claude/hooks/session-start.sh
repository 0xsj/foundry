#!/usr/bin/env bash
# Prints Foundry state the moment the repo opens, so no command is needed to
# find out where you are. Builds the CLI on first run and caches the binary.
set -euo pipefail

root="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$root"

bin="$root/foundry"
if [[ ! -x "$bin" || "$bin" -ot "$root/cmd/foundry" ]]; then
  go build -o "$bin" ./cmd/foundry 2>/dev/null || {
    echo "foundry: CLI failed to build — run 'go build -o foundry ./cmd/foundry' to see why"
    exit 0
  }
fi

"$bin" status 2>/dev/null || echo "foundry: no state yet — say 'go' to start module 00"
