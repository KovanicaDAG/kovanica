# Kovanica Protocol — Security Audit RFP

**Project**: Kovanica Mainnet Genesis Security Audit  
**Issued**: 2026-09-26  
**Response Deadline**: 2026-10-03  
**Engagement Period**: 30 days (target start: 2026-10-06)  
**Budget Range**: To be discussed based on scope  
**Classification**: CONFIDENTIAL — NDA required before code access

---

## 1. Project Overview

Kovanica is a Layer 1 blockchain protocol using **GHOSTDAG BlockDAG (k=3)** consensus with a **pure UTXO ledger**, **Ed25519 signatures**, and native token **KVNC** (1 KVNC = 100M atoms). 

**Current state**: Testnet operational with **PoA consensus** (3 authorities, threshold 2, 3s slots). **Mainnet genesis planned Q4 2026** with PoW consensus (GHOSTDAG k=3).

**Codebase**: Rust monorepo (~50k LOC consensus-critical)
- `kovanica-dag` — GHOSTDAG consensus, DAG ordering, difficulty
- `kovanica-state` — UTXO ledger, transaction validation, script VM
- `kovanica-node` — P2P, mempool, RPC, explorer, block production
- `kovanica-cli` — Wallet, transaction builder, key management

---

## 2. Scope (Consensus-Critical Only)

### In Scope (MUST audit)

| Component | Files | Priority |
|-----------|-------|----------|
| **GHOSTDAG consensus** | `kovanica-dag/src/` | 🔴 CRITICAL |
| **UTXO ledger & validation** | `kovanica-state/src/validation.rs`, `ledger.rs` | 🔴 CRITICAL |
| **RFC-006 tokenomics enforcement** | Supply cap, maturity, fee burn, subsidy | 🔴 CRITICAL |
| **P2P gossip & sync** | `kovanica-node/src/net.rs`, `p2p.rs` | 🔴 CRITICAL |
| **Block production (PoA + PoW)** | `kovanica-node/src/node.rs` (produce_block, try_produce_poa) | 🔴 CRITICAL |
| **Script VM & sighash** | `kovanica-state/src/script.rs`, `sighash.rs` | 🔴 CRITICAL |
| **Ed25519 usage** | All signing/verification paths | 🔴 CRITICAL |

### In Scope (RFC Features)

| KVP | Feature | Files |
|-----|---------|-------|
| KVP-101 | Multisig (M-of-N P2SH) | `kovanica-state/src/multisig.rs` |
| KVP-102 | Native multi-asset | `kovanica-state/src/assets.rs` |
| KVP-103 | Stealth + Script v2 | `kovanica-state/src/stealth.rs`, `script_v2.rs` |
| KVP-104 | HTLC atomic swaps | `kovanica-state/src/htlc.rs` |
| KVP-105 | Time-lock vaults + CSV | `kovanica-state/src/vault.rs` |

### Out of Scope

- Explorer web frontend (TypeScript/TanStack)
- Wallet UI
- CLI UX (non-consensus)
- Documentation

---

## 3. Specific Focus Areas

### 3.1 Consensus Safety (GHOSTDAG k=3)
- [ ] Blue score / selected parent computation correctness
- [ ] Anticone / merge set logic
- [ ] Reorg depth bounds and finality
- [ ] Difficulty adjustment (PoW mode) — no oscillations, bounded retarget
- [ ] PoA admission: authority sig verification, slot scheduling, threshold enforcement

### 3.2 Supply Invariants (RFC-006 Hard Rules)
- [ ] **MAX_SUPPLY = 90.2M KVNC** — never exceeded at any height
- [ ] **Coinbase maturity = 100 blocks** — immature spends rejected
- [ ] **Fee burn = 75%** — exactly 75% burned, 25% to producer
- [ ] **Subsidy curve** — s₀=10 KVNC, era=2M, α=3/4, geometric decay
- [ ] **Fee floor** — `max(1, subsidy/500_000)` atoms/byte enforced
- [ ] Treasury vaults — 10 × 1M KVNC, RFC-005 scripts, correct unlock heights

### 3.3 P2P & Sync Security
- [ ] Block/transaction validation before propagation
- [ ] Full-dump sync: topological ordering (recent fix: `topo_sort_records`)
- [ ] Headers-first sync: difficulty/PoA verification
- [ ] Eclipse/Sybil resistance — peer selection, connection limits
- [ ] DoS bounds: message sizes, timeouts, rate limits
- [ ] No secret material on P2P wire

### 3.4 Cryptographic Correctness
- [ ] Ed25519: signature format (64-byte / 128 hex), verification
- [ ] Sighash: all transaction fields committed, no malleability
- [ ] Script VM: no opcode gaps, resource bounds (ops, stack, bytes)
- [ ] Key derivation: BIP39, Ed25519 from seed, no reuse

### 3.5 Recent Fixes to Verify
- [ ] `net.rs:apply_decoded` — topological sort prevents "parent delta missing" panic
- [ ] `node.rs:verify_header_body` — PoA `authority_sig` in canonical ID

---

## 4. Deliverables

| Deliverable | Format | Deadline |
|-------------|--------|----------|
| **Kickoff call** | 1h video | Day 1 |
| **Threat model** | Markdown (STRIDE per component) | Day 7 |
| **Interim findings** | Verbal + written summary | Day 15 |
| **Final report** | PDF + Markdown | Day 30 |
| **Fix verification** | Re-test after patches | Day 37 |

**Final report must include**:
- Executive summary (business risk)
- Findings table: ID, severity (Critical/High/Medium/Low/Info), component, description, PoC, fix recommendation
- Code coverage: which files/functions reviewed
- Supply invariant verification: automated tests or proof
- Consensus safety argument

---

## 5. Access & Environment

- **Code**: Private GitHub repo `KovanicaDAG/kovanica` (NDA → access granted)
- **Testnet**: Live at `https://testnet.kovanica.online` (PoA, height ~100+)
- **Local dev**: `cargo test -p kovanica-dag -p kovanica-state -p kovanica-node`
- **Binaries**: Reproducible build instructions provided

---

## 6. Evaluation Criteria

| Criterion | Weight |
|-----------|--------|
| Blockchain/consensus audit experience | 30% |
| Rust codebase audit experience | 25% |
| UTXO / Script VM / Crypto expertise | 20% |
| P2P / networking security | 15% |
| Timeline / availability | 10% |

---

## 7. Submission

Email proposal to: **security@kovanica.online** (PGP: `0x...`)

Include:
1. Firm background & relevant audits (public reports preferred)
2. Team members assigned + bios
3. Proposed methodology & tooling
4. Timeline with milestones
5. Proposed fee structure
6. References (2+ past clients)

---

## 8. Contact

**Technical Lead**: [Name] — [Signal/Email]  
**Project Coordinator**: [Name] — [Signal/Email]

*All communications encrypted. NDA executed before repo access.*