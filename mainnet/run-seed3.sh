#!/bin/bash
export KOVANICA_DATA_DIR=/root/Projects/kvnc/mainnet/node-data
export KOVANICA_LISTEN_ADDR=0.0.0.0:8000
export KOVANICA_BOOTNODES=145.223.116.178:8000,76.13.250.65:8000
export KOVANICA_VALIDATOR_KEY=/root/Projects/kvnc/mainnet/validators/keys-out/validator3.key
export KOVANICA_MINE=0
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_OPERATOR=0
exec /usr/local/bin/kovanica-node "$@"
