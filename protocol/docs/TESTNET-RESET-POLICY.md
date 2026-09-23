# Testnet Reset Policy

**Status:** ACTIVE (kovanica-testnet) · Owner: genesis-testnet role
**Applies to:** `kovanica-testnet` only. Mainnet has no reset path.

## 1. When we reset

A testnet reset (genesis wipe) is a **last resort**, triggered only by:

1. **Consensus format bumps** — wire-format changes that make old blobs
   undecodable by new readers (e.g. RFC-002 native-token asset flag, RFC-006
   activation fork). These are *mandatory* resets: old and new nodes cannot
   agree on a chain.
2. **Irrecoverable state corruption** on both seeds (disk loss, bad
   checkpoint) with no usable backup.
3. **Explicit operator decision** documented in this file before execution —
   never ad-hoc.

Non-triggers: difficulty retarget windows, slow sync, mempool churn, explorer
outages, or a single seed going down (the other seed + backups cover these).

## 2. What survives a reset

| Item | Survives? | Notes |
| --- | --- | --- |
| Wallet keys / mnemonics | ✅ | Client-side; addresses are derived from keys, not chain state |
| Address format (`kvnc…dag`) | ✅ | Versioned encoding, unchanged by resets |
| RFC-006 tokenomics constants | ✅ | 90.2M cap, s₀=10 KVNC, era 2M, α=¾, maturity 100, fee 75/25 — frozen |
| Pre-reset balances | ❌ | Wiped at activation forks (RFC-006 wiped all pre-fork balances) |
| Treasury vaults | ⚠️ | Re-created from the RFC-006 genesis (10 × 1M vaults) |
| Node data dirs (`KOVANICA_DATA`) | ❌ | Must be deleted before first sync on the new genesis |

## 3. Reset procedure (operator-only)

1. Announce on Discord + docs.kovanica.online **≥ 24h before** a planned reset.
2. Snapshot both seeds' `data/` to cold storage (for post-mortem only).
3. Stop both seed units; delete `KOVANICA_DATA` on both.
4. Deploy the new genesis binary; start seed1, verify `/api/head` genesis hash
   matches the release notes, then start seed2.
5. Verify: both seeds agree on `/api/head` genesis + block count; a pristine
   clone with `KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000`
   cold-bootstraps to the same tip.
6. Update `protocol/TESTNET-RFC006.md` + `OPERATIONS.md` genesis hash and
   reset date in the same commit as the reset.

## 4. Safety rules

- `KOVANICA_ALLOW_RESET=0` on every public-facing node (default). A reset is
  a **manual, coordinated** operation — never an env-var accident.
- The public explorer never runs `KOVANICA_ALLOW_RESET=1`.
- No open faucet on public-facing nodes without explicit isolation and
  documentation (AGENTS.md rule 7).
- After a reset, the faucet cap and fee floor are re-verified against
  `/api/bootstrap` before announcing.

## 5. History

| Date | Reason | Genesis | Notes |
| --- | --- | --- | --- |
| RFC-006 activation | Consensus fork (tokenomics) | `9565fc20…` | All pre-RFC-006 balances wiped; see `TESTNET-RFC006.md` |