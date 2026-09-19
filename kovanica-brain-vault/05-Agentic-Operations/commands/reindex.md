---
description: Rebuild CODE_INDEX.md from source tree
agent: protocol-dev
---

Rebuild CODE_INDEX.md with current kovanica-protocol source structure.

Steps:
1. Scan `/root/kovanica-protocol` for source files
2. Categorize by crate: kovanica-dag, kovanica-state, kovanica-node, kovanica-cli, web
3. Generate file:// links for each .rs/.tsx/.ts file
4. Update `KovanicaDAG/CODE_INDEX.md`
5. Update links in `myObsidianVaultDAG.md`, `NAVIGATION.md`, `ROADMAP.md`
6. Commit: `git add -A && git commit -m "docs: rebuild CODE_INDEX.md"`

```bash
cd /root/kovanica-protocol
find crates web -name "*.rs" -o -name "*.tsx" -o -name "*.ts" | head -50
```

$ARGUMENTS — optional: "verify" to check current index against source

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
