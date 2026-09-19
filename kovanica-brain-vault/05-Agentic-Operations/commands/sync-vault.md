---
description: Sync vault snapshots from kovanica-protocol
agent: vault-sync
---

Sync the Obsidian vault with the latest kovanica-protocol source.

Steps:
1. Pull latest from `/root/kovanica-protocol`
2. Compare with vault snapshots in `KovanicaDAG/kovanica-*/`
3. Update `KovanicaDAG/myObsidianVaultDAG.md` if structure changed
4. Rebuild `KovanicaDAG/CODE_INDEX.md` if source files added/removed
5. Update `KovanicaDAG/ROADMAP.md` stage checkboxes
6. Update `KovanicaDAG/AGENTS.md` if conventions changed
7. Commit: `git add -A && git commit -m "docs: sync vault snapshot (<what changed>)"`
8. Push: `git push origin main`

$ARGUMENTS — optional: specific area to sync (e.g., "dag", "node", "roadmap")

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
