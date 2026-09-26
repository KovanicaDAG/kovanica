---
title: "SPV / Light Client Compatibility"
category: 60-Planning
source: kovanica-poa-migration/05-SPV-COMPATIBILITY.md
synced: 2026-09-26
---
# SPV / Light Client Compatibility

## Goal

Keep the existing SPV design working with the smallest possible change.

## Current SPV assumptions (typical)

Light clients usually verify:

1. Block headers form a valid GHOSTDAG / chain of headers.
2. Proof-of-work (or equivalent) on each header.
3. Merkle proofs for transactions of interest.
4. Sufficient depth / blue-score for finality.

## Changes required under PoA

### 1. Replace PoW check with Authority signature check

```
Old:  header.work meets difficulty target
New:  header.authority_sig is valid under the Authority Set for that slot
```

### 2. Authority Set distribution to light clients

Light clients need to know the current Authority Set (or a commitment to it).

**Simple approach (recommended for 3–4 validators):**

- Include the full Authority Set (or its hash) in the genesis header / checkpoint.
- When an Authority Update happens, the light client must receive the update (via a special proof or by downloading the Authority Update transaction + Merkle proof).
- Because the set is tiny (3–4 keys), even embedding the whole set in a header commitment is cheap.

**Alternative:**

- Maintain a short commitment (hash of the Authority Set) in every block header.
- Light clients track the commitment and verify updates with a Merkle proof against the Authority UTXO.

### 3. Finality

Finality rule stays the same: wait for sufficient blue-score depth on the selected chain.  
No change needed.

### 4. Header size impact

Adding a 64-byte Ed25519 signature is negligible.  
`nonce` and old `work` fields can stay for wire compatibility.

## Migration for existing light clients

1. Update the verification library to understand `authority_sig`.
2. Ship the initial Authority Set with the client (or fetch it from a trusted checkpoint).
3. After the PoA activation height, stop checking PoW and start checking the authority signature.

Because the set is small and updates are rare, the additional complexity is low.