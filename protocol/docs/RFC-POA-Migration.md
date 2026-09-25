# RFC-POA: Pure Proof-of-Authority Consensus Migration

**Status:** Draft  
**Consensus Impact:** consensus-safe (replaces PoW/VRF admission with authority signatures; GHOSTDAG k=3, UTXO, SPV unchanged)  
**KVP:** KVP-201 (authority set + slot round-robin + authority signature)  
**Related:** RFC-001..RFC-006 (all shipped features preserved), RFC-007/KVP-106 (NFT, client-only), DeFi/RWA (post-PoA)

---

## One-Line Summary
Replace hybrid PoW+VRF block admission with pure Proof-of-Authority (PoA): fixed authority set, slot-based round-robin scheduling, Ed25519 authority signatures on blocks; keep GHOSTDAG (k=3), UTXO ledger, SPV, pruning, and all KVP-1xx features intact.

---

## Motivation
- **Operational simplicity:** No mining hardware, no VRF key management for validators, deterministic block times (~3s slots).
- **Resource efficiency:** Near-zero CPU/RAM for block production; validators sign one Ed25519 signature per slot.
- **Deterministic finality:** Slot-based scheduling + GHOSTDAG k=3 gives predictable confirmation times.
- **Governance readiness:** On-chain authority set updates via threshold-signed AuthorityUpdateTx (KVP-201).
- **SPV compatibility:** Authority signatures replace PoW checks; light clients need only genesis authority set + update proofs.

---

## Specification

### 1. Authority Set (KVP-201)
- **On-chain representation:** Single live "Authority UTXO" (tag `KVA1` || `authority_set_hash`).
- **Contents:** 3–4 Ed25519 public keys (`authorities`), threshold `t` (2 ≤ t ≤ n, n ≤ 16).
- **Genesis:** First Authority UTXO created in genesis block (coinbase output with tag `KVA1`).
- **Update:** `AuthorityUpdateTx` spends current Authority UTXO, requires ≥t distinct signatures from current authorities, creates new Authority UTXO with new set (3–4 keys, n ≤ 16).
- **Maturity:** Authority UTXO spends are subject to 100-block maturity (RFC-005 vault/CSV).

### 2. Block Structure Changes
- **New field:** `authority_sig: [u8; 64]` (Ed25519 signature over `block.hash_without_authority_sig()`).
- **Preserved fields:** `nonce`, `work`, `timestamp_ms`, `parents`, `payload`, `vrf_*` (for wire compatibility; ignored under PoA).
- **Hash:** `block.id()` = BLAKE3(`parents` || `work` || `timestamp_ms` || `nonce` || `payload` || `authority_sig`).

### 3. Slot & Scheduling
- **Slot duration:** `SLOT_DURATION_MS = 3000` (configurable via `KOVANICA_SLOT_DURATION`).
- **Slot number:** `slot = timestamp_ms / SLOT_DURATION_MS`.
- **Active authority:** `authorities[slot % authorities.len()]`.
- **Slot consistency:** Block's `timestamp_ms` must fall within its claimed slot; `slot` must be ≥ parent slots.

### 4. Block Validation (replaces PoW/VRF checks)
1. **Structural:** Parents exist, non-empty, no duplicates, timestamp ≥ all parents.
2. **Authority signature:** Verify `authority_sig` against `active_authority(slot).public_key` over `hash_without_authority_sig()`.
3. **Slot consistency:** `slot == timestamp_ms / SLOT_DURATION_MS`; `slot ≥ parent_slots`.
4. **GHOSTDAG:** Compute selected parent, mergeset, blue/red coloring (k=3 unchanged).
4. **Transaction validation:** UTXO, signatures, conservation, RFC-001..005 rules (unchanged).

### 5. GHOSTDAG & Consensus (unchanged)
- k=3, blue score/work drive chain selection.
- Linearization: `order(B) = order(sp) ++ mergeset ++ [B]`.
- Finality: `finality_depth` blue-score depth (default 50, RFC-008).
- Pruning: payload + block pruning per RFC-008 (unchanged).

### 6. SPV / Light Clients
- **Header:** Add `authority_sig` (64 bytes) to block header.
- **Verification:** Check authority signature against known authority set for the slot.
- **Authority set sync:** Genesis includes initial authority set hash; `AuthorityUpdateTx` provides update proofs (Merkle path to Authority UTXO).
- **Finality:** Blue-score depth unchanged; light clients wait for `finality_depth` blue blocks.

