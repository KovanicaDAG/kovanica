---
description: Run security audit checks (deps, unsafe, secrets, clippy)
agent: security-auditor
---

Run the security audit battery on kovanica-protocol. Read-only — no edits.

```bash
cargo audit
cargo deny check advisories bans licenses sources 2>/dev/null || echo "cargo-deny not configured"
grep -rn "unsafe" crates/ --include="*.rs" | grep -v "#\[test\]" | grep -v "tests/"
grep -rniE "(api[_-]?key|secret|password)\s*=\s*\"" crates/ --include="*.rs" | grep -v test || true
```

Then assess results:
1. List any RUSTSEC advisories with severity + affected crate + fix version
2. Any `unsafe` outside tests → justify or flag as finding
3. Any hardcoded secrets → Critical, report immediately
4. Summarize overall risk: Critical / High / Medium / Low / Clean

Follow the checklist in [[../skills/security-audit]] for consensus-path review.

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
