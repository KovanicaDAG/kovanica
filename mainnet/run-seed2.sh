#!/bin/bash
KOVANICA_ROOT="${KOVANICA_ROOT:-/root/Projects/kovanica}"
export KOVANICA_DATA_DIR="${KOVANICA_ROOT}/mainnet/node-data"
export KOVANICA_LISTEN_ADDR=0.0.0.0:8000
# Note: update KOVANICA_BOOTNODES to mainnet DNS seed names per docs/SEED-DNS-AUDIT.md before mainnet launch.
export KOVANICA_BOOTNODES=seed1.kovanica.online:8000,seed3.kovanica.online:8000
export KOVANICA_VALIDATOR_KEY="${KOVANICA_ROOT}/mainnet/validators/keys-out/validator2.key"
export KOVANICA_MINE=0
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_OPERATOR=0
exec /usr/local/bin/kovanica-node "$@"
