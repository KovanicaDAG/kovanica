---
name: profiling
description: Benchmark and profile kovanica-protocol hot paths — use when optimizing performance, measuring throughput, or finding bottlenecks
---

# Profiling Skill

Use when a component is slow, memory-hungry, or before/after any optimization change. Establish baseline → profile → fix → re-measure.

## Trigger Keywords
benchmark, profile, flamegraph, throughput, latency, bottleneck, optimize, perf, criterion

## Baseline First (never skip)

```bash
cargo bench -p kovanica-dag 2>&1 | tee /tmp/bench-before.txt
```

Record: blocks/s, tx/s, p50/p99 latency, RSS memory. Compare only against same machine + release build.

## Tools by Question

| Question | Tool |
|----------|------|
| Where does CPU time go? | `cargo flamegraph --bin kovanica-node` |
| What allocates most? | `heaptrack` or dhat-rs |
| Lock contention? | `perf record -g` + look for futex; or `parking_lot` deadlock detection |
| Regression between commits? | `cargo bench` + `critcmp baseline feature` |
| Allocation count in hot loop? | `#[global_allocator]` with counting allocator |

## Criterion Workflow

```bash
# Run one benchmark group
cargo bench -p kovanica-dag -- ghostdag_ordering

# Save baseline on main, compare on branch
git stash && cargo bench -- --save-baseline main
git stash pop && cargo bench -- --baseline main
```

## Common Hotspots in This Codebase

1. **GHOSTDAG ordering** — avoid O(n) anticone scans; keep sorted mergeset
2. **Reachability queries** — interval tree must stay balanced; batch inserts
3. **UTXO lookups** — batch DB gets; bloom filter for negative lookups
4. **Serialization** — borsh zero-copy where possible; avoid Vec clones per block
5. **Hashing** — BLAKE3 is fast; don't hash twice; use incremental hasher

## Rules

- Never optimize without a measurement proving the bottleneck
- Micro-benchmarks lie about cache effects — validate with real node run (`-- demo`)
- Consensus changes need determinism check after optimization (same DAG → same output)
- Document wins in PR: numbers before/after

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
