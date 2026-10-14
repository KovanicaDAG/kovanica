#!/bin/bash
export KVNC_DATA_DIR=/root/Projects/kvnc/mainnet/node-data
export KVNC_LISTEN_ADDR=0.0.0.0:8000
export KVNC_BOOTNODES=MAINNET_SEED1_IP:8000,MAINNET_SEED2_IP:8000,MAINNET_SEED3_IP:8000
export KVNC_VALIDATOR_KEY=/root/Projects/kvnc/mainnet/validators/v3.dat
export KVNC_MINE=0
export KVNC_FAUCET=0
export KVNC_ALLOW_RESET=0
export KVNC_OPERATOR=0
exec /usr/local/bin/kvnc-node "$@"
