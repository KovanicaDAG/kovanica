#!/usr/bin/env bash
# Kill one validator process by PID file or name (local/devnet only).
set -euo pipefail
PID="${1:-}"
if [ -z "$PID" ]; then
  echo "usage: $0 <pid>" >&2
  exit 2
fi
kill -9 "$PID"
echo "killed $PID"
