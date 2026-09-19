---
description: Profile a node/bench binary with flamegraph or perf
agent: performance-engineer
---

Profile kovanica-protocol to find CPU/memory bottlenecks.

Target: `$ARGUMENTS` (default: `node demo` — runs the in-process demo workload)

```bash
# CPU flamegraph (requires cargo-flamegraph + perf)
cargo flamegraph -p kovanica-node -- $ARGUMENTS

# Or perf record if flamegraph unavailable
perf record -g cargo run --release -p kovanica-node -- $ARGUMENTS
perf report | head -50
```

$ARGUMENTS — optional: binary args, e.g. `demo`, `sync`, `bench`

Report:
1. Top 5 frames by self time
2. Suspected bottleneck category: CPU / lock contention / allocation / I/O
3. Suggested fix with expected impact; verify with `benchmark` command after change

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
