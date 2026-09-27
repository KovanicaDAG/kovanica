# Technical Mainnet Prep Package

## 1. Reproducible Build Dockerfile

```dockerfile
# Dockerfile.reproducible
# Build kovanica-node and kovanica-cli reproducibly
# Usage: docker build -t kovanica-node:mainnet-genesis-rc1 -f Dockerfile.reproducible .

FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set fixed build environment
ENV CARGO_TERM_COLOR=never
ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-Wl,--build-id=none"
ENV SOURCE_DATE_EPOCH=1729891200  # 2026-10-25 00:00:00 UTC

WORKDIR /src

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build with locked dependencies
RUN cargo build --release --locked -p kovanica-node -p kovanica-cli

# Verify binary hashes
RUN sha256sum target/release/kovanica-node > /output/kovanica-node.sha256 \
 && sha256sum target/release/kovanica-cli > /output/kovanica-cli.sha256

# Minimal runtime image
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/kovanica-node /usr/local/bin/
COPY --from=builder /src/target/release/kovanica-cli /usr/local/bin/
COPY --from=builder /output/*.sha256 /verification/

ENTRYPOINT ["kovanica-node"]
```

### Build & Verify
```bash
# Build
docker build -t kovanica-node:mainnet-genesis-rc1 -f Dockerfile.reproducible .

# Extract binaries + hashes
docker create --name extract kovanica-node:mainnet-genesis-rc1
docker cp extract:/usr/local/bin/kovanica-node ./kovanica-node
docker cp extract:/usr/local/bin/kovanica-cli ./kovanica-cli
docker cp extract:/verification/kovanica-node.sha256 ./kovanica-node.sha256
docker cp extract:/verification/kovanica-cli.sha256 ./kovanica-cli.sha256
docker rm extract

# Verify
sha256sum -c kovanica-node.sha256
sha256sum -c kovanica-cli.sha256

# Publish to GitHub Releases + sigstore
gh release create mainnet-genesis-rc1 \
  kovanica-node kovanica-cli \
  kovanica-node.sha256 kovanica-cli.sha256 \
  --title "Mainnet Genesis RC1" \
  --notes "Reproducible build for mainnet genesis ceremony"
```

---

## 2. Binary Hash Verification Script

```bash
#!/usr/bin/env bash
# verify_binaries.sh
# Verify downloaded binaries match published hashes

set -euo pipefail

HASHES_URL="https://github.com/KovanicaDAG/kovanica/releases/download/mainnet-genesis-rc1"
BINARIES=("kovanica-node" "kovanica-cli")

echo "=== Kovanica Mainnet Genesis RC1 Binary Verification ==="
echo ""

for bin in "${BINARIES[@]}"; do
    echo "Verifying $bin..."
    
    # Download hash
    curl -sL "${HASHES_URL}/${bin}.sha256" -o "${bin}.sha256"
    
    # Download binary
    curl -sL "${HASHES_URL}/${bin}" -o "${bin}"
    chmod +x "${bin}"
    
    # Verify
    if sha256sum -c "${bin}.sha256"; then
        echo "✅ $bin: VERIFIED"
    else
        echo "❌ $bin: HASH MISMATCH!"
        exit 1
    fi
    echo ""
done

echo "=== All binaries verified successfully ==="
echo ""
echo "Run: ./kovanica-node --version"
echo "Run: ./kovanica-cli --version"
```

---

## 3. Mainnet Genesis Dry-Run on Testnet Fork

