---
title: "Kovanica Consensus Migration: PoW + VRF → PoA"
category: 60-Planning
source: kovanica-poa-migration/README.md
synced: 2026-09-26
---
# Kovanica Consensus Migration: PoW + VRF → PoA

**Status:** Draft  
**Date:** 2026-09-24  
**Target:** Minimal-change migration from hybrid PoW + VRF to pure Proof-of-Authority (PoA) while keeping GHOSTDAG + SPV intact.

## Documents in this package

| Document | Purpose |
|----------|---------|
| [[60-Planning/kovanica-poa-migration--01-RFC-POA|01-RFC-POA.md]] | Formal specification of the new PoA consensus |
| [[60-Planning/kovanica-poa-migration--02-AUTHORITY-SET|02-AUTHORITY-SET.md]] | On-chain Authority Set design (3–4 validators, updateable) |
| [[60-Planning/kovanica-poa-migration--03-MIGRATION-GUIDE|03-MIGRATION-GUIDE.md]] | Step-by-step code migration (minimal surface changes) |
| [[60-Planning/kovanica-poa-migration--04-BLOCK-VALIDATION|04-BLOCK-VALIDATION.md]] | Exact changes to block validation & `Dag::insert` |
| [[60-Planning/kovanica-poa-migration--05-SPV-COMPATIBILITY|05-SPV-COMPATIBILITY.md]] | How SPV / light clients continue to work |
| [[60-Planning/kovanica-poa-migration--06-CONFIG-AND-FLAGS|06-CONFIG-AND-FLAGS.md]] | Feature flags, env vars, genesis changes |
| [[60-Planning/kovanica-poa-migration--07-TESTING-PLAN|07-TESTING-PLAN.md]] | Test cases and acceptance criteria |

## High-level summary

- **Keep:** GHOSTDAG (k=3), multi-parent blocks, UTXO ledger, Ed25519, SPV, pruning, all existing RFCs (multisig, multi-asset, stealth, HTLC, vaults…).
- **Remove / disable:** PoW mining, difficulty retargeting, VRF-based leader election.
- **Add:** Authority Set + slot-based round-robin + authority signature check.
- **Optional keep:** VRF only as randomness beacon (not required for leader selection).

## Design goals

1. Minimal code surface change.
2. Lowest possible CPU / RAM usage.
3. 3–4 validators.
4. On-chain updateable Authority Set (no hard-fork needed for membership changes).
5. Full backward compatibility for light clients and existing state format where possible.

## Recommended reading order

1. `01-RFC-POA.md`
2. `02-AUTHORITY-SET.md`
3. `03-MIGRATION-GUIDE.md`
4. Rest as needed.