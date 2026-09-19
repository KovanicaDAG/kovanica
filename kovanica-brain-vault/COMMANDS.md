# COMMANDS — commands that actually work here

Only commands verified against this repository belong here.

## Validation run — 2026-08-25

```bash
bash -n scripts/*.sh
./scripts/build-god-brain.sh --check
```

Both completed successfully before the dogfooding changes.

## Core commands

| Task | Command | Verified behavior |
|------|---------|-------------------|
| Rebuild brain | `./scripts/build-god-brain.sh` | Compiles core + memory; accepts layer paths as additional arguments |
| Freshness check | `./scripts/build-god-brain.sh --check` | Exits non-zero when compiled brain differs; currently passes |
| Shell syntax | `bash -n scripts/*.sh` | Currently passes for all 9 scripts |
| Validate Project Kit | `./scripts/validate-project-kit.sh .` | Requires 14 root surfaces plus `.agent/tasks` |
| Doctor | `./scripts/doctor.sh .` | Checks compiler, scripts, templates, frontmatter, and project kit |
| Scaffold a Project Kit | `./scripts/init-project-kit.sh <target-dir>` | Copies only missing template files and creates `.agent/tasks` |
| Create task contract | `./scripts/new-task-contract.sh <project-dir> <slug>` | Creates a task contract under `.agent/tasks` |
| Check task gate | `./scripts/agent-gate.sh <project-dir> <task-slug> [state]` | Validates task/risk/evidence requirements |
| Write checkpoint | `./scripts/checkpoint.sh <project-dir> <state> <goal> <next-action>` | Writes resumable execution state |
| Wire local tools | `./scripts/sync-brain.sh` | Updates the managed local wiring block for supported tools |

## Release discipline

1. Change source documents.
2. Rebuild `MARKDOWN.god` when a compiled source changed.
3. Run `./scripts/build-god-brain.sh --check`.
4. Run `bash -n scripts/*.sh` when scripts changed.
5. Run `./scripts/validate-project-kit.sh .` and `./scripts/doctor.sh .` for repository health.
6. Commit/push only when Git state is actually available and verified.

## Not verified here

- Any Git commit/push command — `.git/` is absent from the supplied archive.
- Any command inside the external KovanicaDAG protocol checkout — that checkout is absent.
