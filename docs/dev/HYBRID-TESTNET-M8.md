# Hybrid PoW Testnet (Chain ID 5) Runbook & Activation (Milestone M8)

## 1. Overview & Consensus Invariants

Per **ADR 0003** and the locked architectural decisions (Toni 2026-11-05):
- **Chain ID:** `5` (`HYBRID_TESTNET`)
- **Consensus Model:** Hybrid PoW + GHOSTDAG
- **Stake Role:** **NO stake in consensus path** (no committee voting, no VRF leader tickets, no certificate quorums)
- **PoW Algorithm:** **KovanicaHash** (BLAKE3 + 2 MiB memory mix)
- **Block Target Time:** 4 seconds (`TARGET_BLOCK_MS = 4000`)
- **GHOSTDAG Parameter:** $k = 5$ (`GHOSTDAG_K = 5`)
- **Block Subsidy:** 2.5 KVNC (`2_500_000_000` atoms) minted directly to each confirmed block author
- **Difficulty Adjustment:** Asymmetric DAA window (`DAA_WINDOW_BLOCKS = 140`) targeting 4 s block time

---

## 2. Node Configuration & Startup

### Configuration File (`config.chain5.toml`)
Operators can configure a hybrid testnet node using `ops/config.chain5.example.toml`:

```toml
chain_id = 5
data_dir = "./data/chain5"
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
export KOVANICA_CHAIN_ID=5
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
The testnet faucet operates directly with Chain 5:
```bash
kovanica-faucet --chain-id 5 --rpc-url http://127.0.0.1:8545 --port 3000
```

---

## 4. Verification & Health Checklist

1. **RPC Health Check:**
   ```bash
   curl http://127.0.0.1:8545/health
   ```
   Verify `chain_id = 5` and `peer_count >= 1`.

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
- Submits and executes transactions on Chain 5 without any staking dependencies.
