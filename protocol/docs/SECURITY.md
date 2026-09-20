# Kovanica Security Notes

> **Status:** Testnet software — no investment advice. Funds can be lost.

---

## 1. Threat Model

### What GHOSTDAG (k=3) + PoW Protects

| Property | Guarantee | Assumptions |
|----------|-----------|-------------|
| **Safety (no double-spend)** | Two honest nodes never finalize conflicting transactions | > 50% of *blue work* controlled by honest miners; network delay ≤ `k` blocks |
| **Liveness** | New transactions eventually confirm | Honest miners keep mining; network eventually delivers messages |
| **Chain selection** | Heaviest blue work wins | Blue work = sum of PoW of blue blocks; deterministic ordering |

### What It Does **Not** Protect Against

| Risk | Mitigation |
|------|------------|
| **Selfish mining / block withholding** | Not mitigated; k=3 tolerates some but not all |
| **Eclipse attacks on P2P** | DHT + DNS seeds + peer scoring help; not provable |
| **Majority hash-power attack** | None — PoW chain selection follows most work |
| **Private-key compromise** | None — Ed25519 spend auth is single-key |
| **Quantum computer** | Ed25519 is vulnerable; no post-quantum path yet |

---

## 2. Consensus Guarantees

### GHOSTDAG Parameters (Testnet)
- `k = 3` — maximum blue anticone size
- `finality_depth = 100` blocks — UTXO state pruned, reorgs beyond this rejected
- `payload_pruning_depth = 1000` — block payloads evicted, headers retained
- `MULTISIG_ACTIVATION_SCORE = 0` (active from genesis)
- `NATIVE_TOKEN_ACTIVATION_SCORE = 0`
- `STEALTH_ACTIVATION_SCORE = 0`
- `SCRIPT_V2_ACTIVATION_SCORE = 0`
- `HTLC_ACTIVATION_SCORE = 0`
- `VAULT_ACTIVATION_SCORE = 0`
- `TOKENOMICS_ACTIVATION_SCORE = 0` (RFC-006)
- `CHECKPOINT_VERSION = 7`

### Finality Semantics
- A block is **final** when its blue score is > `finality_depth` below the selected tip
- Final blocks cannot be built on; their UTXO state is merged and pruned
- Reorg depth observed on testnet: 0 (linear chain)
- Expected reorg depth under attack: ≤ `k` = 3 blocks for non-final; unbounded for final (safety violation)

---

## 3. Key Handling

| Key Type | Derivation | Storage | Rotation |
|----------|------------|---------|----------|
| **Spend key (Ed25519)** | BIP39 mnemonic (24 words) → 32-byte seed | `kvnc…dag` address = `0x00 \| pk` | Not supported; new wallet required |
| **Stealth scan/spend** | Two independent Ed25519 keys (65-byte `StealthAddress`) | Published off-chain; spend key never on-chain | Not supported |
| **VRF key (staking)** | 32-byte seed (from mnemonic or ceremony) | Node memory + optional `*.wallet` file | Unbond → rebond with new key |
| **Operator wallet** | BIP39 mnemonic (auto-generated on first boot) | `$KOVANICA_DATA/operator-wallet.key` (0600) | Not yet supported |

### Best Practices
- **Never** commit wallet files to git
- **Always** use `KOVANICA_DATA` outside the repo tree
- **Hardware wallets** (Ledger/Trezor) planned — not yet implemented
- **Multisig** (RFC-001) available for M-of-N custody

---

## 4. Network & P2P

### Seed Nodes (Testnet)
| Hostname | IP | Notes |
|----------|----|-------|
| `seed.kovanica.online` | 145.223.116.178 (Hostinger VPS) | Primary, mines 1 block/60s |
| `seed2.kovanica.online` | 76.13.250.65 (Hostinger KVM2 VPS) | Secondary |
| `seed3.kovanica.online` | (retired) | — |

- **P2P port:** 9000 (TCP, **grey-cloud** only — Cloudflare proxy breaks raw TCP)
- **DNS seeds:** `seed.kovanica.online`, `seed2.kovanica.online` (`seed3` retired 2026-09-17)
- **DHT:** Kademlia XOR metric, k-buckets, relay tags 0x20–0x23

