# Kovanica Mainnet Launch & Operations Runbook

> **Network ID:** `1` (`MAINNET`)  
> **Native Ticker:** **KVNC** (9 decimals: $1\text{ KVNC} = 10^9\text{ atoms}$)  
> **Canonical Address Format:** PRO-21 base32 (`kvnc<57 base32>dag`, exactly 64 characters)  
> **Status:** DRAFT, not normative, being rewritten for PoW + GHOSTDAG no-stake  
> **Consensus Impact:** Client and Operational only; strictly conforms to frozen consensus and tokenomics specifications.

---

## 1. Overview and Architecture Principles

This runbook defines the end-to-end operational protocol for launching, validating, monitoring, and maintaining the **Kovanica Mainnet** (Chain ID `1`). 

### Core Architectural Targets
- **Consensus Engine:** Hybrid PoW + GHOSTDAG (target block duration 4 s (`TARGET_BLOCK_MS = 4000`), GHOSTDAG parameter $K=5$, PoW Hash: KovanicaHash; NO stake in consensus path; legacy Mysticeti-C wave length `WAVE_LENGTH = 3` and 12 s leader commitment remain preserved in codebase).
- **Block Timing:** Target block time `4,000 ms` (4 s), with GHOSTDAG $K=5$ topological transaction ordering.
- **Resource Footprint:** Standard commodity hardware and VPS instances: 2–4 GB RAM, 2 vCPU cores, 50 GB NVMe SSD.
- **P2P Networking:** Plaintext TCP on port **8000** over `libp2p` with Noise encryption and Yamux multiplexing.
- **RPC & Metrics:** JSON-RPC / WebSocket on ports **8545** / **8546**, Prometheus metrics on port **9100**.

---

## 2. Phase 0: Pre-Launch Verification & Invariants

Prior to deploying any genesis artifacts or coordinating node startups, the operations team and validator operators must independently verify all frozen protocol parameters.

### 2.1 Protocol & Tokenomics Parameter Matrix

Verify that the binary under execution enforces the frozen parameters defined in `crates/kovanica-params` and `crates/kovanica-staking`:

| Parameter | Mainnet Invariant | Code Source / Check |
|---|---|---|
| **Chain ID** | `1` (`MAINNET`) | `crates/kovanica-params/src/lib.rs` |
| **Total Supply Ceiling** | `90,200,000 KVNC` | `TOTAL_SUPPLY = 90_200_000 * 10^9` |
| **Decimals** | `9` (1 atom = $10^{-9}$ KVNC) | `ONE_KVNC = 1_000_000_000` |
| **Founder Premine** | `200,000 KVNC` | `FOUNDER_PREMINE = 200_000 * 10^9` |
| **Treasury Allocation** | `8,000,000 KVNC` | Linear 8-year vesting (1M KVNC/year) |
| **Mining Subsidy Budget** | `82,000,000 KVNC` | $90.2\text{M} - 0.2\text{M} - 8\text{M}$ |
| **Initial Block Reward** | `2.5 KVNC` | $2.5 \times 10^9$ atoms per committed block |
| **Subsidy Era Length** | `8,100,000 blocks` | Multiplied by $3/4$ decay each era |
| **Genesis Bootstrap Seeds** | `1` to `20` nodes | `3–5` seeds recommended for public launch |
| **Epoch Duration** | `7,200` rounds (~8 hours) | `EPOCH_ROUNDS = 7_200` |
| **Unbonding Cooldown** | `100,000` leader blocks | ~13.9 days at 12 s/leader |

### 2.2 Mainnet Pre-Flight Configuration Guards

Mainnet nodes strictly enforce security configurations at startup. The node binary **fails immediately** if any of the following constraints are violated:

1. **`funded_accounts.toml` is Forbidden:**
   On Chain ID `1`, `funded_accounts.toml` must **NOT** exist in `KOVANICA_DATA_DIR`. All genesis funds must originate strictly from the founder premine or treasury vesting.
