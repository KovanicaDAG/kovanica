#!/bin/bash
KOVANICA_ROOT="${KOVANICA_ROOT:-/root/Projects/kovanica}"
export KOVANICA_DATA_DIR="${KOVANICA_ROOT}/testnet/node-data"
export KOVANICA_LISTEN_ADDR=0.0.0.0:8000
export KOVANICA_BOOTNODES=seed1.kovanica.online:8000,seed2.kovanica.online:8000,seed3.kovanica.online:8000
export KOVANICA_VALIDATOR_KEY="${KOVANICA_ROOT}/testnet/validators/v2.dat"
export KOVANICA_MINE=0
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_OPERATOR=0
exec /usr/local/bin/kovanica-node "$@"
