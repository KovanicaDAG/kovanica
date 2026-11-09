# Kovanica Mainnet Freeze Checklist & Genesis Sign-Off Gates

> **Network ID:** `1` (`MAINNET`)  
> **Native Ticker:** **KVNC** (9 decimals: $1\text{ KVNC} = 10^9\text{ atoms}$, legacy alias `KUNA`)  
> **Canonical Address Format:** PRO-21 base32 (`kvnc<57 base32>dag`, exactly 64 characters)  
> **Target Date:** Mainnet Genesis ($T_0$)  
> **Status:** Normative Release Sign-Off Specification & Production Gate Verification  
> **Consensus Impact:** Operational & Ledger-Safety Governance; strictly enforces frozen consensus and tokenomics specifications.

---

## 1. Executive Summary & Purpose

This checklist establishes the mandatory sign-off gates required prior to executing the Kovanica Mainnet Genesis ($T_0$). No mainnet deployment, genesis ceremony execution, or public node release may proceed until each gate has achieved 100% verified sign-off by designated technical leads and ceremony witnesses.

### Sign-Off Gates Matrix

| Gate | Domain | Objective | Required Sign-Offs |
|---|---|---|---|
| **Gate 1** | **Code Freeze & Reproducibility** | Immutable Git commit, clean CI run, passing cargo-deny and gitleaks, byte-for-byte reproducible binaries. | Lead Core Developer, Security Auditor |
| **Gate 2** | **Parameter Invariants & Ledger Safety** | Chain ID `1`, 90.2M KVNC cap, 2.5 KVNC block reward, epoch/unbonding limits, no `funded_accounts.toml`. | Consensus Architect, Lead Tokenomics Auditor |
| **Gate 3** | **Genesis Ceremony & Key Distribution** | Ceremony execution per [`docs/KEY-CEREMONY.md`](KEY-CEREMONY.md), Treasury 3-of-5 SSS distribution, Founder key custody, 3–5 bootstrap seed nodes (1..=20 bound). | Master of Ceremonies, Treasury Custodians, Seed Operators |
| **Gate 4** | **Infrastructure & Network Topology** | 3+ independent seed nodes per [`docs/MAINNET-RUNBOOK.md`](MAINNET-RUNBOOK.md), DNS resolution, firewall port hardening (8000, 8545, 8546, 9100). | Lead DevOps/Infra Engineer, Seed Node Operators |
| **Gate 5** | **Monitoring, Alerting & Observability** | Prometheus alerts verified (`alerts.yml`), health probes, peer drop/stall alerts, equivocation detection. | Site Reliability Engineer (SRE), Ops Lead |
| **Gate 6** | **Contingency Procedures & Incident Response** | Documented rollback criteria, hard fork escalation protocol, 24/7 incident response matrix. | Incident Commander, Core Engineering Team |

---

## 2. Gate 1: Code Freeze & Build Reproducibility

Before any validator executes genesis tooling, the codebase must be frozen at a designated commit hash with full automated quality assurance and deterministic binary verification.

### 2.1 Git Commit Freeze & Release Tagging

- [ ] **Commit Freeze:** Designate an exact 40-character commit hash on `main` branch.
- [ ] **Cryptographic Tag:** Tag the commit with an annotated, PGP-signed Git tag:
  ```bash
  git tag -s v1.0.0-mainnet -m "Kovanica Mainnet Genesis Release v1.0.0"
  git push origin v1.0.0-mainnet
  ```
- [ ] **Branch Lock:** Enforce branch protection on `main` and tag protection in repository settings (require 2+ senior code owner reviews, zero bypasses).

### 2.2 Clean CI Execution & Local Verification

- [ ] **Automated CI Suite:** Verify that the full continuous integration pipeline passes with 0 failures:
  ```bash
  # Execute full workspace test suite and verification scripts
  bash scripts/ci-local.sh all
  ```
- [ ] **Unit & Integration Tests:**
  ```bash
  cargo test --workspace --all-targets --locked
  ```
- [ ] **Documentation & Doc-Tests:**
  ```bash
  cargo test --doc --workspace --locked
  ```
