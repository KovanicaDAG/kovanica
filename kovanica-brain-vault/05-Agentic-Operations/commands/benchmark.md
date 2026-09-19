---
description: Run criterion benchmarks and compare against baseline
agent: performance-engineer
---

Run benchmarks for kovanica-protocol and report results.

```bash
cd /root/Obsidian-Vault
REPO=/root/kovanica-protocol   # verify path exists first

# Full suite or filtered by $ARGUMENTS (e.g. "ghostdag" or "state utxo")
if [ -n "$ARGUMENTS" ]; then
  cargo bench -p kovanica-dag -- $ARGUMENTS
else
  cargo bench --workspace
fi
```

$ARGUMENTS — optional: benchmark name filter (e.g. `ghostdag_ordering`, `utxo_lookup`)

After running:
1. Report blocks/s, tx/s, latency percentiles from output
2. If a baseline exists (`critcmp`), show delta vs baseline
3. Flag any regression >10% as needing investigation

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