### 7. Configuration & Feature Flags
- **Cargo features:** `poa` (default), `pow-vrf` (opt-in for legacy/test).
- **Env vars:**
  - `KOVANICA_CONSENSUS=poa|pow` (default `poa`)
  - `KOVANICA_SLOT_DURATION=3000` (ms)
  - `KOVANICA_AUTHORITIES=<hex_pk1>,<hex_pk2>,...` (genesis override)
  - `KOVANICA_AUTHORITY_THRESHOLD=<t>` (genesis override)
- **Genesis TOML:**
  ```toml
  [consensus]
  mode = "poa"
  slot_duration_ms = 3000
  [authority]
  keys = ["<hex_pk1>", "<hex_pk2>", "<hex_pk3>"]
  threshold = 2
  ```

### 7. Wire Format & Compatibility
- **Block encoding:** `authority_sig` appended after `nonce` (flag byte: 0 = none, 1 = present + 64 bytes).
- **Legacy blocks:** `insert` strips `authority_sig` when `hybrid=None` (replay compatibility).
- **Snapshot/checkpoint:** v6+ includes `authority_sig`; legacy readers strip it.

---

## Backwards Compatibility
- **Consensus-breaking:** Yes (hard fork). Requires coordinated upgrade at activation height.
- **RPC/API:** New `authority_sig` field in block JSON; `staking` RPC reports authority set.
- **Explorer:** Shows authority signature, slot, active authority.
- **Wallet/CLI:** No changes to transaction signing; block production uses `produce_block` (staked path).

---

## Open Questions
1. **Slot duration:** 3s default; should it be configurable per-network?
2. **Authority set size:** 3–4 for launch; governance process for expansion to 16?
2. **Activation:** Coordinated flag day vs. on-chain signaling (BIP-9 style)?
3. **Fallback:** Keep `pow-vrf` feature for testnet/legacy; remove after mainnet stability?
4. **SPV proofs:** Authority set Merkle proofs vs. full UTXO path for update verification?

---

## Implementation Plan (Milestones)

| Milestone | Deliverable | Exit Criteria |
|-----------|-------------|---------------|
| M1: Core Types ✅ | `AuthoritySet`, `AuthorityUpdateTx`, `Block.authority_sig`, `hash_without_authority_sig()` | Unit tests: sig verify, slot active authority, update tx validation — **done** (`crates/kovanica-dag/src/authority.rs`, 21 tests; snapshot v7 + checkpoint v9 carry the sig; live-parity tests `#[ignore]`d pending the PoA reset) |
| M2: Consensus ✅ | `BlockValidator` authority/slot check in `Dag::insert`; `Dag::insert_for_replay` skips PoW/VRF | Integration: 3-validator round-robin, 4-validator liveness (1 offline), re-org under GHOSTDAG — **done** (`Dag::set_poa` first-class switch + `check_poa` in `dag.rs`; `insert_for_replay` now skips PoW/difficulty/VRF/PoA; `tests/poa.rs`, 10 tests) |
| M3: Config/Genesis ✅ | `KOVANICA_CONSENSUS`, `KOVANICA_SLOT_DURATION`, `KOVANICA_AUTHORITIES`, genesis TOML | Genesis block carries authority set; `poa` feature default — **done** (`poa_config_from_env` + `PoaGenesisConfig` in `explorer.rs`; `genesis_with_poa` commits `KVA1 \|\| set_hash`; node PoA production via `try_produce_poa`/`set_authority_signing_key`; wire `BlockRecord.authority_sig` flag byte 2; PoA-aware immediate sends; `load_log_with_poa*` readers; `tests/poa_node.rs` incl. log round-trip, 9 tests) |
| M4: SPV | Header `authority_sig`; light client authority set sync + update proofs | Light client syncs from genesis, verifies authority sigs, processes update tx |
| M5: RPC/Explorer | `staking` RPC; explorer shows authority sig, slot, active authority | Explorer displays authority set, slot, signatures |
| M6: Testing | 3-validator soak (24h), authority update, re-org, SPV, resource (CPU/RAM vs PoW) | 24h stable testnet; authority update works; CPU/RAM < 10% of PoW |

---

## References
- `/root/kovanica-poa-migration/01-RFC-POA.md` (KVP-201)
- `/root/kovanica-poa-migration/02-AUTHORITY-SET.md`
- `/root/kovanica-poa-migration/03-MIGRATION-GUIDE.md`
- `/root/kovanica-poa-migration/04-BLOCK-VALIDATION.md`
- `/root/kovanica-poa-migration/05-SPV-COMPATIBILITY.md`
- `/root/kovanica-poa-migration/06-CONFIG-AND-FLAGS.md`
- `/root/kovanica-poa-migration/07-TESTING-PLAN.md`
- `protocol/docs/RFC-006-Tokenomics.md` (MAX_SUPPLY, maturity, fee burn)
- `protocol/docs/RFC-001..005.md` (shipped features preserved)