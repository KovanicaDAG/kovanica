---
description: Run cargo fmt --check && cargo clippy --all-targets
agent: code-reviewer
---

Run lint checks for kovanica-protocol.

```bash
cd /root/kovanica-protocol
cargo fmt --check && cargo clippy --all-targets
```

$ARGUMENTS — optional: "fix" to auto-fix fmt issues

If "fix" provided:
```bash
cargo fmt && cargo clippy --all-targets --fix --allow-dirty --allow-staged
```

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
