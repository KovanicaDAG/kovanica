---
description: Plan or execute a store migration / protocol upgrade
agent: migration-engineer
---

Coordinate a schema migration or protocol upgrade for kovanica-protocol.

Mode from `$ARGUMENTS`:
- `plan <name>` — write RFC note: motivation, spec diff, rollback plan, testnet height
- `dry-run` — backup + migrate a DB copy, report time and peak memory
- `apply` — full pre-flight, then apply (requires explicit confirmation)

```bash
# dry-run example
cp -r ~/.kovanica/testnet/db ~/.kovanica/testnet/db.pre-check
/usr/bin/time -v cargo run -p kovanica-node -- migrate --db ~/.kovanica/testnet/db.pre-check --dry-run --verbose
```

$ARGUMENTS — required: `plan <migration-name>` | `dry-run` | `apply`

Safety gates before `apply`:
1. Backup exists and is verified (`db.pre-v<N>`)
2. Dry-run completed on production-sized snapshot
3. Rollback steps documented in this session
4. User confirmed explicitly

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
