---
title: "Block Validation Changes under PoA"
category: 60-Planning
source: kovanica-poa-migration/04-BLOCK-VALIDATION.md
synced: 2026-09-26
---
# Block Validation Changes under PoA

## 1. Current validation (simplified)

```
1. Structural checks (parents exist, timestamp, etc.)
2. PoW target check (meets_target)
3. Optional VRF leader proof
4. GHOSTDAG colouring / selected parent
5. Transaction validity + state transition
```

## 2. New validation under PoA

```
1. Structural checks (unchanged)
2. Authority signature check          ← NEW
3. Slot consistency check             ← NEW
4. GHOSTDAG colouring / selected parent (unchanged)
5. Transaction validity + state transition (unchanged)
```

PoW and VRF-leader checks are skipped when `ConsensusMode::Poa` is active.

## 3. Authority signature verification

```rust
fn verify_authority_signature(
    block: &Block,
    authority_set: &AuthoritySet,
    slot_duration: u64,
) -> Result<(), ValidationError> {
    let slot = block.timestamp / slot_duration;
    let expected_key = authority_set.active(slot);

    // Hash of the block excluding the signature field itself
    let message = block.hash_without_authority_sig();

    if !ed25519_verify(expected_key, &message, &block.authority_sig) {
        return Err(ValidationError::InvalidAuthoritySignature);
    }
    Ok(())
}
```

## 4. Slot rules (recommended)

- A block’s timestamp must fall inside the claimed slot (or the slot is derived purely from timestamp).
- Optionally enforce that the block’s slot is strictly greater than the maximum slot of its parents (prevents extreme timestamp manipulation).
- Clock drift tolerance: ±1–2 slots is usually acceptable for 3-second slots.

## 5. Integration point in code

The cleanest place is the existing pluggable validator hook that `Dag` already supports (`BlockValidator` trait / `Dag::with_validator`).

```rust
impl BlockValidator for PoaValidator {
    fn validate(&self, dag: &Dag, block: &Block) -> Result<(), Error> {
        // 1. basic structural checks (can call existing helpers)
        // 2. authority signature + slot
        verify_authority_signature(block, &self.authority_set, self.slot_duration)?;
        // 3. any extra PoA-specific rules
        Ok(())
    }
}
```

GHOSTDAG logic stays completely outside this validator.

## 6. What stays exactly the same

- Multi-parent references
- Reachability / mergeset calculation
- Blue / red colouring (k-cluster)
- Selected parent choice
- Linearisation order used by the state layer
- All UTXO spending rules, multisig, stealth, HTLC, vaults, multi-asset, etc.