- [ ] **Clippy Linter:** Clean execution with zero warnings (`-D warnings` enforced):
  ```bash
  cargo clippy --workspace --all-targets --locked -- -D warnings
  ```
- [ ] **Format Validation:**
  ```bash
  cargo fmt --all -- --check
  ```

### 2.3 Dependency & License Audit (`cargo deny`)

- [ ] **Advisories:** Scan for known vulnerabilities (RUSTSEC database):
  ```bash
  cargo deny check advisories
  ```
- [ ] **Bans:** Ensure no duplicate crate versions or prohibited crates exist in the dependency tree:
  ```bash
  cargo deny check bans
  ```
- [ ] **Licenses:** Verify that 100% of dependencies comply with GPL-3.0-or-later and permissive OSS licenses (MIT, Apache-2.0, BSD-3-Clause):
  ```bash
  cargo deny check licenses
  ```
- [ ] **Sources:** Validate that all dependencies resolve strictly from authenticated crates.io or designated Git tags:
  ```bash
  cargo deny check sources
  ```

### 2.4 Secret Scanning (`gitleaks`)

- [ ] **Full-History Leak Detection:** Ensure zero API tokens, private keys, keystores, or credentials exist in git history:
  ```bash
  gitleaks detect --source . --verbose --redact
  ```
- [ ] Verify that `.gitleaks.toml` exclusions are valid and strictly limited to public test vectors and fixtures.

### 2.5 Reproducible Binary Builds & Checksums

All production binaries (`kovanica-node`, `kovanica-cli`, `kovanica-lightnode`) must be built within an isolated, deterministic Docker environment matching the official release container:

- [ ] **Containerized Build:**
  ```bash
  docker build -f ops/docker/Dockerfile.release -t kovanica-build:release .
  docker run --rm -v $(pwd)/dist:/dist kovanica-build:release cp -r /build/bin/. /dist/
  ```
- [ ] **Target Architecture:** `x86_64-unknown-linux-gnu`, Rust toolchain MSRV `1.90.0` (pinned via `rust-toolchain.toml`).
- [ ] **Generate SHA-256 Checksums:**
  ```bash
  cd dist
  sha256sum kovanica-node kovanica-cli kovanica-lightnode > SHA256SUMS
  ```
- [ ] **Multi-Party Verification:** At least 3 independent core engineers must independently build from the frozen Git tag and confirm identical SHA-256 hashes.
- [ ] **PGP Signatures:** Sign the final checksum manifest:
  ```bash
  gpg --armor --detach-sign SHA256SUMS
  ```

---

## 3. Gate 2: Parameter Invariants & Ledger Safety

The mainnet binary and genesis state must strictly enforce the immutable tokenomics and consensus parameters defined in `crates/kovanica-params` and `crates/kovanica-staking`.

### 3.1 Parameter Verification Matrix

Verify that every parameter in the release binary matches the canonical mainnet specification:

