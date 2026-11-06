# Kovanica Docker

## Build the node image

```bash
docker build -t kovanica-node:local .
```

The image contains only the `kovanica-node` binary and a minimal `debian:bookworm-slim`
runtime. It runs as the unprivileged `kovanica` user (uid/gid 10001), stores chain data
under `/var/lib/kovanica` (declared `VOLUME`), and exposes P2P `8000` / JSON-RPC `8545`.
A plain `docker run kovanica-node:local` prints `--help`; it never boots a node by
default.

## Run the local multi-node testnet / devnet

### Step 1: Generate Devnet Validator Keys & Genesis

The compose services mount validator seed keys and a shared genesis file. Before running `docker compose up`, generate these local development keys:

```bash
# Build the CLI tool required to generate addresses and genesis
cargo build --release -p kovanica-cli

# Run the key generation helper script
./ops/docker/validators/generate.sh
```

This creates:
- `ops/docker/validators/val1.pem` through `val5.pem` (32-byte hex seeds, mode 0600)
- `ops/docker/validators/genesis_validators.toml` (shared genesis validator declarations)
- `ops/docker/validators/funded_accounts.toml` (initial account allocations)

*(Note: These files are development keys and are intentionally gitignored).*

### Step 2: Launch the Compose Stack

```bash
docker compose up --build -d     # start stack in background
docker compose ps                # verify health of seed and validators
docker compose logs -f seed      # stream seed node logs
docker compose down              # stop containers, preserve data volumes
docker compose down -v           # stop and wipe all data volumes
```

### Topology (`docker-compose.yml`)

The compose stack runs **1 seed / bootnode + 5 active validators**:

| Service | Container Name | Role | Host RPC Endpoint | P2P Listen |
|---------|----------------|------|-------------------|------------|
| `seed`  | `kovanica-testnet-seed` | Read-only RPC & Bootnode | `127.0.0.1:8545` | `8000` |
| `val1`  | `kovanica-testnet-val1` | Active Validator 1 | `127.0.0.1:8546` | `8000` |
| `val2`  | `kovanica-testnet-val2` | Active Validator 2 | `127.0.0.1:8547` | `8000` |
| `val3`  | `kovanica-testnet-val3` | Active Validator 3 | `127.0.0.1:8548` | `8000` |
| `val4`  | `kovanica-testnet-val4` | Active Validator 4 | `127.0.0.1:8549` | `8000` |
| `val5`  | `kovanica-testnet-val5` | Active Validator 5 | `127.0.0.1:8550` | `8000` |

All six nodes publish their respective JSON-RPC ports to loopback (`127.0.0.1`).

### Interacting with the Stack

Query the seed node health and status:
```bash
curl -fsS http://127.0.0.1:8545/health
curl -s -X POST http://127.0.0.1:8545/rpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"kovanica_blockNumber","params":[]}'
```

Query validator 1:
```bash
curl -fsS http://127.0.0.1:8546/health
```

### Healthcheck

`GET /health` returns HTTP 200 with JSON status:
```json
{"status":"ok","peer_count":3,"is_healthy":true,...}
```
The compose healthchecks monitor `/health` and report when nodes are running and synchronized.

---

## Chain 5 Hybrid PoW Testnet Cluster (`docker-compose.chain5.yml`)

A standalone 1-command Docker Compose testnet cluster for **Chain 5** (`HYBRID_TESTNET`) featuring 3 peered hybrid testnet nodes with active KovanicaHash PoW mining, GHOSTDAG $k=5$ ordering, a dedicated faucet service, and pre-configured Prometheus + Grafana dashboards.

### Topology

| Service | Role | Container Port | Host Port |
|---------|------|----------------|-----------|
| `node1` | Seed / Bootnode & Miner (author 0) | `8545` (RPC), `8000` (P2P) | `127.0.0.1:8545`, `8000`, `127.0.0.1:9100` (metrics) |
| `node2` | Peered Node & Miner (author 1) | `8546` (RPC) | `127.0.0.1:8546` |
| `node3` | Peered Node & Miner (author 2) | `8547` (RPC) | `127.0.0.1:8547` |
| `faucet` | Chain 5 KVNC Dispenser | `3000` | `127.0.0.1:3000` |
| `prometheus` | Prometheus Metrics Scraper | `9090` | `127.0.0.1:9090` |
| `grafana` | Grafana Dashboards | `3000` | `127.0.0.1:3001` (admin/admin) |

### Quick Start

```bash
# Start all 6 services in the background
docker compose -f ops/docker/docker-compose.chain5.yml up -d --build

# Run automated smoke test
./ops/docker/test-chain5-cluster.sh

# View container status
docker compose -f ops/docker/docker-compose.chain5.yml ps

# Follow logs from node1
docker compose -f ops/docker/docker-compose.chain5.yml logs -f node1

# Dispense tokens from faucet
curl -X POST http://127.0.0.1:3000/faucet \
  -H "Content-Type: application/json" \
  -d '{"address": "kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag"}'

# Stop the cluster and wipe volumes
docker compose -f ops/docker/docker-compose.chain5.yml down -v
```

### Observability

- **Prometheus**: Accessible at `http://localhost:9090`. Scrapes `/metrics` from all three nodes every 2s.
- **Grafana**: Accessible at `http://localhost:3001` (credentials `admin` / `admin`). The `Kovanica Hybrid Testnet (Chain 5) Overview` dashboard visualizes:
  - Total PoW Blocks Mined (`pow_blocks_mined_total`) and mining rate
  - GHOSTDAG Blue vs Red blocks breakdown (`ghostdag_blue_blocks_total`, `ghostdag_red_blocks_total`, `ghostdag_tips_count`)
  - Estimated network hashrate (`pow_hashrate_estimate`) and target difficulty (`pow_target_bits`)
  - Connected peers per node (`peer_count`)
  - Mempool size (`mempool_size`) and block commit latency (`commit_latency_ms`)
