#!/bin/bash
export KVNC_DATA_DIR=/root/Projects/kvnc/mainnet/node-data
export KVNC_LISTEN_ADDR=0.0.0.0:8000
export KVNC_BOOTNODES=145.223.116.178:8000,76.13.250.65:8000
export KVNC_VALIDATOR_KEY=/root/Projects/kvnc/mainnet/validators/keys-out/validator3.key
export KVNC_MINE=0
export KVNC_FAUCET=0
export KVNC_ALLOW_RESET=0
export KVNC_OPERATOR=0
exec /usr/local/bin/kvnc-node "$@"