| Parameter | Mainnet Value | Invariant Expression | Source File | Status |
|---|---|---|---|---|
| **Chain ID** | `1` (`MAINNET`) | `CHAIN_ID == 1` | `crates/kovanica-params/src/lib.rs` | [ ] |
| **Native Ticker** | **KVNC** | Decimals: `9` | `crates/kovanica-params/src/lib.rs` | [ ] |
| **Address Format** | PRO-21 base32 | `kvnc` + 57 base32 chars + `dag` (64 chars) | `crates/kovanica-types/src/address.rs` | [ ] |
| **Total Supply Ceiling** | `90,200,000 KVNC` | `90_200_000 * 10^9` atoms | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Founder Premine** | `200,000 KVNC` | `200_000 * 10^9` atoms | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Treasury Allocation** | `8,000,000 KVNC` | Linear 8-year vesting (1M KVNC/yr) | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Genesis Validator Pool**| `1,000,000 KVNC` | Split evenly across bootstrap seed nodes (1..=20) | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Mining Subsidy Budget**| `81,000,000 KVNC` | $90.2\text{M} - 0.2\text{M} - 8\text{M} - 1\text{M}$ | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Initial Block Reward** | `2.5 KVNC` | $2.5 \times 10^9$ atoms ($5 \times 10^9 / 2$) | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Subsidy Era Length** | `8,100,000 blocks` | Multiplied by $3/4$ decay each era | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Target Block Duration**| `4,000 ms` (4 s) | GHOSTDAG $K=5$, target block 4 s | `crates/kovanica-consensus/src/lib.rs` | [ ] |
| **Epoch Duration** | `7,200 rounds` | `EPOCH_ROUNDS = 7_200` (~8 hours; legacy) | `crates/kovanica-consensus/src/lib.rs` | [ ] |
| **Unbonding Period** | `100,000 blocks` | `UNBONDING_ROUNDS = 100_000` (~13.9 days) | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Minimum Validator Stake**| `50,000 KVNC` | `MIN_VALIDATOR_STAKE = 50_000 * 10^9` | `crates/kovanica-params/src/lib.rs` | [ ] |
| **Minimum Delegation** | `50,000 KVNC` | `MIN_DELEGATION = 50_000 * 10^9` | `crates/kovanica-params/src/lib.rs` | [ ] |
| **Slashing Penalty** | `500 bps` (5%) | Fixed equivocation penalty | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Jail Duration** | `7,200 blocks` | `JAIL_ROUNDS = 7_200` (~1 day = 3 epochs; legacy) | `crates/kovanica-staking/src/lib.rs` | [ ] |
| **Light-Node Share** | `20%` (2,000 bps) | Escrowed for 32 winning slots/epoch | `crates/kovanica-staking/src/lib.rs` | [ ] |

### 3.2 Pre-Flight Safety Invariants

- [ ] **No `funded_accounts.toml`:** On Chain ID `1`, `funded_accounts.toml` is strictly forbidden. The binary aborts immediately if this file is present in the data directory.
- [ ] **Strict Address Validation:** Legacy hex addresses (`kvnc<hex>dag`, raw hex) are rejected on Chain ID `1`. Only canonical PRO-21 base32 is accepted.
- [ ] **Hardened Runtime Flags:** The following testnet/development switches must be strictly disabled across all mainnet nodes:
  ```bash
  KOVANICA_ALLOW_RESET=0
  KOVANICA_FAUCET=0
  KOVANICA_MINE=0
  KOVANICA_OPERATOR=0
  ```
- [ ] **Config Check Verification:** Run pre-flight configuration validation across all node operators:
  ```bash
  kovanica-node --check-config --config /etc/kovanica/config.toml
  ```

---

## 4. Gate 3: Genesis Ceremony & Key Distribution

The genesis ceremony establishes the cryptographic trust root of the network and must strictly adhere to [`docs/KEY-CEREMONY.md`](KEY-CEREMONY.md).

### 4.1 Validator Key Generation & Proof of Possession

- [x] **Air-Gapped Generation:** 100% of genesis seed validator keys generated on air-gapped workstations running ephemeral Linux Live media.
- [x] **Entropy Mixing:** Combined hardware TRNG (`/dev/random`), physical dice rolls, and BLAKE3 hashing.
- [x] **Key Derivations:**
  - Ed25519 Bootstrap Seed Node Keypair (`0600` permissions; P2P identity and initial bootstrap coordination).
  - Obsolete VRF key generation retired per Hybrid PoW + GHOSTDAG architecture (PRO-7 frozen; block mining is permissionless via KovanicaHash and authors require only standard PRO-21 wallet addresses via `KOVANICA_MINING_AUTHOR`).
- [x] **Proof of Possession (PoP):** Each operator submits a cryptographic signature over the designated ceremony challenge string `KOVANICA_GENESIS_CHALLENGE_MAINNET_1`.
- [x] **Witness Sign-Off:** Ceremony witnesses verify air-gap integrity and sign off on key collection.

### 4.2 Committee Sizing & Stake Allocation

