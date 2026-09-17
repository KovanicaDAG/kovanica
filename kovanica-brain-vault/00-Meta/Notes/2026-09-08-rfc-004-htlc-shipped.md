# 2026-09-08 — RFC-004 (HTLC / Atomic Swap) shipped + route to 5.2 Vault

> **Links:** [[2026-09-05-protocol-evolution-6-points]] · [[ROADMAP]] · [[AGENTS]] · [[SESSIONS]]

## Shipped

RFC-004 = KVP-104 **HTLC / atomic swap** is now on `main`. This was the
"where work stopped" point from the [[2026-09-05-protocol-evolution-6-points]]
critical path (`3A → 3B+6.1 → 5.1 HTLC → 5.2 vault → …`).

- **Rebased RFC-004 onto current `main`** (PR #88, merged `fb13741`).
  The old branch was 8 commits behind `main` and carried redundant RFC-003
  commits; re-applied the pure HTLC delta as one clean commit (`7adc211`,
  +4612/−20, zero conflicts). Verified: `fmt --check` clean, `clippy
  --all-targets` 0 warnings, `cargo test` **740 passed / 0 failed** (incl. 23
  HTLC consensus tests, 4 node swap tests, FFI).
- **Restored `main` web typecheck** (PR #95, merged `a960742`). Pre-existing
  break from `77e1faa`: `routes/{network,roadmap}.tsx` added but
  `routeTree.gen.ts` never regenerated (`/network` + `/roadmap` missing from
  `FileRoutesByPath`), plus an `AddressQr address=` → `value=` prop mismatch.
  Regenerated route tree via the `tanstackStart` vite plugin + fixed the prop.
  This was blocking CI on **every** new PR, incl. #88.
- **Docs status → Shipped** (PR #96, merged `9fb83b5`): KVP-104 → Shipped,
  LEGIT-BOARD P1.1 checked off, RFC-004 spec + plan status headers updated.

RFC-004 details (for reference): dedicated `0x04` HTLC template (100-byte
`HtlcScript`), two consensus spend paths (redeem = preimage + recipient sig;
refund = sender sig at `height >= timeout`), activation gate
`HTLC_ACTIVATION_SCORE`, Tier Nolan `atomic_swap.rs`, 4 RPC commands, 6 FFI
methods. **No wire-format bump / no testnet reset.**

## Where next (critical path)

The evolving plan's critical path continues **5.1 HTLC → 5.2 Vault**
(time-lock vault / escrow via CSV + CLTV).

RFC-004 explicitly deferred **CSV (BIP-68/BIP-112 relative locktime / sequence)**
to 5.2 because it needs **per-UTXO creation-height tracking**, which does not
exist yet. CLTV is already real (RFC-004's companion fix — `apply_regular`
rejects `n_lock_time > block height`). So 5.2 = add per-UTXO creation height +
wire input `sequence` to CSV semantics + the time-lock vault template.

## Lessons burned in

- Consensus slices built on an unmerged predecessor carry redundant commits;
  rebase onto `main` after the dependency merges, dropping the now-merged
  stack (`onto-main`/`onto-main-v2` intermediate branches were the messier
  attempt at this).
- When a new route file lands on `main`, regenerate `routeTree.gen.ts` in the
  same change or CI typecheck goes red for every subsequent PR.

## Open follow-ups

- [ ] 5.2 Time-lock Vault (CSV/CLTV) — RFC + impl (next per plan)
- [ ] Soak tuning review after ~2 weeks of data (roadmap ACTIVE NEXT); seed3
      OOM (t3.micro 913MB) still needs a resize or memory reduction
- [ ] Validate RFC-004 e2e on the live testnet once deployed
