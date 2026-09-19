---
description: Run cargo test with common filters
agent: test-engineer
---

Run kovanica-protocol tests with optional filters.

Usage: `$ARGUMENTS` — test filter (e.g., "adversarial", "kovanica-dag", "consensus")

Examples:
- `run-tests` — all tests
- `run-tests adversarial` — adversarial tests only
- `run-tests kovanica-dag` — consensus crate only
- `run-tests consensus.rs` — specific test file

Commands:
```bash
cd /root/kovanica-protocol
cargo test $ARGUMENTS
```

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
