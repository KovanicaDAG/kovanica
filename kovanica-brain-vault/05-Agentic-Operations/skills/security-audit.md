---
name: security-audit
description: Audit crypto, consensus, and input handling for vulnerabilities — use when reviewing security-sensitive code or preparing releases
---

# Security Audit Skill

Use for security review of consensus paths, crypto usage, P2P input handling, and dependency hygiene. Run before every release and on any crypto/consensus change.

## Trigger Keywords
security, audit, vulnerability, CVE, exploit, DoS, attack, threat, secrets

## Audit Order (highest risk first)

1. **Consensus determinism** — any RNG/time/HashMap-order influence = Critical
2. **Crypto usage** — nonce reuse, non-constant-time compares, weak defaults
3. **Untrusted input parsing** — blocks/txs/peer messages: size caps before allocation
4. **Resource exhaustion** — unbounded Vec growth from peer data, mempool floods
5. **Secrets** — keys in logs, hardcoded test keys in prod paths, `.env` committed
6. **Dependencies** — `cargo audit`, `cargo deny check advisories`

## Quick Checks

```bash
cargo audit
cargo deny check advisories bans licenses sources
cargo clippy --all-targets -- -D warnings
grep -rn "unsafe" crates/ | grep -v test
grep -rniE "(api[_-]?key|secret|password)\s*=" crates/ --include="*.rs" | grep -v test
```

## Input Validation Checklist

- [ ] Max size checked BEFORE deserializing (block ≤ 1MB default, tx ≤ 100KB)
- [ ] Varint/length-prefixed fields bounded — no `vec![0; len]` with attacker `len`
- [ ] All arithmetic near overflow uses checked/saturating ops in consensus math
- [ ] Peer messages rate-limited per connection AND globally
- [ ] Malformed input → disconnect + ban score, never panic

## Consensus Invariants (verify on every audit)

- Same DAG bytes → identical linearization (run twice, compare)
- `blue_anticone_size <= k` holds for adversarial wide-fork fixtures
- Finalized blocks never revert under reorg depth tests
- VRF outputs unique per block (no two blocks same round win)

## Reporting

Severity: **Critical** (funds loss / chain halt) → **High** (safety violation) → **Medium** (DoS) → **Low** (info leak) → **Info** (hardening).

Format findings as: title, severity, component path, impact, PoC/repro steps, fix. See [[../subagents/security-auditor]] for full template.

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
