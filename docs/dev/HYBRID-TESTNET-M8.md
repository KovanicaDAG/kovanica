# Hybrid PoW Testnet (Chain ID 2) Runbook & Activation (Milestone M8)

## 1. Overview & Consensus Invariants

Per **ADR 0003** and the locked architectural decisions (Toni 2026-11-05):
- **Chain ID:** `2` (`TESTNET`, legacy alias `HYBRID_TESTNET`)
- **Consensus Model:** Hybrid PoW + GHOSTDAG
- **Stake Role:** **NO stake in consensus path** (no committee voting, no VRF leader tickets, no certificate quorums)
- **PoW Algorithm:** **KovanicaHash** (BLAKE3 + 2 MiB memory mix)
- **Block Target Time:** 4 seconds (`TARGET_BLOCK_MS = 4000`)
- **GHOSTDAG Parameter:** $k = 5$ (`GHOSTDAG_K = 5`)
- **Block Subsidy:** 2.5 KVNC (`2_500_000_000` atoms) minted directly to each confirmed block author
- **Difficulty Adjustment:** Asymmetric DAA window (`DAA_WINDOW_BLOCKS = 140`) targeting 4 s block time

---

## 2. Node Configuration & Startup

### Configuration File (`config.chain2.toml`)
Operators can configure a hybrid testnet node using `ops/config.chain2.example.toml`:

```toml
chain_id = 2
data_dir = "./data/chain2"
listen_addr = "0.0.0.0:8000"
rpc_addr = "127.0.0.1"
rpc_port = 8545

[mining]
enabled = true
threads = 2
author = 0
```

### Environment Variable Overrides
Alternatively, configure via standard environment variables:
```bash
export KOVANICA_CHAIN_ID=2
export KOVANICA_MINING_ENABLED=true
export KOVANICA_MINING_THREADS=2
export KOVANICA_LISTEN_ADDR="0.0.0.0:8000"
export KOVANICA_RPC_ADDR="127.0.0.1"
export KOVANICA_RPC_PORT=8545
kovanica-node
```

---

## 3. Tooling & CLI Integration

### Mining Status
Inspect mining parameters, current target, difficulty, and network hashrate estimate:
```bash
kovanica mining status --rpc-url http://127.0.0.1:8545
```

### DAG Tips
Query the active DAG tips and current GHOSTDAG $k$ parameter:
```bash
kovanica dag tips --rpc-url http://127.0.0.1:8545
```

### Faucet Dispensing
The testnet faucet operates directly with Chain 2:
```bash
kovanica-faucet --chain-id 2 --rpc-url http://127.0.0.1:8545 --port 3000
```

---

## 4. Verification & Health Checklist

1. **RPC Health Check:**
   ```bash
   curl http://127.0.0.1:8545/health
   ```
   Verify `chain_id = 2` and `peer_count >= 1`.

2. **Mining Info Endpoint:**
   ```bash
   curl -X POST http://127.0.0.1:8545/rpc -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"kovanica_getMiningInfo","params":[],"id":1}'
   ```
   Ensure `mining_enabled: true`, `ghostdag_k: 5`, and `target_block_time_ms: 4000`.

3. **Block Advancement & Rewards:**
   ```bash
   curl -X POST http://127.0.0.1:8545/rpc -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"kovanica_blockNumber","params":[],"id":1}'
   ```
   Observe the committed block height increasing as PoW blocks are mined and blue sets are sequenced.

---

## 5. Automated E2E Cluster Test

The full live 3-node cluster test is automated in `crates/kovanica-node/tests/hybrid_node_cluster_integration.rs`:
- Spawns 3 independent `kovanica-node` instances connected via P2P.
- Mines blocks with KovanicaHash and sequences them through GHOSTDAG $k=5$.
- Verifies that 2.5 KVNC mining rewards are minted to block authors.
- Submits and executes transactions on Chain 2 without any staking dependencies.

---

## 6. Docker Compose Multi-Node Testnet Cluster & Observability Suite

For local multi-node evaluation, benchmarking, and real-time observability, a 1-command Docker Compose stack is provided in `ops/docker/docker-compose.chain2.yml`.

### Stack Topology

| Service | Container Name | Role | Host Endpoints |
|---------|----------------|------|----------------|
| `node1` | `kovanica-chain2-node1` | Bootnode & Miner (author 0) | RPC `127.0.0.1:8545`<br>P2P `8000`<br>Metrics `127.0.0.1:9100` |
| `node2` | `kovanica-chain2-node2` | Peered Node & Miner (author 1) | RPC `127.0.0.1:8546` |
| `node3` | `kovanica-chain2-node3` | Peered Node & Miner (author 2) | RPC `127.0.0.1:8547` |
| `faucet` | `kovanica-chain2-faucet` | Chain 2 Token Dispenser | HTTP `127.0.0.1:3000` |
| `prometheus` | `kovanica-chain2-prometheus` | Metrics Scraper (node1, node2, node3) | HTTP `127.0.0.1:9090` |
| `grafana` | `kovanica-chain2-grafana` | Pre-provisioned Dashboards | HTTP `127.0.0.1:3001` (admin/admin) |

### Starting the Cluster

```bash
docker compose -f ops/docker/docker-compose.chain2.yml up -d --build
```

Verify that all containers report healthy:
```bash
docker compose -f ops/docker/docker-compose.chain2.yml ps
```

### Automated Smoke Test

Run the full end-to-end verification script:
```bash
./ops/docker/test-chain2-cluster.sh
```

The script verifies:
1. Node healthchecks (`/health` on ports 8545, 8546, 8547).
2. Faucet healthcheck (`/health` on port 3000).
3. Live PoW block advancement via `kovanica_blockNumber`.
4. GHOSTDAG blue block sequencing and metrics exportation on port 9100.
5. Faucet KVNC dispensation on Chain 2.
6. Prometheus and Grafana health.

### Real-Time Grafana Dashboard

Navigate to `http://localhost:3001` (credentials `admin` / `admin`). The dashboard `Kovanica Hybrid Testnet (Chain 2) Overview` is automatically provisioned and displays:
- **Total PoW Blocks Mined (`pow_blocks_mined_total`) & Mining Rate**: Real-time rate of PoW blocks found across the cluster.
- **GHOSTDAG Blue vs Red Blocks Breakdown (`ghostdag_blue_blocks_total`, `ghostdag_red_blocks_total`)**: Tracking DAG colouring, sub-DAG ordering, and active DAG tips (`ghostdag_tips_count`).
- **Network Hashrate & Target Difficulty**: Estimated hashrate in H/s and current DAA target bits over time.
- **Connected Peer Count (`peer_count`)**: P2P mesh connectivity.
- **Mempool Size & Block Commit Latency**: Transaction queue depth and sub-DAG commit latency in ms.

### Stopping the Cluster

```bash
docker compose -f ops/docker/docker-compose.chain2.yml down -v
```

