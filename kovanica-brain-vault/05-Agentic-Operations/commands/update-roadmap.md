---
description: Update ROADMAP.md with stage progress
agent: doc-writer
---

Update the ROADMAP.md in the vault with current stage completion status.

Steps:
1. Check kovanica-protocol for completed items
2. Update checkboxes in `KovanicaDAG/ROADMAP.md`
3. Move completed items to appropriate stage
4. Add new items to "Beyond" or "Post-Stage 3" as needed
5. Commit: `git add -A && git commit -m "docs: update ROADMAP.md — <what changed>"`

$ARGUMENTS — optional: specific stage or item to update (e.g., "stage3", "vrf", "multi-seed")

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
