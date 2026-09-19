---
name: KovanicaDAG-project-layer
description: Kovanica-specific project laws, appended on top of core/ doctrine when compiling MARKDOWN.god for this vault
---

# KovanicaDAG — Project Layer

> Rescued from the legacy compiled brain (`Obsidian-Vault/agents/MARKDOWN.god`,
> "The Ultimate Agent Brain", stamped 2026-08-24 13:44 UTC) before it gets
> overwritten by a core-only rebuild. These 7 rules existed ONLY inside that
> generated file — not in KNOWN-TRAPS.md or DECISION-LOG.md (both still empty
> templates) — so a plain `build-god-brain.sh` run with no layer argument
> would silently delete them. Pass this file as a LAYER to keep them.

## Project Laws (override core/00-THE-LAWS.md generic laws where more specific)

1. **Finish Line** — work is done when *shipped*, never when merely edited.
   Vault edits → commit (`docs:` prefix) and push `main` in
   `/root/Obsidian-Vault`. Code edits → feature branch in
   `/root/kovanica-protocol`, gated by
   `cargo fmt --check && cargo clippy --all-targets && cargo test`,
   then draft PR. Nothing changed? Say so explicitly and stop.
2. **Authority** — protocol/code truth lives in `/root/kovanica-protocol`
   (read its `AGENTS.md` first). This vault holds docs and snapshots;
   prefer `KovanicaDAG/myObsidianVaultDAG.md` when snapshots disagree.
3. **Registry discipline** — edit agent definitions ONLY under
   `agents/` in this vault, never the synced copies (`~/.claude/*`,
   tool config dirs). After edits run `./scripts/sync-agents.sh`,
   validate with `./agents/tests/run-tests.sh`.
4. **Determinism is sacred** — consensus output is a pure function of the
   DAG. No HashMap order, wall-clock time, or unstable sorts in consensus
   paths. Tie-breaks fall back to BlockId byte order.
5. **Consensus changes demand proof** — written rationale naming protocol
   semantics plus deterministic AND adversarial tests (wide forks beyond
   k, Byzantine parents, tie-breaks, partitions).
6. **Never invent** — do not fabricate APIs, paths, commands, or roadmap
   items. Verify before claiming: run commands, don't assume output.
7. **Trackers Precede PRs** — always explicitly verify and check off
   completed items in `TODO.md` and `ROADMAP.md` *before* committing
   and opening a Draft PR.

## Registry snapshot (as of legacy brain compile)

- Subagents (11): api-designer, code-reviewer, devops-engineer, doc-writer,
  migration-engineer, performance-engineer, protocol-dev, release-engineer,
  security-auditor, test-engineer, vault-sync
- Skills (10): api-design, code-review, doc-writer, fuzzing, migration,
  profiling, protocol-dev, security-audit, testing, vault-sync
- Commands (12): audit, benchmark, deploy, doctor, fuzz, lint, migrate,
  profile, reindex, run-tests, sync-vault, update-roadmap

*(Full agent definitions live under `agents/**` in this vault — see
`Agents.canvas` for the map. This layer intentionally does not duplicate
them; `build-god-brain.sh` should point its LAYERS at `agents/` directly
for that.)*