2. **Address Format Enforcement:**
   Only canonical PRO-21 base32 addresses (`kvnc` + 57 base32 chars + `dag`) are permitted. Hexadecimal or legacy addresses trigger parse rejection.
3. **No Reset Switch / No Public Faucet:**
   ```bash
   KOVANICA_ALLOW_RESET=0
   KOVANICA_FAUCET=0
   KOVANICA_MINE=0
   KOVANICA_OPERATOR=0
   ```
4. **Configuration Check Command:**
   Each operator must execute pre-flight validation:
   ```bash
   kovanica-node --check-config --config /etc/kovanica/config.toml
   ```

---

## 3. Phase 1: Seed Infrastructure & DNS Configuration

A resilient P2P mesh requires at least 3 geographically independent DNS seed nodes acting as initial entry points.

### 3.1 DNS Architecture (Cloudflare DNS-Only / Grey-Cloud)

P2P communication runs over plaintext TCP on port `8000`. **Cloudflare HTTP proxy (orange-cloud) must be disabled** for all seed records.

| DNS Record | Type | Target IP / Host | Cloudflare Status | Purpose |
|---|---|---|---|---|
| `mainnet.kovanica.online` | A | `<Seed-1-IP>` | **DNS-only (grey-cloud)** | Primary mainnet seed |
| `seed1.kovanica.online` | A | `<Seed-1-IP>` (EU Central) | **DNS-only (grey-cloud)** | Independent Seed 1 |
| `seed2.kovanica.online` | A | `<Seed-2-IP>` (US East) | **DNS-only (grey-cloud)** | Independent Seed 2 |
| `seed3.kovanica.online` | A | `<Seed-3-IP>` (AP Southeast) | **DNS-only (grey-cloud)** | Independent Seed 3 |
| `rpc.kovanica.online` | A | `<RPC-LoadBalancer-IP>` | **DNS-only (grey-cloud)** | Public JSON-RPC (port 8545) |

### 3.2 Verifying Seed Reachability

Run network diagnostics across all seed targets:

```bash
# Verify DNS resolution
dig +short seed1.kovanica.online
dig +short seed2.kovanica.online
dig +short seed3.kovanica.online
dig +short mainnet.kovanica.online

# Verify TCP 8000 ingress
nc -zvw3 seed1.kovanica.online 8000
nc -zvw3 seed2.kovanica.online 8000
nc -zvw3 seed3.kovanica.online 8000
```

### 3.3 Bootstrap Peer Multiaddr Formatting

Each seed node generates a persistent libp2p identity key (`p2p.key`). The resulting Multiaddrs populate `KOVANICA_BOOTNODES`:

```text
/dns4/seed1.kovanica.online/tcp/8000/p2p/12D3KooW...,/dns4/seed2.kovanica.online/tcp/8000/p2p/12D3KooW...,/dns4/seed3.kovanica.online/tcp/8000/p2p/12D3KooW...
```

---

## 4. Phase 2: Genesis Validator Coordination & Bootstrap

Mainnet genesis requires simultaneous multi-validator consensus setup governed by `ops/genesis/ceremony.sh` and [`docs/KEY-CEREMONY.md`](KEY-CEREMONY.md).

### 4.1 Committee Size & Allocation

TODO: rewrite for PoW + GHOSTDAG

### 4.2 Genesis Generation Procedure

The ceremony coordinator collects verified `validators.json` and executes:

```bash
./ops/genesis/ceremony.sh \
    --network mainnet \
    --validators validators.json \
    --treasury-address kvnc1...dag \
    --founder-address kvnc1...dag \
    --output ./mainnet-genesis
```

The resulting bundle produces:
- `genesis.json`: Core state containing founder balance, treasury vesting vault, and initial validator accounts.
- `genesis_validators.toml`: Validator committee definitions, Ed25519 addresses, and Ristretto255 VRF public keys.
- `bootnodes.toml`: Canonical seed bootstrap entries.

### 4.3 Genesis Bundle Checksum & Network Tag Verification

