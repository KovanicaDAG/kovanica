---
description: Create a milestone or checkpoint in the obsidian vault and save it
agent: vault-sync
---

Create a milestone checkpoint in the Obsidian Vault.

Steps:
1. Update tracking documents or create an artifact noting the milestone using the provided $ARGUMENTS.
2. Commit the changes: `git add -A && git commit -m "docs: checkpoint - $ARGUMENTS"`
3. Push the changes: `git push origin main`

$ARGUMENTS — required: Description of the checkpoint or milestone.

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
