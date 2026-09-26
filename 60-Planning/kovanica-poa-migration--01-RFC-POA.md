---
title: "RFC: Proof-of-Authority (PoA) Consensus for Kovanica"
category: 60-Planning
source: kovanica-poa-migration/01-RFC-POA.md
synced: 2026-09-26
---
# RFC: Proof-of-Authority (PoA) Consensus for Kovanica

**RFC Number:** KVP-201 (Consensus)  
**Status:** Draft  
**Author:** Kovanica Core  
**Created:** 2026-09-24  
**Replaces:** Hybrid PoW + VRF block production  
**Compatible with:** GHOSTDAG (k=3), UTXO ledger, SPV, all existing KVP-1xx features

---

## 1. Motivation

The current hybrid PoW + VRF design is too resource-heavy for the intended low-memory / low-CPU deployment targets (edge devices, light nodes, small VPS).  

We replace the block-production mechanism with a pure **Proof-of-Authority** model while keeping the rest of the protocol (especially GHOSTDAG ordering and SPV) unchanged.

## 2. Goals

- Extremely low CPU and RAM usage.
- Deterministic, predictable block production.
- Support for 3–4 validators.
- On-chain updateable validator set (no hard fork required).
- Minimal changes to existing crates (`kovanica-dag`, `kovanica-state`, `kovanica-node`).
- Full compatibility with current SPV / light-client verification path.

## 3. Non-Goals

- Changing GHOSTDAG parameters or colouring rules.
- Changing the UTXO model or transaction format (except for the new Authority Update transaction type).
- Introducing staking, slashing, or economic security (that can be added later as PoS).
- Breaking existing light clients more than necessary.

## 4. Overview of the new consensus

```
Block Production   →  PoA (Authority Set + Slot Round-Robin)
Ordering           →  GHOSTDAG (unchanged)
Finality           →  GHOSTDAG blue-score depth (unchanged)
Light verification →  SPV headers + authority signature (minimal change)
```

### 4.1 Authority Set

A fixed-size ordered list of Ed25519 public keys (validators).  
Stored on-chain and updateable by a threshold of the current set.

### 4.2 Slot-based Round-Robin

```
SLOT_DURATION = 3 seconds          // configurable
slot_number   = floor(timestamp / SLOT_DURATION)
active_index  = slot_number % authorities.len()
active_key    = authorities[active_index]
```

Only the active validator for the current slot may produce a block.  
A block is valid only if it carries a valid Ed25519 signature from the active key for that slot.

### 4.3 Block Header Changes (minimal)

Existing fields stay.  
We add / reinterpret:

| Field              | Change                                      |
|--------------------|---------------------------------------------|
| `nonce`            | Unused (kept for wire compatibility)        |
| `work` / difficulty| Ignored when PoA is active                  |
| `authority_sig`    | **New** – Ed25519 signature over the block  |
| `slot`             | Optional explicit slot number (or derived)  |

Signature is computed over the canonical block hash **excluding** the signature itself (standard).

### 4.4 GHOSTDAG remains the source of truth

- Multi-parent references stay.
- Blue / red colouring stays.
- Selected parent and linearisation stay.
- Only the *permission to create* a block changes.

## 5. Security model

With 3–4 validators and threshold updates:

- Safety assumes at most `f = 1` Byzantine validator (for 4 validators).
- Liveness requires at least one honest validator that is online in its slots.
- Authority Update requires a configurable threshold (recommended: `ceil(2/3)` or absolute 3-of-4).

This is a classic permissioned / consortium security model.

## 6. Upgrade path

See `03-MIGRATION-GUIDE.md`.  
A feature flag (`KOVANICA_CONSENSUS=poa`) allows gradual rollout and easy rollback during testing.

## 7. Future extensions

- Weighted round-robin (if some authorities should produce more blocks).
- VRF randomness beacon (optional, already present in codebase).
- Later migration to stake-weighted PoA or pure VRF-PoS without touching GHOSTDAG.