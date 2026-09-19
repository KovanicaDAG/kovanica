---
name: code-review
description: Review PRs for consensus safety, style, correctness — use when reviewing kovanica-protocol changes
---

# Code Review Skill

Use for reviewing pull requests in kovanica-protocol. Focus on consensus correctness first, then general quality.

## Trigger Keywords
review, PR, pull request, consensus, safety, correctness

## Review Priority Order

1. **Consensus correctness** — Any change to selected-parent, mergeset, k-cluster, blue score, linearization
2. **Determinism** — No HashMap iteration, wall-clock, unstable sorts in consensus paths
3. **Adversarial tests** — Required for consensus changes
4. **Style/lint** — `fmt`, `clippy`, module docs
5. **Security** — No secrets, input validation

## Consensus Change Checklist
- [ ] Written rationale naming reference protocol (GHOSTDAG, Kaspa, PHANTOM, etc.)
- [ ] Adversarial tests: wide forks > k, Byzantine parents, tie-breaks, partitions
- [ ] Deterministic: pure function of DAG, tie-break = BlockId byte order
- [ ] No regression on k-cluster invariant (`blue_anticone_size <= k`)

## Commands
- `cargo fmt --check`
- `cargo clippy --all-targets`
- `cargo test`
- `cargo test -p kovanica-dag adversarial_`

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
