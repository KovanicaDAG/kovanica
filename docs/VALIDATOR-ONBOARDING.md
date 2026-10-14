# KVNC Testnet Validator Onboarding Guide

This guide walks you through becoming a validator on the **KVNC testnet** (chain ID: `2`).

---

## 1. Prerequisites

| Requirement | Specification |
|-------------|---------------|
| **Hardware** | 2–4 GB RAM VPS, 2 vCPU, 50 GB SSD |
| **OS** | Ubuntu 22.04+ / Debian 12+ (systemd) |
| **Network** | Public IPv4, port `8000` (TCP) open for P2P |
| **Software** | Docker 24+ **or** Rust 1.90+ toolchain |
| **Keys** | Ed25519 validator key + VRF key pair (generated below) |

> **Note**: Testnet uses chain ID `2`. Local devnet uses `1337`. Mainnet will use `1`.

---

## 2. Key Generation

### 2.1 Validator Identity Key (Ed25519)

```bash
# Using kvnc-cli (recommended)
cargo install --path crates/kvnc-cli
kvnc-cli key generate --output validator.keystore

# Or manually with openssl
openssl genpkey -algorithm ed25519 -out validator.key
# Convert to KVNC keystore format if needed
```

The keystore file is encrypted with a passphrase. **Store the passphrase securely.**

### 2.2 VRF Key Pair (for leader selection)

```bash
# Using kvnc-cli (VRF key generation included in key generate)
kvnc-cli key generate --output validator.keystore --with-vrf

# The VRF public key will be embedded in the keystore
# You can also extract it:
kvnc-cli key inspect --keystore validator.keystore
```

> **Why VRF?** KVNC uses stake-weighted VRF leader selection (ECVRF-ED25519-SHA512-Elligator2 per RFC 9381). The VRF public key must be registered at genesis.

---

## 3. Genesis Participation

### 3.1 Submit Your Keys

Before the genesis ceremony, submit to the testnet coordinators:

1. **Validator Ed25519 public key** (from keystore)
2. **VRF public key** (from keystore)
3. **P2P address** (your public IP:8000)
4. **Desired stake amount** (≥ 50,000 KUNA)

### 3.2 Genesis Ceremony

Coordinators will:
1. Collect all validator keys
2. Generate `genesis.json` with staking state
3. Distribute `genesis.json` + `genesis_validators.toml`

You will receive:
- `genesis.json` — genesis block + initial staking state
- `genesis_validators.toml` — validator set with VRF keys

---

## 4. Node Configuration

### 4.1 Copy Example Config

```bash
cp config.example.toml kvnc.toml
```

### 4.2 Edit `kvnc.toml` for Validator

```toml
# Network
listen_addr = "0.0.0.0:8000"          # P2P port (must be publicly reachable)
rpc_addr = "127.0.0.1:8545"           # JSON-RPC (bind to localhost only!)
bootnodes = [
    "seed1.kovanica.online:8000",           # testnet seed 1
    "seed2.kovanica.online:8000",              # testnet seed 2
    "seed3.kovanica.online:8000"               # testnet seed 3
]

# Validator
run_validator = true                   # Enable block production
validator_key = "/path/to/validator.keystore"
validator_key_pass = "your-passphrase" # Or use KVNC_VALIDATOR_KEY_PASS env var

# Data directory
data_dir = "/var/lib/kvnc"

# Optional: enable faucet (only for seed nodes)
# faucet = false
```

### 4.3 Environment Variables (Alternative to Config File)

```bash
export KVNC_LISTEN=0.0.0.0:8000
export KVNC_RPC=127.0.0.1:8545
export KVNC_PEERS="seed1.kovanica.online:8000,seed2.kovanica.online:8000,seed3.kovanica.online:8000"
export KVNC_MINE=0                      # 1 only if you run a seed node
export KVNC_FAUCET=0
export KVNC_OPERATOR=0
export KVNC_ALLOW_RESET=0
export KVNC_DATA="/var/lib/kvnc"
export KVNC_VALIDATOR_KEY="/path/to/validator.keystore"
export KVNC_VALIDATOR_KEY_PASS="your-passphrase"
```

---

## 5. Staking

### 5.1 Minimum Stake

| Parameter | Value |
|-----------|-------|
| **Min stake** | 50,000 KUNA (50,000,000,000,000 atoms) |
| **Active validators** | 15–21 |
| **Epoch** | 7,200 committed leader blocks (~1 day) |
| **Unbonding** | 1 epoch (configurable) |

### 5.2 Stake via CLI

```bash
# Get testnet KUNA from faucet first
curl -X POST https://faucet.testnet.kvnc.io/faucet \
  -H "Content-Type: application/json" \
  -d '{"address": "kvnc<your-address>dag"}'

# Stake
kvnc-cli stake \
  --keystore /path/to/validator.keystore \
  --amount 50000000000000 \
  --chain-id 2 \
  --rpc-url http://127.0.0.1:8545
```

### 5.3 Delegate (Optional)

```bash
# Delegate to another validator
kvnc-cli delegate \
  --keystore /path/to/delegator.keystore \
  --validator <validator-address> \
  --amount 10000000000000 \
  --chain-id 2
```

---

## 6. Running the Node

### Option A: Docker Compose (Recommended)

```yaml
# docker-compose.yml
version: '3.8'
services:
  kvnc:
    image: ghcr.io/kovanica/kvnc-node:latest
    container_name: kvnc-validator
    restart: unless-stopped
    ports:
      - "8000:8000"          # P2P
      - "127.0.0.1:8545:8545" # RPC (localhost only)
    volumes:
      - ./kvnc.toml:/kvnc.toml:ro
      - ./validator.keystore:/validator.keystore:ro
      - kvnc-data:/var/lib/kvnc
    environment:
      - KVNC_VALIDATOR_KEY=/validator.keystore
      - KVNC_VALIDATOR_KEY_PASS=${VALIDATOR_PASS}
    deploy:
      resources:
        limits:
          memory: 3G
          cpus: '2.0'

volumes:
  kvnc-data:
```

