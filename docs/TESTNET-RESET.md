# Testnet reset procedure

> This codebase has **no `reset` subcommand and no `KOVANICA_ALLOW_RESET`
> switch**. A reset is performed by stopping the node, wiping its data
> directory, and starting a build again. There is nothing to "apply": a fresh
> `KOVANICA_DATA_DIR` starts a fresh chain under whatever rules the running
> binary enforces.

## What a reset does and does not do

A reset changes the live mesh only when **every** seed/validator in the mesh is
rebuilt from the same commit and started again together. Restarting a single
node on a rebuilt binary while the others keep running splits that node from
the mesh.

- Chain height, balances, and the mempool are gone — the new chain starts at
  genesis.
- Consensus follows the running binary's rules. Under MysticGhost, commit
  colouring is scoped GHOSTDAG, and each committed leader mints the block
  reward. If no light nodes are bonded, the light escrow share returns to the
  contributing leaders.

## Procedure

1. **Agree and freeze.** Pick the commit that the whole mesh will run. Note
   the chain id and genesis inputs; a reset with mismatched genesis inputs
   will not converge.
2. **Stop every node in the mesh.** The running process keeps
   `consensus.redb` / `state.redb` open even if you delete them on disk, so
   the process must be stopped before the wipe.
3. **Wipe the data directory on every node.** The data directory is whatever
   `data_dir` in the config points at (`./data` by default), overridable with
   the `KOVANICA_DATA_DIR` environment variable. Delete that directory (it
   holds `consensus.redb` and `state.redb`). Some older scripts still spell the
   variable `KOVANICA_DATA`; the node reads **`KOVANICA_DATA_DIR`**.
4. **Deploy the frozen build to every node** (same binary, same commit).
5. **Start them together**, with matching bootnodes
   (`KOVANICA_BOOTNODES`, or `bootnodes` in the config; default P2P listen is
   `0.0.0.0:8000`).
6. **Verify convergence** — compare head height / digest across all nodes
   before re-opening the mesh to public traffic.

## Genesis digest guard (startup behaviour)

The node persists a digest of its genesis configuration in the data directory
on first start. On later starts it recomputes that digest and compares it with
the persisted value:

- **match** — startup proceeds normally;
- **mismatch** — startup **refuses** with a clear error naming the genesis
  mismatch, because it means the data directory was created for a different
  genesis than the one now configured (a silent start would fork);
- an **override environment variable** exists to force startup past the
  mismatch. Do not set it on a live mesh: it defeats exactly the guard that
  stops two nodes from running diverged genesis state. It is only for a
  deliberate single-node migration;
- **no persisted digest but chain state present** — startup **refuses**, with
  or without the override. Genesis writes the digest together with the state,
  so such a data directory predates the chain-4 format change (#81) and cannot
  be migrated. Wipe it (step 3).

If you see the mismatch error after a legitimate reset, the fix is the wipe in
step 3 — not the override.

## Historical note (2026-10-14)

The note this document replaced described a live mesh where the running
process was still an older `/usr/local/bin/kovanica-node` while
`consensus.redb` and `state.redb` had already been deleted from disk, and
where `seed2` / `seed3` were still on the older build. It correctly observed
that `KOVANICA_ALLOW_RESET=0` does nothing — there is no such switch in this
codebase. The correct remedy was, and remains, the manual procedure above:
stop all nodes, wipe the data directories, deploy one build everywhere, start
together.