Every validator operator must independently compute and compare the SHA-256 checksums:

```bash
cd /var/lib/kovanica
sha256sum genesis.json genesis_validators.toml bootnodes.toml > SHA256SUMS.local
sha256sum -c SHA256SUMS
```

All nodes derive a **Network Tag**:
$$\text{Network Tag} = \text{chain\_id} - \text{digest}_{16}(\text{genesis\_config})$$
For Mainnet, the tag is formatted as `1-<16 hex chars>`. Nodes with mismatched network tags are rejected by the libp2p handshake.

### 4.4 Synchronized Launch ($T_0$)

1. **Target Time:** Agree on a precise UTC timestamp $T_0$ (e.g. `2026-11-25 12:00:00 UTC`).
2. **Seed Startup ($T_0 - 15\text{m}$):** Boot seeds 1, 2, and 3 with clean data directories.
3. **Node & Miner Startup ($T_0$):** Bootstrap seed nodes and mining nodes start `kovanica-node` simultaneously via systemd:
   ```bash
   systemctl start kovanica-node
   journalctl -u kovanica-node -f
   ```

---

## 5. Phase 3: Launch Verification & Health Checks

Immediately upon launch, run the automated verification suite.

### 5.1 CLI Node Operations Inspection

Using `crates/kovanica-cli`:

```bash
# Check sync state and peer convergence
kovanica node sync --rpc http://127.0.0.1:8545

# Inspect connected P2P peers
kovanica node peers --rpc http://127.0.0.1:8545

# JSON formatted checks for automation scripts
kovanica node sync --json --rpc http://127.0.0.1:8545
kovanica node peers --json --rpc http://127.0.0.1:8545
```

Expected Sync Output:
```text
+-------------------+--------------------------------------+
| Field             | Value                                |
+-------------------+--------------------------------------+
| Sync State        | in sync                              |
| Current Height    | 42                                   |
| Round             | 126                                  |
| Target Height     | 42                                   |
| Blocks Behind     | 0                                    |
| Connected Peers   | 18                                   |
+-------------------+--------------------------------------+
```

### 5.2 JSON-RPC Cluster Health Probe

Run `ops/monitoring/check-cluster.sh` or query directly:

```bash
# Node health endpoint
curl -fsS http://127.0.0.1:8545/health | jq .

# Check current block height
curl -fsS -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"kovanica_blockNumber","params":[],"id":1}' | jq .

# Verify consensus sync status
curl -fsS -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"kovanica_syncStatus","params":[],"id":1}' | jq .
```

Verify that:
1. `peer_count` $\ge 14$ (for a 15–20 node committee).
2. `seconds_since_last_commit` stays $< 20\text{ s}$ (nominal 12 s).
3. `blockNumber` advances monotonically every 12 seconds.

---

## 6. Phase 4: Prometheus & Grafana Monitoring & Alert Runbook

Nodes expose Prometheus metrics on port `9100` (or configured metrics port). Observability rules are defined in `ops/prometheus/alerts.yml`.

### 6.1 Critical Alerts & Remediation Procedures

#### Alert: `KovanicaPeerCountDrop` / `KovanicaPeerCountCritical`
- **Condition:** `peer_count < 3` for 2 minutes (Drop) or `peer_count == 0` for 1 minute (Critical).
- **Severity:** Critical.
- **Triage Steps:**
  1. Check firewall and external connectivity: `curl -I https://1.1.1.1`.
  2. Verify TCP port 8000 is open and listening: `ss -tulpn | grep 8000`.
  3. Inspect node log for connection drops or protocol mismatches: `journalctl -u kovanica-node | grep -E "disconnect|swarm|identify"`.
  4. Ensure `KOVANICA_BOOTNODES` are reachable and responsive.

