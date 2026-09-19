# MARKDOWN.god agent guide

Single-purpose repo: the portable doctrine loaded into AI coding agents.
Full document map and how everything connects: [README.md](README.md).

## Rules

1. Edit only `core/*.md`, `templates/**/*.md`, `README.md`, this file, and
   `scripts/`.
2. `MARKDOWN.god` is GENERATED. Never hand-edit it; run
   `./scripts/build-god-brain.sh` after any core change.
3. After edits run `./scripts/sync-brain.sh` so the local tool configs
   stay wired.
4. Core doctrine must stay project-agnostic — no repo names, no paths to
   specific projects, no tool lock-in. Project knowledge belongs in layers
   appended at build time.
5. Keep each core file frontmatter-valid (`name:` matches filename stem, case-insensitively,
   one-line `description:`).
6. Agents may write ONLY to `memory/` (append-only lesson entries).
   Changes to `core/` require the human.
7. End-of-session ritual: reflect → append kept lessons → rebuild →
   freshness-check → commit (`brain:` prefix) → push if remote exists,
   otherwise commit locally and say so.
8. Docs and doctrine name no tools, vendors, or specific agents — all are
   equal. Only `scripts/` may reference concrete tool paths, because wiring
   requires them.
9. Commit style: imperative subjects, prefix `brain:` (e.g.
   `brain: add debugging protocol sweep step`). Pushes go straight to main.
