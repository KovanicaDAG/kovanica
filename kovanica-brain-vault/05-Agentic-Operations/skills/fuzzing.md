---
name: fuzzing
description: Fuzz deserialization, consensus inputs, and parsers with cargo-fuzz — use when hardening input handling or after parsing changes
---

# Fuzzing Skill

Use to find crashes/panics in anything that parses untrusted bytes: block/tx decoding, wire messages, VRF proofs, filters, config.

## Trigger Keywords
fuzz, fuzzing, cargo-fuzz, crash, corpus, arbitrary, proptest

## Setup

```bash
cargo install cargo-fuzz
cargo fuzz init   # creates fuzz/ crate once
```

## Targets to Maintain (fuzz/fuzz_targets/)

| Target | Entry point | Priority |
|--------|-------------|----------|
| `block_decode` | `Block::decode(&[u8])` | High |
| `tx_decode` | `Transaction::decode(&[u8])` | High |
| `vrf_verify` | `VrfProof::verify(bytes)` | High |
| `wire_message` | P2P message framing | High |
| `filter_decode` | SPV Golomb-Rice filter read | Medium |
| `config_parse` | node config loader | Low |

## Running

```bash
# One target, 5 min smoke (CI-friendly)
cargo fuzz run block_decode -- -max_total_time=300

# Deep run with dict + parallel jobs
cargo fuzz run block_decode -- -dict=fuzz/dict/block.dict -jobs=8 -max_total_time=3600

# Reproduce a crash found by OSS-Fuzz or CI
cargo fuzz run block_decode fuzz/artifacts/block_decode/crash-<hash>
```

## Rules for Good Fuzz Targets

1. **No panics allowed** — target must return `Result`; any panic = bug
2. **Bound work per input** — reject oversized early: `if data.len() > MAX { return Ok(()) }`
3. **Seed corpus** — commit valid fixtures (genesis block, sample txs) in `fuzz/corpus/<target>/`
4. **Dictionary helps** — magic bytes, known opcodes in `fuzz/dict/`
5. **Determinism check inside target** — decode twice, assert equal output (catches HashMap-order bugs)

## CI Integration

Nightly job: 10 min per high-priority target; upload artifacts on crash. Block release if new crash within 7 days of tag.

## Triage Flow

Crash → minimize (`-minimize_crash=1`) → classify (panic vs hang vs OOM) → fix root cause → add regression test with minimized artifact → re-run 30 min clean before closing.

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
