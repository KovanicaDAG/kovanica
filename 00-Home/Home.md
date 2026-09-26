---
title: Kovanica Protocol Vault
synced: 2026-09-26
generated_by: scripts/build-vault.py
---

# Kovanica Protocol Vault

Generated from the `kovanica-protocol` monorepo by `scripts/build-vault.py`
(`./scripts/build-vault.sh` is the wrapper). `90-Notes/` is yours — the build
never reads, writes, or deletes it.

## Browse

- [[00-Home/10-Protocol|Protocol]] — 17 notes
- [[00-Home/20-Network|Network]] — 15 notes
- [[00-Home/30-Operations|Operations]] — 18 notes
- [[00-Home/40-Node|Node]] — 10 notes
- [[00-Home/50-Components|Components]] — 19 notes
- [[00-Home/60-Planning|Planning]] — 52 notes
- [[00-Home/70-Policy|Policy]] — 7 notes
- [[00-Home/80-Repo-Doctrine|Repo-Doctrine]] — 7 notes
- [[00-Home/99-Unfiled|Unfiled]] — 2 notes

## How sync stays honest

- **Desktop** — `./scripts/build-vault.sh` regenerates every category from the
  repo. The repo is the source of truth; the vault is a projection of it.
- **Phone** — this vault is its own git repo with its own remote, so any
  Obsidian git plugin can pull and push.
- **Never lose notes** — hand-written notes live in `90-Notes/`. Re-running the
  build leaves that tree untouched.
- **Trace any doc** — every vendored note carries `source:` frontmatter with its
  repo path. Nothing in the vault is hand-maintained except `90-Notes/`.

## When a doc lands in the wrong place

Add or tighten a glob in the `CATEGORIES` list in
`scripts/build-vault.py` and re-run. Notes with no matching rule land in
[[00-Home/99-Unfiled|99-Unfiled]] so they are never silently dropped.
