---
title: "RAM Reduction Design Note (RFC-008 follow-up)"
category: 30-Operations
source: protocol/docs/RAM-REDUCTION.md
synced: 2026-09-26
---
# RAM Reduction Design Note (RFC-008 follow-up)

Status: **Draft** — consensus-safe, no fork, no validation-rule change.
Scope: `kovanica-dag`, `kovanica-state`, `kovanica-node`.

## 1. Measured baseline (live testnet, 2026-09-24)

| Metric | seed1 (explorer) | seed2 (seed) |
|---|---|---|
| Chain | 29,400 blocks | 29,400 blocks |
| RSS after load (old binary, no block pruning) | ~8.1GB (OOM'd) | ~7.5GB |
| RSS after load (RFC-008, block pruning 1000) | **10.1GB** | **7.44GB** |
| DAG length after pruning | 1,454 blocks | 1,455 blocks |
| Available RAM | 4.0Gi / 15Gi | 290Mi / 7.8Gi |

Key observation: **the DAG is pruned to ~1,450 blocks, yet RSS stays at the
load peak.** The peak is built during log replay, when pruning is disabled;
the post-load prune frees the memory, but the allocator retains it.

## 2. Memory ownership

### 2.1 `Dag` (`kovanica-dag/src/dag.rs`)
- `nodes: HashMap<BlockId, Node>` — one `Node` per present block.
- Each `Node` holds `GhostdagData`:
  - `blue_anticone_sizes: HashMap<BlockId, KParam>` — **one entry per blue
    block in the block's past**. `ghostdag.rs:47` clones the selected parent's
    entire map on every insert. **O(blue_score) per block → O(n²) across the
    DAG.** This is the dominant term (≈7–10GB at 29,400 blocks).
  - `mergeset_blues` / `mergeset_reds: Vec<BlockId>` — bounded by DAG width.
- `reach: Reachability` — `intervals`, `fcs`, `tree_children` — O(n) each.
- Bounded **post-load** by `block_pruning_depth` (RFC-008), but the replay
  builds the full DAG first.

### 2.2 `Ledger` (`kovanica-state/src/ledger.rs`)
- `tip_state: UtxoSet` — single materialised state at the selected tip;
  bounded by the number of *unspent* outputs (small).
- `deltas` / `stake_deltas` / `heights` / `staked_seen` — per-block, non-final
  only; `Ledger::insert` calls `self.prune()` (ledger.rs:3110), so these are
  bounded **once `finality_depth` is set**.
- `asset_registry` — per-asset metadata, small.

### 2.3 `Node` (`kovanica-node/src/node.rs`)
- `mempool: MempoolV2` — capped at 100k txs / 100MB by default.
- `pending`, `multisig_scripts` — small.

## 3. Root cause of the 7.5–10GB RSS

`LedgerStore::open_impl` (store.rs:119) replays the whole log with pruning
**disabled** (`u64::MAX`); `restore_miner_and_policy` (explorer.rs:710) applies
the depths only *after* the load. The replay therefore materialises the full
O(n²) `blue_anticone_sizes` maps. The post-load prune drops 28,000 blocks, but
glibc retains the freed pages, so RSS never returns to the OS.

## 4. Minimal change set (largest RAM reduction first)

### 4.1 Prune during replay — the big win (consensus-safe)
Pass the network profile's pruning policy into `LedgerStore::open` and set
`finality_depth`, `payload_pruning_depth`, `block_pruning_depth` on the ledger
**before** the replay loop. Both `Dag::insert` (dag.rs:1212) and
`Ledger::insert` (ledger.rs:3110) already prune incrementally, so the DAG and
per-block state stay bounded throughout the load.

- Expected effect: load peak drops from ~7.5–10GB to the steady-state DAG size
  (~1,450 blocks) ≈ **100–300MB**.
- Invariant: every evicted block is already final (`block_pruning_depth >=
  finality_depth`, enforced by `apply_block_pruning_depth`), and
  `Ledger::is_final` treats evicted blocks as final, so `reconstruct_state` /
  `reconstruct_stake` stop at the pruning boundary. Replay acceptance is
  unchanged: a replayed block's selected parent is always within
  `finality_depth` of the replay tip (topological order ⇒ `parent_score =
  s-1 >= tip_score - finality_depth`), so no replayed block is newly rejected.
- Classification: **consensus-safe** — no validation rule changes; identical
  post-load state, identical acceptance, only the memory high-water mark moves.

### 4.2 Operational env vars (client-only)
- `KOVANICA_MINE` already defaults to `false` (explorer.rs:558). Document it.
- `KOVANICA_MAX_PEERS` (default 8) — parameterises the hard-coded
  `replenish_peers_from_dht(8)` (explorer.rs:385).
- `KOVANICA_MEMPOOL_MAX_TXS` — wires into `MempoolConfig::max_txs`.
- `KOVANICA_PRUNE_DEPTH` — overrides `block_pruning_depth` (clamped to
  `>= finality_depth` by the ledger).
- `KOVANICA_LIGHT_MODE=1` — aggressive profile: smaller prune depth, no
  historical body retention beyond the window, smaller mempool cap.
- Classification: **client-only** (node-local policy; consensus is a pure
  function of the DAG).

### 4.3 Background prune reporting (client-only)
The prune already runs on every insert. Add a periodic log/metrics line
reporting DAG length, evicted count, and freed bytes (via `/api/metrics` or a
DEBUG log), so operators can observe the bounded steady state.

### 4.4 CPU / networking (client-only)
- Peer cap from `KOVANICA_MAX_PEERS`; inbound connection rate already limited
  by `p2p_hardening` per-peer budgets.
- Miner loop is already gated on `self.mining` (explorer.rs:306) and the tick
  loop sleeps 40ms — no busy spin when `KOVANICA_MINE=0`. Verify and document.
- Reduce DEBUG log noise in release builds.

### 4.5 Build (client-only)
- Add `[profile.release] lto = "thin"` (currently default `lto = false`),
  `codegen-units = 1` for the node workspace.

## 5. Why this is consensus-safe

- No change to block validation, GHOSTDAG ordering, selected-parent/blue-work
  selection, or RFC-006 tokenomics.
- Pruning only removes blocks that are already final and can never be built on
  (`BuildsOnPrunedHistory` fires exactly where `Finality` would reject).
- The replay path produces the same ledger state as today; only the memory
  high-water mark during replay changes.
- All new knobs are node-local policy (env vars), never consensus parameters.

## 6. Deliverables

1. Diff/patches for `kovanica-dag`, `kovanica-state`, `kovanica-node`.
2. This design note.
3. Env-var table + recommended light-node config block (see
   `HOWTO_LIGHT_NODE.md`).
4. Tests: pruning keeps selected-tip calculation and UTXO spends inside the
   kept window; replay-with-policy produces identical state to
   replay-then-prune.
5. Before/after RSS measurement procedure (see §1 baseline).