```bash
export VALIDATOR_PASS="your-passphrase"
docker compose up -d
```

### Option B: Systemd Service

```ini
# /etc/systemd/system/kvnc.service
[Unit]
Description=KVNC Validator Node
After=network.target

[Service]
Type=simple
User=kvnc
WorkingDirectory=/opt/kvnc
ExecStart=/opt/kvnc/kvnc-node --config /opt/kvnc/kvnc.toml
Restart=on-failure
RestartSec=10
LimitNOFILE=65536
Environment=KVNC_VALIDATOR_KEY=/opt/kvnc/validator.keystore
Environment=KVNC_VALIDATOR_KEY_PASS=your-passphrase

# Resource limits (cgroup v2)
MemoryMax=3G
CPUQuota=200%
IOWeight=100

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now kvnc
```

### Option C: Binary Directly

```bash
# Build from source
cargo build --release -p kvnc-node

# Run
./target/release/kvnc-node --config kvnc.toml
```

---

## 7. Monitoring

### 7.1 Health Check

```bash
curl http://127.0.0.1:8545/health
# {"status":"ok","height":12345,"peers":3,"round":456}
```

### 7.2 Prometheus Metrics

```bash
curl http://127.0.0.1:8545/metrics
# Exposes: consensus_height, mempool_size, peer_count, block_commit_latency, etc.
```

### 7.3 Grafana Dashboard

Import `ops/monitoring/grafana-dashboard.json` for:
- Consensus height & round
- Mempool size & transaction throughput
- Peer count & sync status
- Block commit latency
- Memory / CPU usage

### 7.4 Key Metrics to Watch

| Metric | Healthy Range | Alert If |
|--------|---------------|----------|
| `consensus_height` | Increasing | Stalled > 5 min |
| `peer_count` | ≥ 3 | < 3 for > 10 min |
| `mempool_size` | < 10,000 | > 50,000 |
| `block_commit_latency` | < 20s | > 60s |
| `rss_memory_mb` | < 2000 | > 2800 (3G limit) |

---

## 8. Testnet Specifics

| Parameter | Value |
|-----------|-------|
| **Chain ID** | `2` |
| **RPC endpoint** | `http://<your-ip>:8545` (local) / `https://rpc.testnet.kvnc.io` (public) |
| **Explorer** | `https://explorer.testnet.kvnc.io` |
| **Faucet** | `https://faucet.testnet.kvnc.io` (3 req/hr/IP, 10 KUNA) |
| **Seed nodes** | `seed1.kovanica.online:8000`, `seed2.kovanica.online:8000`, `seed3.kovanica.online:8000` |
| **Genesis file** | Distributed by coordinators |

### 8.1 Get Test KUNA

```bash
# Via CLI
kvnc-cli faucet --address kvnc<your-address>dag --chain-id 2

# Via HTTP
curl -X POST https://faucet.testnet.kvnc.io/faucet \
  -H "Content-Type: application/json" \
  -d '{"address": "kvnc<your-address>dag"}'
```

---

## 9. Troubleshooting

### 9.1 Peers Not Connecting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `peer_count = 0` | Port 8000 blocked | Open TCP 8000 in firewall/security group |
| `peer_count = 1` | Only one seed reachable | Check all 3 seed IPs; verify `bootnodes` config |
| `dial error` | IPv6 mismatch | Ensure `listen_addr = "0.0.0.0:8000"` (not `::`) |

### 9.2 Sync Stuck

| Symptom | Cause | Fix |
|---------|-------|-----|
| Height not increasing | Non-blue blocks, fork | Check logs for `non_blue` / `fork` messages; restart node |
| `round` not advancing | No quorum | Verify ≥ 3 peers; check seed nodes are up |
| `mempool_size` growing | Transactions not committing | Check if you're leader; verify stake ≥ 50K KUNA |

### 9.3 RPC Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `connection refused` | RPC not bound | Check `rpc_addr = "127.0.0.1:8545"` and firewall |
| `chain_id mismatch` | Wrong chain ID | Use `--chain-id 2` for testnet |
| `nonce too low` | Stale local state | Restart node to resync |

### 9.4 Validator Not Producing Blocks

| Check | Command |
|-------|---------|
| Am I in active set? | `curl -s http://127.0.0.1:8545/rpc -d '{"method":"kvnc_getValidators"}'` |
| Is my stake ≥ 50K? | `kvnc-cli balance --address <your-address>` |
| Is VRF key registered? | `kvnc-cli key inspect --keystore validator.keystore` |
| Is node leader this round? | Check `round` in `/health` vs your validator index |

---

## 10. Useful Commands

```bash
# Check node status
curl -s http://127.0.0.1:8545/health | jq .

# Check validator set
curl -s -X POST http://127.0.0.1:8545/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"kvnc_getValidators","params":[],"id":1}' | jq .

# Check own balance
kvnc-cli balance --address kvnc<your-address>dag --rpc-url http://127.0.0.1:8545

# Check staking info
kvnc-cli staking-info --address kvnc<your-address>dag --rpc-url http://127.0.0.1:8545

# View logs (docker)
docker logs -f kvnc-validator

# View logs (systemd)
journalctl -u kvnc -f
```

---

## 11. Support

- **Testnet Discord**: `#kvnc-testnet-validators`
- **GitHub Issues**: `github.com/kovanica/kvnc/issues`
- **Docs**: `https://docs.kvnc.io`

---

*Last updated: 2026-10-14 for testnet chain ID 2*