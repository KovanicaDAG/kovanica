# Consistency Fixes — Roadmap · Backlog · SDK Skeleton

**Date:** 2026-09-23  
**Status:** Applied  
**Monorepo home:** `docs/backlog/` (source of truth; pack copies may lag)

This note records the inconsistencies found across the planning pack and
`kovanica-sdk` skeleton, and the resolutions applied.

---

## 1. Issues found

| # | Area | Problem | Resolution |
|---|------|---------|------------|
| 1 | **Address format** | SDK used `kov1` / then assumed bech32. **Node uses base58:** `kvnc` + base58(version‖payload) + `dag`. | Implemented P2PK encode/decode in `kovanica-keys`. Spec: `ADDRESS-AND-SIGHASH-SPEC.md`. |
| 2 | **SDK scaffold status** | Task **S-01** still "Todo", but the workspace skeleton already exists. | S-01 → **Done**. Remaining work is S-02+ and the address/sighash lock. |
| 3 | **Roadmap phase vs backlog** | Roadmap put full `kovanica-sdk` only in **Phase 1** (post-mainnet). Backlog + sprints treat SDK core as **P0** overlapping pre-mainnet. | Phase 0 now includes: SDK scaffold + address/sighash alignment. Phase 1 keeps: publish, WASM/TS polish, cookbook. |
| 4 | **Immediate actions stale** | Roadmap item "Scaffold kovanica-sdk" already done. | Replaced with: lock address + sighash to node; then M-01…M-11; then S-02…S-08. |
| 5 | **Module naming** | Backlog wrote `kovanica_types` (underscore). Crates use `kovanica-types` (hyphen). | Docs use crate names with hyphens; Rust paths use underscores — both noted as normal. |
| 6 | **Examples list** | Crate draft listed `transfer.rs`, `create_asset.rs`, `htlc_swap.rs`. Skeleton only ships `generate_wallet.rs`. | Documented: only `generate_wallet` exists; other examples are S-10. |
| 7 | **Sighash** | Tx signing used a non-consensus placeholder sighash. Not called out in backlog tasks. | Added **S-03c**: implement the node-identical sighash before any broadcast. |
| 8 | **Derivation path** | Skeleton + CLI each derived keys differently (first-32-bytes-of-BIP39-seed / sha256 stopgap). | **Frozen:** SLIP-0010 ed25519 `m/44'/3007'/0'/0'/i'` (all hardened). Canon: `DERIVATION.md` + `sdk/crates/kovanica-keys/tests/slip10_vectors.rs`. CLI, SDK, and web now agree. |
| 9 | **Fee floor** | Skeleton fee estimate ignored the RFC-006 minimum. | `kovanica-fee::estimate_with_min(size_bytes, subsidy, min)` = `max(size-based, fee_floor)`; fee floor = `max(1, subsidy/500_000)` atoms/byte. |
| 10 | **Address HRP wording** | SDK crate-structure draft still said "bech32-style". | Corrected: the node format is **base58**, not bech32. Bech32 appears only as an unused transitive npm dependency in the web lockfile. |

---

## 2. Close-out notes (monorepo import, 2026-09-23)

- **M-01…M-09, M-11 Done** (see `TASK-BREAKDOWN-BIP39-SDK.md`): bip39 pinned;
  CLI `wallet new|restore|show`; web flows (M-05/M-06, web branch); path frozen
  (M-07); unit + known-answer + property tests (M-08/M-09); docs (M-11 —
  `DERIVATION.md` + this backlog). **M-10** (security review) tracked in the task
  breakdown.
- **S-03 keys Done** (SLIP-0010 + `kvnc…dag` P2PK codec + known-answer vectors);
  **S-03b** Done for P2PK (stealth/HTLC/vault helpers pending); **S-03c** Partial
  (algorithm locked; byte-identical `encode_into` port open); **S-08** Done
  (fee-floor-aware estimate). Remaining S-04…S-07, S-09…S-13 per the task table.
- **Foundation / Founder**: imported as **drafts** — NOT done, awaiting counsel
  (see `docs/foundation/`).
- Web wallet and CLI now share one derivation; a drifting implementation fails
  the shared known-answer suites (SDK Rust + web `tests/`).

---

*Keep this file updated when a new inconsistency is resolved.*