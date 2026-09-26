#!/usr/bin/env bash
# Developing environment - local dev node (PoA)
# Use: source /root/kovanica/deploy/developing/env.sh
# Runs: kovanica-node explorer 127.0.0.1:8080

# Consensus: PoA (testnet default)
export KOVANICA_CONSENSUS=poa
export KOVANICA_AUTHORITIES=4a4172c14e6073998caf9ad256974cd2908a67b7751fdbc2f031c24736b8e8ec,8ebc8a73235b631845d32ed4ea2d1dc563acfa1215b18428b17364c6e1563cf3,d6903aa7a17abfe681988f1b49a8adcec0d475c5e24ab955c1349a1c463bedae
export KOVANICA_AUTHORITY_THRESHOLD=2
export KOVANICA_SLOT_DURATION=3000

# Node config
export KOVANICA_LISTEN=127.0.0.1:9000
export KOVANICA_PEERS=seed.kovanica.online:9000
export KOVANICA_FAUCET=0
export KOVANICA_OPERATOR=0
export KOVANICA_DATA=/root/kovanica/deploy/developing/data

# Dev-only: allow genesis reset (isolated host only)
# export KOVANICA_ALLOW_RESET=1
# export KOVANICA_ISOLATED_HOST=1

# Block production (optional, for local testing)
# export KOVANICA_PRODUCE=1
# export KOVANICA_PRODUCE_SECS=3
# Authority key loaded from separate file if producing:
# source /root/kovanica/deploy/testnet/authority-keys/authority-1.env
