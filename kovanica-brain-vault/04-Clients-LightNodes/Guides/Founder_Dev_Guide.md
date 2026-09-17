# Kovanica: Founder & Developer Guide (Expanded)

*The authoritative master document for orchestrating the Kovanica ecosystem.*

## 1. Architectural Overview & Consensus

Kovanica is not a traditional blockchain; it is a high-performance **BlockDAG (Directed Acyclic Graph)**. It achieves high throughput and fast finality by allowing parallel block creation rather than forcing a single sequential chain.

### The Hybrid Consensus Model
Kovanica utilizes a dual-engine consensus mechanism:
1. **GHOSTDAG (Chain-Selection):** This is the underlying protocol that organizes the DAG. It uses a "Blue Work" scoring system to cryptographically determine the heaviest, most secure topological ordering of blocks. GHOSTDAG decides *what wins* and ensures network agreement.
2. **VRF-Staked Sortition (Block Admission):** To decentralize block production away from purely industrial mining farms, Kovanica implements an Algorand/Praos-style cryptographic lottery. Mobile devices and light nodes sign a Verifiable Random Function (VRF) against the current DAG tip. If the output satisfies a difficulty threshold scaled by their staked KOV, they win the right to produce the next block. This decides *who may add*.
3. **PoW Fallback:** Classical Blake3 Proof-of-Work remains continuously active as a fallback and security anchor, ensuring that even if staked nodes drop offline, the network keeps ticking.

## 2. Infrastructure Footprint & Topography

As the founder, your infrastructure spans several distinct domains that must be monitored and maintained:

### A. Seed Nodes & Network Backbone
- **Purpose:** Provide initial peer discovery and maintain the canonical DAG state.
- **Current Deployment:** e.g., `seed3` on AWS EC2 (`eu-north-1`).
- **Management:** Deployed via systemd (`kovanica-seed3`). Always ensure `KOVANICA_POW` is appropriately toggled for off-box testing vs production.
- **Monitoring:** Watch `systemctl status kovanica-seed3` and the standard output logs for sync status, DAG tip advancement, and memory utilization.

### B. Mining Infrastructure (`/root/kovanica-hash/`)
To support the widest array of hardware, Kovanica runs a dual-pool system:
- **Native Pool (`go-pool`):** A modern, high-performance Go-based Stratum pool. It aggregates native Blake3 hashrate from CPU and GPU miners. It runs as `kovanica-go-pool` under PM2, offering extremely low latency and low memory footprint.
- **ASIC Proxy (`proxy/`):** A Python-based Stratum proxy (`proxy.py`). Since SHA-256 and Scrypt ASICs cannot natively hash Blake3, this proxy acts as a "Smart Pool." It routes alien hashrate to public BTC/DOGE pools, generating revenue for the Kovanica Treasury. A background script (`payout.py`) intercepts the submitted shares, calculates the equivalent KOV value, and issues RPC commands to the node daemon to pay the miners.

### C. Core Codebases
- **`kovanica-protocol` (Rust/Go):** The core node implementation, GHOSTDAG logic, and P2P networking stack.
- **`kovanica-web`:** The frontend explorer and web wallet UI.
- **`kovanica-hash`:** The mining infrastructure mentioned above.

## 3. Maintenance, Operations & Alerting

- **Prometheus/Grafana:** The `alerting_rules.yml` file contains the baseline for network health. Critical alerts include DAG tip stalling, excessive fork resolution times, and node memory leaks.
- **Vault Syncing:** The Obsidian vault is your brain. Treat `/root/kovanica-protocol/` as the authoritative source of truth for protocol documentation. Use `scripts/sync-vault.sh` regularly to pull changes from the codebase into the vault.
- **Treasury Management for the Proxy:** The node wallet associated with the `payout.py` script must remain funded. Monitor the inflow of BTC/DOGE from the public pools and periodically liquidate or balance it against the KOV emitted to the foreign ASIC miners.

## 4. Long-Term Vision & Development Cycles

- **Incentive Compatibility:** Every protocol upgrade must pass strict game-theory auditing. There must be *no profitable deviation* from honest behavior (e.g., selfish mining, block withholding, or equivocation must remain strictly less profitable than honest participation).
- **Mobile-First Decentralization:** Continuously optimize the mobile light node codebase. The easier it is for a user to toggle "Stake" on their phone without draining their battery, the more resilient the network becomes to 51% attacks.
- **Smart Contracts (Future):** As the DAG stabilizes, architecting a parallel state execution environment that does not bottleneck the underlying GHOSTDAG topology will be the next major frontier.