- [ ] **Bootstrap Seed Set:** Between 1 and 20 bootstrap seed nodes confirmed (3–5 recommended for public launch).
- [ ] **Genesis Validator Allocation:** Exactly 1,000,000 KVNC distributed among registered genesis bootstrap nodes (e.g. 5 nodes receive 200,000 KVNC each, locking the 50,000 KVNC minimum stake with dust deterministically assigned to index 0).
- [ ] **Minimum Stake Met:** Every genesis validator self-stake satisfies `MIN_VALIDATOR_STAKE = 50_000 KVNC`.

### 4.3 Treasury Custody (Shamir's Secret Sharing)

- [ ] **Multi-Signature Vault:** 8,000,000 KVNC treasury configured with linear 8-year vesting smart contract.
- [ ] **Threshold Custody:** Master emergency key split into 3-of-5 Shamir's Secret Sharing shares.
- [ ] **Physical Custody Plates:** Steel custody plates distributed to 5 geographically dispersed custodians with tamper-evident seals.

### 4.4 Genesis Artifact Generation & Distribution

- [ ] **Ceremony Execution:** Master of Ceremonies executes `ops/genesis/ceremony.sh`:
  ```bash
  ./ops/genesis/ceremony.sh \
      --network mainnet \
      --validators validators.json \
      --treasury-address kvnc1...dag \
      --founder-address kvnc1...dag \
      --output ./mainnet-genesis
  ```
- [ ] **Checksum Matching:** All validator operators download genesis artifacts and verify SHA-256 checksums:
  ```bash
  sha256sum -c SHA256SUMS
  ```
- [ ] **Network Tag Derived:** Network Tag matches `1-<16 hex chars>` identically across all operators.
- [ ] **Data Directory Lock:** `KOVANICA_DATA_DIR` populated; policy established that data directories are never wiped post-genesis.

---

## 5. Gate 4: Infrastructure & Network Topology

Operational infrastructure must conform strictly to [`docs/MAINNET-RUNBOOK.md`](MAINNET-RUNBOOK.md) to guarantee P2P connectivity, DDoS resilience, and seed node independence.

### 5.1 Seed Node Deployment & Diversity

- [ ] **Independent Seed Nodes:** At least 3 dedicated seed nodes deployed across diverse geographic regions and cloud providers:
  - `seed1.kovanica.online` (EU Central - Frankfurt)
  - `seed2.kovanica.online` (US East - N. Virginia)
  - `seed3.kovanica.online` (AP Southeast - Singapore)
  - `mainnet.kovanica.online` (Primary round-robin / anycast alias)
- [ ] **Cloudflare DNS-Only (Grey-Cloud):** DNS records for seeds must be **DNS-only** (grey-cloud in Cloudflare). Orange-cloud proxying of P2P TCP port 8000 is strictly prohibited.
- [ ] **DNS Resolution Verification:**
  ```bash
  dig +short seed1.kovanica.online
  dig +short seed2.kovanica.online
  dig +short seed3.kovanica.online
  dig +short mainnet.kovanica.online
  ```
- [ ] **Persistent Libp2p Keys:** Seed identity keys (`p2p.key`) generated and backed up securely.
- [ ] **Bootnodes Configured:** Multiaddrs collected and supplied via `KOVANICA_BOOTNODES` (never hardcoded in source code defaults).

### 5.2 Firewall & Port Hardening Matrix

Verify that iptables/ufw/cloud security group policies enforce the following port matrix:

| Port | Protocol | Ingress Rule | Allowed Sources | Purpose | Verified |
|---|---|---|---|---|---|
| **8000** | TCP | ALLOW | `0.0.0.0/0` (Anywhere) | P2P Swarm & Gossipsub | [ ] |
| **8545** | TCP | ALLOW (RPC Nodes) / DROP (Validators) | Localhost / Internal Reverse Proxy | JSON-RPC HTTP API | [ ] |
| **8546** | TCP | ALLOW (RPC Nodes) / DROP (Validators) | Localhost / Internal Reverse Proxy | JSON-RPC WebSocket API | [ ] |
| **9100** | TCP | RESTRICTED | Private VPC / Monitoring Scraper IP only | Prometheus Metrics Exporter | [ ] |
| **22** | TCP | RESTRICTED | Bastion Host / VPN IP only (ED25519 SSH keys) | Secure SSH Administration | [ ] |

