---
title: "Authority Set Design"
category: 60-Planning
source: kovanica-poa-migration/02-AUTHORITY-SET.md
synced: 2026-09-26
---
# Authority Set Design

**Status:** Draft  
**Related:** KVP-201 (PoA)

## 1. Requirements

- Exactly 3–4 validators at genesis (configurable).
- On-chain updateable without hard fork.
- Threshold signature requirement for updates.
- Minimal impact on existing UTXO ledger.

## 2. Recommended storage: Special Authority UTXO

We reuse the existing UTXO model with a **special-purpose output type**.

### 2.1 Authority Output

```rust
// Conceptual
struct AuthorityOutput {
    authorities: Vec<PublicKey>,   // ordered list, len 3..=4 (or up to 16 later)
    threshold: u8,                 // e.g. 3 for 3-of-4
    // optional: metadata, version, activation height
}
```

- There is at most **one live Authority UTXO** at any time (the “current set”).
- Spending it requires signatures from at least `threshold` of the *current* authorities.
- The spending transaction creates a new Authority UTXO with the updated list.

### 2.2 Genesis

In the genesis block / genesis state the first Authority UTXO is created with the initial 3–4 keys and the chosen threshold.

## 3. Authority Update Transaction

New transaction type (or a special script / opcode):

```
AuthorityUpdateTx {
    inputs:  [ current Authority UTXO ],
    outputs: [ new Authority UTXO ],
    signatures: Vec<Signature>,   // from current authorities
}
```

Validation rules:

1. Input must be the current live Authority UTXO.
2. Number of valid signatures ≥ current threshold.
3. Signatures must come from distinct keys that are in the *old* set.
4. New set must satisfy size constraints (3–4 recommended, hard limit e.g. 16).
5. New threshold must be sane (`1 ≤ threshold ≤ new_set.len()`).

After successful application the old Authority UTXO is spent and the new one becomes the sole live set.

## 4. Alternative (if pure UTXO is awkward)

If adding a special output type is too invasive, store the Authority Set in a **dedicated state object** parallel to the UTXO set (similar to how some protocols keep a stake registry).  

Both approaches are acceptable; the UTXO approach stays closer to the current design philosophy.

## 5. Slot assignment

```rust
fn active_authority(slot: u64, authorities: &[PublicKey]) -> &PublicKey {
    &authorities[(slot as usize) % authorities.len()]
}

fn slot_from_timestamp(ts: u64, slot_duration: u64) -> u64 {
    ts / slot_duration
}
```

Recommended `SLOT_DURATION`: **3 seconds** (configurable via genesis / config).

## 6. Missed slots

If the assigned authority does not produce a block in its slot:

- The slot simply passes.
- The next slot’s authority may produce the next block.
- No punishment (this is pure PoA, not PoS).

Optional future improvement: allow any authority to produce after a grace period (e.g. 2× slot duration).

## 7. Coinbase / subsidy

Two simple policies (choose one):

**A.** Coinbase goes entirely to the authority that produced the block.  
**B.** Coinbase is split equally among all current authorities (or burned + small reward).

Recommendation for minimal change: **Policy A**.

## 8. Wire & storage compatibility

- Keep the existing block header layout as much as possible.
- Add `authority_signature: [u8; 64]` (Ed25519).
- `nonce` and `work` fields can remain for backward compatibility but are ignored under PoA.