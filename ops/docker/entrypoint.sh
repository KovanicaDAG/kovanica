#!/usr/bin/env sh
#
# Kovanica node container entrypoint.
#
#   1. provision a persistent validator seed inside the data volume (unless the
#      operator already points KOVANICA_VALIDATOR_KEY at their own file), and
#   2. hand control to the node binary or requested command.
#
# Validator seeds are never baked into the image or the repository: they are
# generated on first boot and live only in the mounted data volume.

set -eu

# A node only starts when explicitly asked to (the image default is --help).
case "${1:-}" in
    --help|-h|--version|-V)
        exec kovanica-node "$@"
        ;;
    kovanica-faucet|kovanica|sh|bash)
        exec "$@"
        ;;
esac

DATA_DIR="${KOVANICA_DATA_DIR:-/var/lib/kovanica}"
KEY_FILE="${KOVANICA_VALIDATOR_KEY:-$DATA_DIR/validator.key}"

install -d -m 700 "$DATA_DIR"

if [ -n "${KOVANICA_VALIDATOR_KEY:-}" ] && [ -f "$KOVANICA_VALIDATOR_KEY" ] && [ "$KOVANICA_VALIDATOR_KEY" != "$DATA_DIR/validator.key" ]; then
    # Operator provided external validator key file; copy to volume with strict 0600 permissions
    cat "$KOVANICA_VALIDATOR_KEY" > "$DATA_DIR/validator.key"
    KEY_FILE="$DATA_DIR/validator.key"
    chmod 600 "$KEY_FILE"
elif [ ! -s "$KEY_FILE" ]; then
    umask 077
    # 32 bytes -> 64 hex chars; the node reads a 32-byte hex seed.
    head -c 32 /dev/urandom | od -An -v -tx1 | tr -d ' \n' > "$KEY_FILE"
    printf '\n' >> "$KEY_FILE"
    chmod 600 "$KEY_FILE"
    echo "entrypoint: generated validator seed at $KEY_FILE"
else
    chmod 600 "$KEY_FILE" 2>/dev/null || true
fi

# Let the node pick up the seed file (its own key never leaves the volume).
KOVANICA_VALIDATOR_KEY="$KEY_FILE"
export KOVANICA_VALIDATOR_KEY

# Provision persistent P2P identity if provided via KOVANICA_P2P_KEY or existing file
P2P_KEY_FILE="$DATA_DIR/p2p_node.key"
if [ -n "${KOVANICA_P2P_KEY:-}" ]; then
    if [ -f "$KOVANICA_P2P_KEY" ]; then
        cat "$KOVANICA_P2P_KEY" > "$P2P_KEY_FILE"
    else
        printf '%s\n' "$KOVANICA_P2P_KEY" > "$P2P_KEY_FILE"
    fi
    chmod 600 "$P2P_KEY_FILE"
    echo "entrypoint: configured persistent p2p node key at $P2P_KEY_FILE"
elif [ -f "$P2P_KEY_FILE" ]; then
    chmod 600 "$P2P_KEY_FILE" 2>/dev/null || true
fi

exec kovanica-node "$@"