### Dry-Run Script
```bash
#!/usr/bin/env bash
# dry_run_genesis.sh
# Run full genesis ceremony on testnet fork

set -euo pipefail

FORK_HEIGHT=1000  # Fork from testnet at this height
DRY_RUN_DIR="/tmp/kovanica-dry-run-$(date +%s)"

echo "=== Kovanica Mainnet Genesis Dry-Run ==="
echo "Fork height: $FORK_HEIGHT"
echo "Work dir: $DRY_RUN_DIR"
echo ""

mkdir -p "$DRY_RUN_DIR"
cd "$DRY_RUN_DIR"

# 1. Generate test authority keys
echo "[1/7] Generating test authority keys..."
for i in {1..7}; do
    cargo run --release --example generate_authority_keys -- \
      --out-dir "$DRY_RUN_DIR/keys/validator-$i" \
      --count 1 --threshold 1 --slot-duration 120000
done

# 2. Generate genesis signer keys
echo "[2/7] Generating genesis signer keys..."
for i in {1..7}; do
    kovanica-cli keygen --offline --output "$DRY_RUN_DIR/keys/signer-$i.key"
    kovanica-cli pubkey --input "$DRY_RUN_DIR/keys/signer-$i.key" \
      --output "$DRY_RUN_DIR/keys/signer-$i.pub"
done

# 3. Build authority set
echo "[3/7] Building authority set..."
cat "$DRY_RUN_DIR/keys/validator-*/authorities.conf" | \
  grep KOVANICA_AUTHORITIES | cut -d= -f2 | tr ',' '\n' > "$DRY_RUN_DIR/validator-pubkeys.txt"

# 4. Create genesis transaction
echo "[4/7] Creating genesis transaction..."
cat > genesis-tx.json << EOF
{
  "network": "kovanica-dry-run",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "outputs": [
    {"amount": "2000000000000000", "script": "<founder-pubkey> OP_CHECKSIG", "timelock": 525600},
    {"amount": "1000000000000000", "script": "<vault-1-script>", "vault_id": 1},
    {"amount": "1000000000000000", "script": "<vault-2-script>", "vault_id": 2},
    {"amount": "1000000000000000", "script": "<vault-3-script>", "vault_id": 3},
    {"amount": "1000000000000000", "script": "<vault-4-script>", "vault_id": 4},
    {"amount": "1000000000000000", "script": "<vault-5-script>", "vault_id": 5},
    {"amount": "1000000000000000", "script": "<vault-6-script>", "vault_id": 6},
    {"amount": "1000000000000000", "script": "<vault-7-script>", "vault_id": 7},
    {"amount": "1000000000000000", "script": "<vault-8-script>", "vault_id": 8},
    {"amount": "1000000000000000", "script": "<vault-9-script>", "vault_id": 9},
    {"amount": "1000000000000000", "script": "<vault-10-script>", "vault_id": 10}
  ],
  "multisig": "dry-run-multisig-hash"
}
EOF

# 4. Sign genesis (simulate 5-of-7)
echo "[5/7] Signing genesis transaction (5-of-7)..."
for i in {1..5}; do
    kovanica-cli sign --multisig \
      --input genesis-tx.json \
      --key "$DRY_RUN_DIR/keys/signer-$i.key" \
      --output "genesis-sig-$i.sig"
done

# Combine signatures
kovanica-cli multisig-combine --threshold 5 --signatures genesis-sig-*.sig --output genesis-tx-signed.json

# 5. Start seed nodes
echo "[6/7] Starting seed nodes..."
for i in {1..3}; do
    PORT=$((9000 + i))
    HTTP_PORT=$((8080 + i))
    DATA_DIR="$DRY_RUN_DIR/seed$i"
    
    mkdir -p "$DATA_DIR"
    
    # Create genesis with PoA
    kovanica-cli genesis_poa 3 1000000000 2000000000000000 1 100 120000 \
      $(cat "$DRY_RUN_DIR/validator-pubkeys.txt" | tr '\n' ' ') \
      --data-dir "$DATA_DIR"
    
    # Start node
    kovanica-node --data-dir "$DATA_DIR" --p2p-port "$PORT" --http-port "$HTTP_PORT" &
    SEED_PIDS+=($!)
done

sleep 10

# 6. Start validators
echo "[7/7] Starting validators..."
for i in {1..7}; do
    PORT=$((9010 + i))
    HTTP_PORT=$((8090 + i))
    DATA_DIR="$DRY_RUN_DIR/validator$i"
    
    mkdir -p "$DATA_DIR"
    cp "$DRY_RUN_DIR/keys/validator-$i/authority-1.env" "$DATA_DIR/"
    
    kovanica-node --data-dir "$DATA_DIR" --p2p-port "$PORT" --http-port "$HTTP_PORT" \
      --peers "127.0.0.1:9001,127.0.0.1:9002,127.0.0.1:9003" &
    VAL_PIDS+=($!)
done

sleep 30

# 8. Verify
echo "=== Verification ==="
for i in {1..3}; do
    HTTP_PORT=$((8080 + i))
    curl -s "http://127.0.0.1:$HTTP_PORT/api/head" | jq '{height: .blocks, tip: .tip, genesis: .genesis}'
done

# Check all seeds have same genesis
GENESIS_1=$(curl -s http://127.0.0.1:8081/api/head | jq -r .genesis)
GENESIS_2=$(curl -s http://127.0.0.1:8082/api/head | jq -r .genesis)
GENESIS_3=$(curl -s http://127.0.0.1:8083/api/head | jq -r .genesis)

if [[ "$GENESIS_1" == "$GENESIS_2" && "$GENESIS_2" == "$GENESIS_3" ]]; then
    echo "✅ Genesis hashes match across all 3 seeds: $GENESIS_1"
else
    echo "❌ Genesis mismatch!"
    exit 1
fi

# Check block production
sleep 300  # 5 minutes = 2.5 blocks at 120s
for i in {1..3}; do
    HTTP_PORT=$((8080 + i))
    HEIGHT=$(curl -s "http://127.0.0.1:$HTTP_PORT/api/head" | jq -r .blocks)
    echo "Seed $i height: $HEIGHT"
done

# Cleanup
for pid in "${SEED_PIDS[@]}" "${VAL_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
done

echo ""
echo "=== Dry-run complete ==="
echo "Work dir preserved at: $DRY_RUN_DIR"
```

