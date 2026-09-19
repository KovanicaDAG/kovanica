---
name: migration
description: Plan and execute store schema migrations and protocol upgrades — use when changing DB layout, checkpoint format, or consensus activation
---

# Migration Skill

Use when changing store schema, checkpoint formats, pruning policy, or activating new consensus rules.

## Trigger Keywords
migration, schema, upgrade, hard fork, soft fork, activation, backfill, rollback

## Migration Anatomy

Every migration = ordered `(version, fn)` pair in a registry. See [[../subagents/migration-engineer]] for the Rust registry pattern. Rules:

1. **Idempotent** — re-run is no-op (check version first)
2. **Batched** — ≤10k keys per batch; log progress every batch
3. **Atomic version bump** — data write + `set_version` in one transaction where store supports it
4. **Bounded memory** — never load full UTXO set; stream by prefix scans

## Pre-flight (mandatory)

```bash
# 1. Backup
cp -r ~/.kovanica/testnet/db ~/.kovanica/testnet/db.pre-v<N>

# 2. Dry run on copy, production-sized snapshot
cargo run -p kovanica-node -- migrate --db /tmp/db-copy --dry-run --verbose

# 3. Measure: time + peak RSS must fit maintenance window
/usr/bin/time -v cargo run -p kovanica-node -- migrate --db /tmp/db-copy
```

## Protocol Upgrade Ladder

| Step | Action |
|------|--------|
| 1 | RFC note in vault: motivation, spec diff, rollback plan |
| 2 | Implement behind feature flag, default off |
| 3 | Testnet activation at height H with ≥2 weeks notice |
| 4 | Monitor: orphan rate, peer versions, sync lag for 48h |
| 5 | Mainnet height H; flag flips to enforcing |
| 6 | Remove old code path after compat window |

## Rollback

Store migrations are forward-only. Rollback = restore backup:
```bash
systemctl stop kovanica-node
rm -rf ~/.kovanica/testnet/db && mv ~/.kovanica/testnet/db.pre-v<N> ~/.kovanica/testnet/db
systemctl start kovanica-node   # verify tip height + peer count recover
```

## Verification After Any Migration

- [ ] `--verify --deep` passes on migrated DB
- [ ] Fresh genesis sync in parallel reaches same tip hash as migrated node
- [ ] Metrics normal 30 min post-start (height lag, peer count, error rate)

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
