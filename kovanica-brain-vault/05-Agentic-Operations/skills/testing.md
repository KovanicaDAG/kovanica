---
name: testing
description: Write/run adversarial tests, property tests, consensus invariants — use when testing kovanica-protocol
---

# Testing Skill

Use for writing and running tests in kovanica-protocol. Specializes in adversarial and property-based testing for consensus code.

## Trigger Keywords
test, adversarial, property, invariant, consensus, k-cluster, reachability

## Test Categories

### Consensus (`kovanica-dag`)
- **k-cluster invariant**: `blue_anticone_size <= k` for all blue blocks
- **Determinism**: Same DAG → same linearization, same colours
- **Adversarial**: Wide forks, equivocation, partition heals, reorgs
- **Reachability**: Differential vs naive walk, incremental == fresh build
- **Difficulty/PoW**: Enforcement, target determinism, Nakamoto arithmetic

### State (`kovanica-state`)
- **Double-spend**: Parallel blocks spending same output
- **Order-independence**: Same final state regardless of merge order
- **Per-block state**: Matches batch `apply_dag`
- **Persistence**: Snapshot round-trip, store append-only
- **Finality**: Pruning, deep-reorg rejection, implicit re-org

### Node/P2P (`kovanica-node`)
- **Convergence**: Multi-node identical DAG
- **Discovery/Relay**: Hello, gossip, tx flood, mempool eviction
- **TCP/WS**: Persistent sessions, framed exchange, WebSocket
- **SPV/DHT**: Sync wire protocol, Kademlia discovery

## Test Commands
```bash
# All
cargo test

# Single
cargo test adversarial_wide_fork

# Package-specific
cargo test -p kovanica-dag

# With backtrace
RUST_BACKTRACE=1 cargo test <name>

# Property tests (proptest)
cargo test property_
```

## Adding Tests
- Location: `crates/<crate>/tests/<topic>.rs`
- Naming: `test_<scenario>`, `adversarial_<attack>`, `property_<invariant>`
- Use `proptest` for property tests
- Assert invariants directly, not implementation details

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
