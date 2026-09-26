---
title: "Migration Guide: PoW + VRF → PoA"
category: 60-Planning
source: kovanica-poa-migration/03-MIGRATION-GUIDE.md
synced: 2026-09-26
---
# Migration Guide: PoW + VRF → PoA

**Goal:** Change only the block-production permission layer.  
Leave GHOSTDAG, UTXO application, SPV, networking, and all existing RFCs untouched as much as possible.

## 1. High-level change map

| Component              | Action                          | Notes |
|------------------------|---------------------------------|-------|
| `pow.rs`               | Disable / feature-gate          | Keep file for now |
| `difficulty.rs`        | Disable under PoA               | |
| VRF leader selection   | Remove from block production    | Keep VRF module for optional randomness |
| `Block` struct         | Add `authority_sig`             | Minimal layout change |
| `Dag::insert` / validator | New authority check          | Replace PoW target check |
| Genesis                | Add initial Authority Set       | |
| New TX type            | AuthorityUpdate                 | Only new feature |
| SPV / light client     | Verify authority signature      | Small change |
| Config / env           | `KOVANICA_CONSENSUS=poa`        | |

## 2. Recommended implementation order

### Step 1 – Feature flag & config
```rust
// In config or env
pub enum ConsensusMode {
    PowVrf,   // current
    Poa,      // new
}
```
Default to `Poa` after migration, but keep the old path compilable behind a feature for testing.

### Step 2 – AuthoritySet type
Create a new small module (e.g. `crates/kovanica-dag/src/authority.rs` or inside `kovanica-state`):

```rust
pub struct AuthoritySet {
    pub authorities: Vec<PublicKey>,
    pub threshold: u8,
}

impl AuthoritySet {
    pub fn active(&self, slot: u64) -> &PublicKey { ... }
    pub fn verify_update(...) -> bool { ... }
}
```

### Step 3 – Genesis
- Hard-code or load the initial 3–4 public keys.
- Create the first Authority UTXO (or state object) in genesis.

### Step 4 – Block production (node side)
Replace the mining / VRF lottery loop with:

```rust
loop {
    let now = current_timestamp();
    let slot = slot_from_timestamp(now, SLOT_DURATION);
    if i_am_active_authority(slot) {
        let block = build_block(...);
        let sig = sign(my_key, &block_hash_without_sig);
        block.authority_sig = sig;
        broadcast(block);
    }
    sleep_until_next_slot();
}
```

### Step 5 – Validation path
In the block validator (the place that currently calls `meets_target` or VRF verify):

```rust
if consensus_mode == Poa {
    let slot = slot_from_timestamp(block.timestamp, SLOT_DURATION);
    let expected = authority_set.active(slot);
    verify_ed25519(expected, &block_hash_without_sig, &block.authority_sig)?;
    // no PoW check
} else {
    // old PoW + VRF path
}
```

### Step 6 – Authority Update transaction
Implement the special transaction that spends the current Authority UTXO and creates a new one.  
Apply it inside the normal UTXO / state application pipeline.

### Step 7 – SPV / light client
See `05-SPV-COMPATIBILITY.md`.  
Only the header verification changes: instead of checking PoW, check the authority signature against the known Authority Set (or a commitment to it).

### Step 8 – Clean-up (optional, later)
- Once PoA is stable, remove or `#cfg` out the old mining code.
- Keep VRF code if you want a randomness beacon.

## 3. Files that will most likely change

```
crates/kovanica-dag/
  src/block.rs          // add authority_sig
  src/dag.rs            // insert / validation hook
  src/pow.rs            // feature-gate
  src/difficulty.rs     // feature-gate
  src/vrf.rs            // stop using for leader election
  src/authority.rs      // NEW

crates/kovanica-state/
  src/ledger.rs         // Authority UTXO handling
  src/tx_validity.rs    // new AuthorityUpdate tx

crates/kovanica-node/
  src/node.rs           // block production loop
  src/...               // config parsing
```

## 4. Backward compatibility strategy

- Keep the old `KOVANICA_POW=1` path behind a flag during the transition.
- Existing blocks on testnet remain valid under the old rules until a hard activation height / epoch.
- After activation height, only PoA blocks are accepted.
- Light clients need a one-time update to understand the new signature field and Authority Set.

## 5. Activation

Two possible approaches:

**A. Genesis reset** (simplest for testnet)  
Restart testnet with PoA from block 0.

**B. Scheduled activation height**  
At a predetermined blue-score / height the rules switch.  
Requires a bit more code but keeps history.

Recommendation for current stage: **A (clean testnet reset)**.