#### Alert: `KovanicaBlockHeightStalled`
- **Condition:** `increase(block_height[5m]) == 0` for 5 minutes.
- **Severity:** Critical (Consensus stall).
- **Triage Steps:**
  1. Determine if stall is local or global across all validators:
     ```bash
     for s in seed1 seed2 seed3; do
       curl -s https://$s.kovanica.online/health | jq .block_height
     done
     ```
  2. If local: restart the local node (`systemctl restart kovanica-node`). Check storage errors in `consensus.redb`.
  3. If global: TODO: rewrite for PoW + GHOSTDAG

#### Alert: `KovanicaBlockHeightDrift`
- **Condition:** `max(block_height) - min(block_height) > 10` across cluster for 2 minutes.
- **Severity:** Warning.
- **Triage Steps:**
  1. Identify lagging nodes via `kovanica node sync`.
  2. Verify whether lagging node is rate-limited on P2P gossip or experiencing high disk I/O wait (`iostat -xz 1`).
  3. Trigger catch-up sync or adjust `KOVANICA_MAX_PEERS`.

#### Alert: `KovanicaCommitLatencyHigh`
- **Condition:** `commit_latency > 5000` ms for 3 minutes.
- **Severity:** Warning.
- **Triage Steps:**
  1. High DAG propagation delay. Check network ping latencies between validators.
  2. Check CPU utilization: ensure Wasmi contract execution or cryptographic verification is not throttled.

#### Alert: `KovanicaMemoryHigh` / `KovanicaMemoryCritical`
- **Condition:** RSS proxy $> 3000\text{ MB}$ (Warning) or $> 3500\text{ MB}$ (Critical, OOM danger).
- **Severity:** Critical.
- **Triage Steps:**
  1. Check memory consumer: `ps aux --sort=-%mem | head -n 5`.
  2. Inspect state cache and mempool size (`mempool_size`).
  3. If mempool is flooded with spam, ensure fee priority ordering is working and adjust transaction eviction thresholds.
  4. If required, perform a rolling graceful restart to release RSS.

#### Alert: `KVCNNodeDown`
- **Condition:** `up{job="kovanica"} == 0` for 1 minute.
- **Severity:** Critical.
- **Triage Steps:**
  1. Check systemd service status: `systemctl status kovanica-node`.
  2. Inspect exit code or panic backtrace in `/var/log/kovanica/node.log`.
  3. Verify disk space: `df -h /var/lib/kovanica`. If disk is full, expand volume immediately.

---

## 7. Phase 5: Incident Response & Emergency Procedures

### 7.1 Equivocation Triage

TODO: rewrite for PoW + GHOSTDAG

### 7.2 Network Partition Recovery

If a network split occurs:
1. **Never reset or wipe single nodes** — this risks unrecoverable fork splits.
2. Verify all validators have intact `consensus.redb` and `state.redb` storage.
3. Restore inter-partition network connectivity (BGP routing / cloud provider interconnect).
4. TODO: rewrite for PoW + GHOSTDAG

### 7.3 Disaster Recovery / State Restoration

In the event of catastrophic data corruption on a single validator:
1. Stop the node: `systemctl stop kovanica-node`.
2. Delete the corrupted database:
   ```bash
   rm -rf /var/lib/kovanica/data/*.redb
   ```
3. Verify `genesis.json`, `genesis_validators.toml`, and keystore are untouched.
4. Restart the node. The node will perform full state catch-up sync from peer nodes via `kovanica_syncStatus` and block sync protocol `/kovanica/<tag>/block-sync/1.1.0`.

---

## 8. Post-Launch Operational Checklist

- [ ] Bootstrap seed nodes online and discovery mesh established.
- [ ] Block height steadily increasing with 4 s target block interval (Hybrid PoW + GHOSTDAG).
- [ ] Network tag matching across all nodes (`1-<16 hex chars>`).
- [ ] Zero unhandled errors in Prometheus alerts (`ops/prometheus/alerts.yml`).
- [ ] JSON-RPC rate limits active and CORS headers restricted on public endpoints.
- [ ] Explorer (`explorer.kovanica.online`) indexing blocks and transactions.
- [ ] Off-site daily snapshot backups enabled for validator and seed data directories.
