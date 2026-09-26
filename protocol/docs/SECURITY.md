# Kovanica Security Notes

> **Status:** Draft — testnet software, no investment advice. Funds can be lost.  
> **Consensus impact:** none (documentation of the threat model; changes no
> protocol rule). The *threat model itself* changes class with the PoA
> decision below.

---

## 0. Consensus decision — the threat model changes class (2026-09-25)

> **Consensus decision (ratified 2026-09-25): Kovanica is PoA-only.**
> Proof-of-Work is being **removed**, not merely disabled. Items marked
> `[TARGET]` are ratified but not yet implemented; `[CURRENT]` items describe
> shipped code. Canonical policy and the removal inventory live in
> [`RFC-POA-Migration.md` §0](./RFC-POA-Migration.md).

**Read this before quoting anything below.** §1 is written for the
PoW-era chain and is therefore *only* true `[CURRENT]`. The ratified target is
a **permissioned** network, and permissioned networks do not get their safety
from hash power — they get it from a *small, known, accountable set of block
producers*. The security posture changes in the following ways, and this
document will be rewritten to match once the removal lands:

| Property | `[CURRENT]` PoW-era (what §1 describes) | `[TARGET]` PoA-only | Effect on the guarantees |
|----------|------------------------------------------|----------------------|--------------------------|
| Source of safety | Majority of *blue work* is honest | Majority of the **authority set** is honest | Assumption moves from "miners" to "a named, finite key set" — a smaller and *auditable* attack surface, but also a *concentrated* one |
| Sybil / cost of a majority attack | Buy or rent hash power | Compromise or coerce **N/2 of the authority keys** | Key custody becomes *the* critical security property. Authority keys must therefore be held in a real ceremony — they are **not** self-derived and **not** PoW keys, and the shipped testnet placeholders must never reach mainnet. With hybrid also removed (§0.7.1) there is **no** capital-weighted or stake-weighted fallback: one lever, one trust assumption |
| Permissionless participation | Anyone may mine and submit blocks | Only configured authorities may propose blocks | **This is a real loss of functionality, not a wash.** The permissionless admission path is removed with PoW, and the staked-VRF path is removed with hybrid; the network stops being open-entry. Nothing in this document should be read as "PoA is at least as safe" |
| Censorship resistance | Grows with hash distribution | Bounded by the authority set | A colluding majority *can* censor; a hash-power majority could not have done so as cheaply. Restated honestly in §1.1 rather than deleted |
| Auditability | Anonymous, unidentifiable producers | Known, key-attributable producers | A net **improvement**: misbehaviour is attributable to a key, so slashing/exclusion is possible — *if* governance decides it (see `[OPEN]`) |

### 1.1 What does not change

- **GHOSTDAG k=3** is untouched. Linearization, colouring, reachability and the
  finality model are the same before and after.
- **UTXO ledger, Ed25519 spend authentication, 1 KVNC = 100_000_000 atoms** —
  unchanged.
- **RFC-006 tokenomics is unchanged and stays canonical:** MAX_SUPPLY
  **90.2M KVNC**, genesis subsidy **s₀ 10 KVNC/block**, era **2 000 000**
  blocks, decay **α 3/4 per era**, coinbase maturity **100 blocks**, fee split
  **75% burned / 25% to producer**. The emission curve is height-indexed, not
  work-indexed, and `cumulative_minted` is capped in `apply_block`, so removing
  PoW does not move a single supply figure. Supply guarantees are therefore
  **identical** under both models.
- Ed25519's post-quantum weakness (§1) is likewise unchanged and is not a
  consequence of the PoA decision.

### 1.2 Open decisions this document cannot answer

1. **Governance and slashing.** Nothing here states who may sit in the
   authority set, how membership changes, or what happens to a key that
   double-signs or goes offline. The *mechanism* is settled — the set is fixed
   at genesis and rotates only by on-chain M-of-N `AuthorityUpdateTx` — but the
   *inputs* (initial set choice, eligibility, key ceremony, threshold `t`,
   expansion, dissolution/recovery) are `[OPEN]` per
   [`RFC-POA-Migration.md` §0.7.2](./RFC-POA-Migration.md). In particular
   "misbehaviour is attributable to a key, so slashing is possible" above is
   **conditional** — no slashing mechanism is specified.
2. **Adversarial coverage for PoA.** The `challenger_*` mining suites are the
   repository's adversarial *consensus* coverage and are slated for deletion
   with the mining path. Re-establishing them as PoA adversarial tests
   (wrong-producer, double-sign/slot violation, stale-slot, missing-sig,
   work inflation, authority-update abuse) is `[OPEN]` per §0.7.3. Until then,
   PoA's adversarial posture is **thinner than PoW's was** — a gap that widens
   exactly as the trusted key set narrows.
3. **Client-side admission.** With a fixed authority set, a client's
   "is this chain valid?" check becomes "did a majority of the configured
   authorities sign?", which is a *local configuration* question as much as a
   cryptographic one. The API surface for that is undecided.

---

## 1. Threat Model

> `[CURRENT]` — **PoW-era framing.** Superseded in shape by §0; kept verbatim
> so the delta stays auditable. After the PoA removal lands this section is
> replaced, not deleted.

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
| **Majority hash-power attack** | None — PoW chain selection follows most work `[TARGET]`-superseded: a **majority-authority** attack replaces this row, with no equivalent "None" — the mitigation is key custody plus governance, neither of which exists yet |
| **Private-key compromise** | None — Ed25519 spend auth is single-key. *Sharper under PoA:* authority keys become a consensus-critical compromise, not just a funds compromise |
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
| `kovanica-dag` | ~4,500 | Critical | GHOSTDAG, reachability, PoW `[TARGET]`-for-removal, VRF `[TARGET]`-for-removal (hybrid dropped, §0.7.1) |
| `kovanica-state` | ~8,500 | Critical | UTXO ledger, **RFC-005 vault/CSV + treasury vaults**; stake registry and hybrid admission `[TARGET]`-removed (§0.7.1) |
| `kovanica-node` RPC | ~2,000 | High | Line RPC, explorer HTTP API |

**Out of scope:** FFI/mobile, CLI, web frontend, infra.

`[TARGET]` — after the PoA removal the dag row's critical surface becomes
GHOSTDAG + **authority-set validation** (membership, threshold counting, slot
ordering), which is *new* code and must be added to this scope before it is
written. The VRF / staked-VRF surface is **`[TARGET]`-removed** (hybrid is
dropped entirely, RFC-POA-Migration §0.7.1) rather than merely out of scope, so
this row shrinks; the compensating gap is PoA adversarial coverage (§1.2 item 2).
The ledger's RFC-006 constants are out of scope for change by this document and
are unchanged: MAX_SUPPLY 90.2M KVNC, s₀ 10 KVNC/block, era 2 000 000, α 3/4,
maturity 100, fee 75% burned / 25% producer. **RFC-005 vault/CSV and the
treasury vaults are also unaffected** by the hybrid removal — they do not
depend on the stake registry.

---

## 10. Version & Update Policy

- **Testnet resets** only on wire-format bumps or safety incidents
- **Epoch tags:** `kovanica-testnet-eN` (current: reset at RFC-006 activation)
- **Binary releases:** Rolling `v0.1.0` with SHA256SUMS on GitHub Releases
- **Dependency updates:** `cargo audit` in CI; `metrics` minor version must match exporter

---

*Last updated: 2026-09-17*
*See also: `LEGIT-BOARD.md`, `TOKENOMICS.md`, `OPS-HARDENING.md`, `AUDIT-PLAN.md`*