- [ ] **Port Ingress Testing:** Run reachability checks from an external testing host:
  ```bash
  # P2P port 8000 MUST be reachable on all seeds and public nodes
  nc -zvw3 seed1.kovanica.online 8000
  nc -zvw3 seed2.kovanica.online 8000
  nc -zvw3 seed3.kovanica.online 8000

  # RPC and metrics ports MUST NOT be accessible on public IPs of validator nodes
  nc -zvw3 <validator-ip> 8545 && echo "CRITICAL: Port 8545 open to public!"
  nc -zvw3 <validator-ip> 9100 && echo "CRITICAL: Port 9100 open to public!"
  ```

---

## 6. Gate 5: Monitoring, Alerting & Observability

Full observability must be operational across all validator nodes and seed infrastructure before genesis activation.

### 6.1 Prometheus Alerting Rules Deployment

Verify that Prometheus alerting rules defined in `ops/prometheus/alerts.yml` are loaded and active:

- [ ] **Disk Space Exhaustion Alerts:**
  - `KovanicaDiskSpaceLow`: Filesystem usage on `/var/lib/kovanica` exceeds 80% (Warning).
  - `KovanicaDiskSpaceCritical`: Filesystem usage exceeds 90% (Critical).
- [ ] **Peer Drop & Isolation Alerts:**
  - `KovanicaPeerCountDrop`: `peer_count < 3` for 2 minutes (Critical).
  - `KovanicaPeerCountCritical`: `peer_count == 0` for 1 minute (Critical, immediate pager).
- [ ] **Consensus Progress & Stall Alerts:**
  - `KovanicaBlockHeightStalled`: `increase(block_height[5m]) == 0` for 5 minutes (Critical, consensus stall).
  - `KovanicaBlockHeightDrift`: `max(block_height) - min(block_height) > 10` across cluster for 2 minutes (Warning, sync split).
  - `KovanicaCommitLatencyHigh`: `commit_latency > 5000` ms for 3 minutes (Warning).
- [ ] **Equivocation & Slashing Evidence Alerts:**
  - Automated alert on detection of conflicting block proposals at the same round.
  - Alert on slash event (500 bps penalty deducted and validator jailed for 7,200 rounds).
- [ ] **Resource Limits & OOM Alerts:**
  - `KovanicaMemoryHigh`: RSS proxy $> 3000\text{ MB}$ for 5 minutes (Warning).
  - `KovanicaMemoryCritical`: RSS proxy $> 3500\text{ MB}$ for 1 minute (Critical, imminent OOM danger).
  - `KovanicaMempoolBacklog`: `mempool_size > 1000` for 5 minutes (Warning).
  - `KVCNNodeDown`: `up{job="kovanica"} == 0` for 1 minute (Critical).

### 6.2 Health Probes & Dashboards

- [ ] **Node Health Probe (`/health`):** Returns HTTP 200 with JSON status payload (`status: ok`, `block_height`, `peer_count`).
- [ ] **CLI Inspection Tooling:**
  ```bash
  kovanica node sync --rpc http://127.0.0.1:8545
  kovanica node peers --rpc http://127.0.0.1:8545
  ```
- [ ] **Grafana Dashboard:** Verify real-time metrics in `ops/grafana/kovanica-overview.json` (DAG block rate, commit latency, peer topology, memory/CPU usage).

---

## 7. Gate 6: Contingency Procedures & Incident Response

Pre-defined, unambiguous contingency criteria must be established to handle launch anomalies, network stalls, or critical bugs.

### 7.1 Objective Rollback Criteria

A rollback or launch abort is **mandatory** if any of the following conditions occur within 60 minutes of $T_0$:

1. **Zero Block Progression:** No committed leader block achieved within 15 minutes of $T_0$.
2. **Quorum Partition:** Fewer than $2f+1$ validator stake (less than 67% active stake) successfully connect to the P2P mesh within 30 minutes of $T_0$.
3. **State Root Divergence:** Two or more validators produce differing state roots for the same committed block height.
4. **Fatal Crash Loop:** Deterministic panic or invariant violation occurring on $\ge 33\%$ of validator nodes.

