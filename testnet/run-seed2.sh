#!/bin/bash
export KVNC_DATA_DIR=/root/Projects/kvnc/testnet/node-data
export KVNC_LISTEN_ADDR=0.0.0.0:8000
export KVNC_BOOTNODES=seed1.kovanica.online:8000,seed2.kovanica.online:8000,seed3.kovanica.online:8000
export KVNC_VALIDATOR_KEY=/root/Projects/kvnc/testnet/validators/v2.dat
export KVNC_MINE=0
export KVNC_FAUCET=0
export KVNC_ALLOW_RESET=0
export KVNC_OPERATOR=0
exec /usr/local/bin/kvnc-node "$@"
