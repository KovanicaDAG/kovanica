#!/usr/bin/env bash
# Genesis ceremony script for Kovanica testnet/mainnet
# Usage: ./ops/genesis/ceremony.sh --network mainnet [OPTIONS]
set -euo pipefail

NETWORK="testnet"
VALIDATORS_FILE=""
OUTPUT_DIR="./genesis"
TREASURY_ADDRESS=""
FOUNDER_ADDRESS=""
CHAIN_ID=""
FORCE=false
GENERATE_KEYS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --network) NETWORK="$2"; shift 2 ;;
        --validators) VALIDATORS_FILE="$2"; shift 2 ;;
        --output) OUTPUT_DIR="$2"; shift 2 ;;
        --treasury-address) TREASURY_ADDRESS="$2"; shift 2 ;;
        --founder-address) FOUNDER_ADDRESS="$2"; shift 2 ;;
        --chain-id) CHAIN_ID="$2"; shift 2 ;;
        --force) FORCE=true; shift ;;
        --generate-keys) GENERATE_KEYS=true; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

if [[ -z "$CHAIN_ID" ]]; then
    case "$NETWORK" in
        mainnet) CHAIN_ID=1 ;;
        testnet) CHAIN_ID=2 ;;
        devnet) CHAIN_ID=3 ;;
        testnet-v2|testnet_v2) CHAIN_ID=4 ;;
        hybrid-testnet|hybrid_testnet) CHAIN_ID=5 ;;
        local) CHAIN_ID=1337 ;;
        *) CHAIN_ID=1 ;;
    esac
fi

if [[ -z "$VALIDATORS_FILE" ]]; then
    if [[ -f "$NETWORK/validators/validators.json" ]]; then
        VALIDATORS_FILE="$NETWORK/validators/validators.json"
    elif [[ -f "mainnet/validators/validators.json" && "$NETWORK" == "mainnet" ]]; then
        VALIDATORS_FILE="mainnet/validators/validators.json"
    elif [[ -f "testnet/validators/validators.json" && "$NETWORK" == "testnet" ]]; then
        VALIDATORS_FILE="testnet/validators/validators.json"
    elif [[ -f "./validators.json" ]]; then
        VALIDATORS_FILE="./validators.json"
    else
        echo "ERROR: --validators <file.json> required or could not find validators.json for network $NETWORK"
        exit 1
    fi
fi

if [[ ! -f "$VALIDATORS_FILE" ]]; then
    echo "ERROR: Validators file not found: $VALIDATORS_FILE"
    exit 1
fi

if [[ -z "$TREASURY_ADDRESS" ]]; then
    TREASURY_ADDRESS="kvncqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqgw9aq4rdag"
fi

if [[ -z "$FOUNDER_ADDRESS" ]]; then
    FOUNDER_ADDRESS="kvncqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsp3cjmndag"
fi

mkdir -p "$OUTPUT_DIR"

echo "=== Kovanica Genesis Ceremony ==="
echo "Network: $NETWORK (Chain ID: $CHAIN_ID)"
echo "Validators: $VALIDATORS_FILE"
echo "Treasury: $TREASURY_ADDRESS"
echo "Founder: $FOUNDER_ADDRESS"
echo "Output: $OUTPUT_DIR"
echo ""

# Find binary: prefer ./target/release/kovanica-node if built, otherwise cargo run
if [[ -x "./target/release/kovanica-node" ]]; then
    NODE_BIN="./target/release/kovanica-node"
elif [[ -x "./target/debug/kovanica-node" ]]; then
    NODE_BIN="./target/debug/kovanica-node"
else
    NODE_BIN="cargo run --release -p kovanica-node --"
fi

# Step 1: Generate genesis block + state
echo "[1/4] Generating genesis..."
GENESIS_ARGS="--chain-id $CHAIN_ID --validators $VALIDATORS_FILE --treasury-address $TREASURY_ADDRESS --founder-address $FOUNDER_ADDRESS --output $OUTPUT_DIR/genesis.json"
if $FORCE; then
    GENESIS_ARGS="$GENESIS_ARGS --force"
fi
if $GENERATE_KEYS; then
    GENESIS_ARGS="$GENESIS_ARGS --validator-keys-out $OUTPUT_DIR/keys"
fi

$NODE_BIN genesis $GENESIS_ARGS

# Step 2: Key generation or verification
if $GENERATE_KEYS; then
    echo "[2/4] Generated validator keys in $OUTPUT_DIR/keys"
else
    echo "[2/4] Validator keys preserved from input ceremony configuration"
fi

# Step 3: Create genesis_validators.toml
echo "[3/4] Creating genesis_validators.toml..."
python3 -c "
import json
with open('$VALIDATORS_FILE') as f:
    v_in = json.load(f)
with open('$OUTPUT_DIR/genesis_validators.toml', 'w') as f:
    f.write(f'# Kovanica $NETWORK genesis validators\n# Network: $NETWORK (chain_id = $CHAIN_ID)\nchain_id = $CHAIN_ID\n\n')
    for v in v_in:
        f.write('[[validator]]\n')
        f.write(f'address = \"{v[\"address\"]}\"\n')
        f.write(f'stake = {v.get(\"stake\", 50000000000000)}\n')
        if 'public_key' in v:
            f.write(f'public_key = \"{v[\"public_key\"]}\"\n')
        f.write('\n')
"

# Step 4: Create bootnode config
echo "[4/4] Creating bootnode config..."
if [[ "$NETWORK" == "mainnet" ]]; then
    SEED="mainnet.kovanica.online:8000"
else
    SEED="testnet.kovanica.online:8000"
fi

cat > "$OUTPUT_DIR/bootnodes.toml" << EOF
# Kovanica $NETWORK bootnodes
# All nodes should use this seed
bootnodes = ["$SEED"]
EOF

echo ""
echo "=== Genesis Ceremony Complete ==="
echo "Files generated:"
echo "  $OUTPUT_DIR/genesis.json"
echo "  $OUTPUT_DIR/genesis_validators.toml"
echo "  $OUTPUT_DIR/bootnodes.toml"
if $GENERATE_KEYS; then
    echo "  $OUTPUT_DIR/keys/ (validator keys)"
fi
echo ""
echo "Next steps:"
echo "1. Distribute genesis.json and genesis_validators.toml to bootstrap seed nodes"
echo "2. Each node places files in their KOVANICA_DATA_DIR"
echo "3. Configure bootnodes = [\"$SEED\"]"
echo "4. Launch nodes with target block time 4 s and GHOSTDAG K=5"