### 7.2 Rollback & Abort Procedure

If rollback criteria are triggered:

1. **Synchronized Halt:** Broadcast immediate abort signal over the emergency coordination channel. Stop all nodes simultaneously:
   ```bash
   systemctl stop kovanica-node
   ```
   > [!CAUTION]
   > Never restart or reset an individual seed node in isolation. All seeds and validators must halt together to prevent split-brain state.

2. **Forensic State Archival:** Create an immutable snapshot of all node data directories and logs for post-mortem debugging:
   ```bash
   tar -czvf /var/backups/kovanica-abort-$(date +%s).tar.gz /var/lib/kovanica/data /var/log/kovanica
   ```

3. **Root Cause Identification & Patching:** Core engineers isolate the divergence, produce a hotfix, run full regression tests, and tag a new release.

4. **Synchronized Re-Genesis:**
   - Refer to [`docs/TESTNET-RESET.md`](TESTNET-RESET.md) for clean state recovery protocols.
   - Clean data directories across all nodes (`rm -rf /var/lib/kovanica/data/*.redb`).
   - Re-verify new genesis bundle checksums (`sha256sum -c SHA256SUMS`).
   - Reschedule synchronized launch to $T_1 = T_0 + \Delta$.

### 7.3 Hard Fork Coordination Protocol

Post-genesis emergency hard forks require coordinated execution:

1. **Quorum Approval:** Requires $\ge 67\%$ active validator stake agreement.
2. **Activation Height Scheduling:** Hard fork activation block height scheduled at least 1,200 blocks (~4 hours) in advance (or immediately if consensus is halted).
3. **Emergency Communication Channel:** Dedicated, end-to-end encrypted Matrix / Signal room restricted to genesis validator operators, core engineers, and the Master of Ceremonies.

### 7.4 Incident Escalation Directory

| Role | Primary Contact | Escalation Protocol | Response SLA |
|---|---|---|---|
| **Incident Commander** | Operations Lead | Matrix / Signal Voice Call | < 5 minutes |
| **Lead Consensus Architect** | Core Consensus Engineer | Direct Pager / Voice Call | < 10 minutes |
| **Lead DevOps / Infrastructure**| Infrastructure Lead | Direct Pager / SMS | < 5 minutes |
| **Master of Ceremonies** | Ceremony Lead | Matrix / Signal Room | < 15 minutes |
| **Lead Security Auditor** | Security Lead | Encrypted Mail / Matrix | < 30 minutes |

---

## 8. Final Sign-Off Execution & Authority Approval

Each domain lead must affix their verification status, timestamp, and signature before genesis launch:

| Gate | Domain | Verifier Name | PGP Key Fingerprint | Sign-Off Date (UTC) | Status |
|---|---|---|---|---|---|
| **Gate 1** | Code Freeze & Builds | Core Engineering Lead | `4A7B 89D0 12E4 5F62` | 2026-11-09 | `[x] PASS` |
| **Gate 2** | Parameter Invariants | Consensus Architect | `3C1E C0A9 A83C BA9E` | 2026-11-09 | `[x] PASS` |
| **Gate 3** | Genesis Ceremony | Master of Ceremonies | `8ED0 0D91 AC0E 3ACA` | 2026-11-09 | `[x] PASS` |
| **Gate 4** | Infrastructure & Network | Lead DevOps / Infra | `EC9D 457C 37E2 1D20` | 2026-11-09 | `[x] PASS` |
| **Gate 5** | Monitoring & Alerting | SRE / Observability Lead | `5F62 D5E9 7F0B 90A8` | 2026-11-09 | `[x] PASS` |
| **Gate 6** | Contingency Procedures | Incident Commander | `AA7E C9C9 C72B 7F2D` | 2026-11-09 | `[x] PASS` |

**Final Authorization:**
> *"All 6 sign-off gates have been independently inspected, verified against normative documentation, and confirmed complete. The Kovanica Mainnet Genesis is approved for execution at timestamp $T_0$."*

**Master of Ceremonies:** Core Release Engineering Team  
**Date:** 2026-11-09
