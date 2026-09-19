---
description: Run cargo-fuzz targets for input-parsing hardening
agent: test-engineer
---

Fuzz untrusted-input parsers in kovanica-protocol.

```bash
# Target from $ARGUMENTS, or default smoke over high-priority targets
if [ -n "$ARGUMENTS" ]; then
  cargo fuzz run "$ARGUMENTS" -- -max_total_time=600
else
  for t in block_decode tx_decode vrf_verify wire_message; do
    cargo fuzz run "$t" -- -max_total_time=300 || exit 1
  done
fi
```

$ARGUMENTS — optional: fuzz target name (e.g. `block_decode`)

On crash:
1. Minimize: `cargo fuzz run <target> <artifact> -- -minimize_crash=1`
2. Classify: panic / hang / OOM
3. File finding with minimized artifact path; add regression test after fix

Clean run = all targets survive their time budget with no new artifacts.

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