### Peer Scoring (Hardening)
- Valid block: +1 score
- Duplicate block: -5 score
- Invalid block: -20 score
- Auto-ban at score ≤ -50
- Rate limits: max bytes/msg per peer per window

---

## 5. Tokenomics (RFC-006)

| Parameter | Value |
|-----------|-------|
| Genesis subsidy | 10 KVNC/block (1,000,000,000 atoms) |
| Halving era | 2,000,000 blocks |
| Emission decay | Smooth α = ¾ (geometric) |
| MAX_SUPPLY | 90.2M KVNC |
| Coinbase maturity | 100 blocks |
| Fee split | 75% burned / 25% producer |
| Treasury | 10 × 1M KVNC vaults (placeholder keys) |

### Emission Formula
```
subsidy(h) = initial_subsidy * (3/4)^floor(h / era_length)
```
Total supply converges to ≈ 90.2M KVNC.

---

## 6. Known Limitations & Deferred Work

| Area | Limitation | Tracking |
|------|------------|----------|
| **Stealth `r_secret`** | Node derives deterministically; production must use random `r` | RFC-003 |
| **CSV relative locktime** | Per-UTXO creation-height tracking added (RFC-005); script v2 CSV opcode still decorative | RFC-005 |
| **Mint/burn authority** | Native tokens: coinbase-only mint; no tag-based policy yet | KVP-102 |
| **SPV light client** | KVLS v1 blob sync + filters + Merkle proofs shipped; browser SPV not yet | Slice 5 |
| **Hardware wallet** | Ledger (WebHID) + Trezor (WebUSB) planned; not implemented | PRODUCT-POLISH |
| **Audit** | Target Q1 2027; scope = dag + state + RPC | AUDIT-PLAN |
| **Bug bounty** | Draft ready; not yet live | BUG-BOUNTY |

---

## 7. Incident Response

### Chain Stall
- **Symptom:** Block height stops advancing
- **Check:** `KOVANICA_MINE=1` on at least one seed; seed3 OOM (resize to ≥2GB)
- **Fix:** Ensure at least one seed mining; restart stalled nodes

### Fork / Reorg
- **Symptom:** Multiple tips, peer count drops
- **Check:** `/api/head` on all seeds; Prometheus `kovanica_peer_count`
- **Fix:** Wait for GHOSTDAG resolution; manual intervention only if safety violation suspected

### Data Corruption
- **Backup:** `/root/kovanica-backups/` (encrypted, 48h/30d/90d retention)
- **Restore:** `scripts/restore-node.sh --data-dir /root/kovanica-data`
- **Drill:** Quarterly restore to `/tmp/kov-restore-drill/` and verify genesis

---

## 8. Security Contacts

- **Vulnerability disclosure:** GitHub Security Advisories (preferred) or `security@kovanica.online`
- **Public security discussions:** GitHub Issues using the [Security template](https://github.com/KovanicaDAG/kovanica/issues/new?template=security.yml)
- **Bug bounty (planned):** `BUG-BOUNTY.md` — severity tiers, safe harbor
- **Maintainer:** Toni (see `ENTITY-LEGAL.md`)

---

## 9. Audit Scope (Planned)

| Component | LoC (approx) | Criticality | Notes |
|-----------|--------------|-------------|-------|
| `kovanica-dag` | ~4,500 | Critical | GHOSTDAG, reachability, PoW, VRF |
| `kovanica-state` | ~8,500 | Critical | UTXO ledger, stake registry, hybrid admission |
| `kovanica-node` RPC | ~2,000 | High | Line RPC, explorer HTTP API |

**Out of scope:** FFI/mobile, CLI, web frontend, infra.

---

## 10. Version & Update Policy

- **Testnet resets** only on wire-format bumps or safety incidents
- **Epoch tags:** `kovanica-testnet-eN` (current: reset at RFC-006 activation)
- **Binary releases:** Rolling `v0.1.0` with SHA256SUMS on GitHub Releases
- **Dependency updates:** `cargo audit` in CI; `metrics` minor version must match exporter

---

*Last updated: 2026-09-17*
*See also: `LEGIT-BOARD.md`, `TOKENOMICS.md`, `OPS-HARDENING.md`, `AUDIT-PLAN.md`*