---

## 4. Verification Checklist (Post-Dry-Run)

| Check | Command | Expected |
|-------|---------|----------|
| Genesis hash identical | `curl /api/head \| jq .genesis` | All 3 seeds identical |
| Supply accounting | `curl /api/head \| jq .supply` | 90.2M KVNC cap |
| Coinbase maturity | `kovanica-cli check-maturity --height 0` | 100 blocks |
| Fee burn | `curl /api/head \| jq .fee_burn_ratio` | 0.75 |
| Treasury vaults | `kovanica-cli vault list` | 10 vaults, 1M each |
| Founder timelock | `kovanica-cli output inspect <txid>` | CSV=525600 |
| Block production | Monitor 1h | ~30 blocks/h (120s) |
| No orphan blocks | Monitor 1h | 0 orphans |
| P2P connectivity | `curl /api/head \| jq .peers` | 3/3 connected |

---

## 5. Go/No-Go Criteria

| Criterion | Pass | Fail |
|-----------|------|------|
| Genesis hash match | ✅ | ❌ Abort |
| Supply ≤ 90.2M KVNC | ✅ | ❌ Abort |
| Fee burn = 75% | ✅ | ❌ Abort |
| Coinbase maturity = 100 | ✅ | ❌ Abort |
| 10 treasury vaults | ✅ | ❌ Abort |
| Founder CSV = 525600 | ✅ | ❌ Abort |
| Block production rate | ~30/hr | <20/hr or >40/hr |
| Orphan rate | 0% | >1% |
| Seed sync | All 3 | <3 |

---

## 6. Rollback Procedure

```bash
#!/usr/bin/env bash
# rollback.sh - Emergency rollback

echo "=== EMERGENCY ROLLBACK ==="

# 1. Stop all nodes
for pid in $(pgrep -f kovanica-node); do
    kill -9 "$pid"
done

# 2. Destroy data directories
rm -rf /var/lib/kovanica-mainnet /root/kovanica-data /var/lib/kovanica-seed*

# 3. Rotate all authority keys
# (Generate new keys per AUTHORITY_KEY_CEREMONY_KIT.md)

# 4. Restart from Phase 1 with new keys
echo "Rollback complete. Ready for new ceremony."
```