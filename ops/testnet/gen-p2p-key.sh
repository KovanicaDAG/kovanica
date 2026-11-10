#!/usr/bin/env bash
# Generate a random 32-byte hex P2P key once per seed host (never commit it).
# Usage: gen-p2p-key.sh /etc/kovanica/seedN.p2p.key
set -euo pipefail
out="${1:?usage: $0 <path>}"
[ -e "$out" ] && { echo "refusing to overwrite $out"; exit 1; }
umask 077
openssl rand -hex 32 > "$out"
echo "wrote $out (0600)"
