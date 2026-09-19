# CODE-MAP — topic map

Topic → path, so navigation is a lookup instead of a quest. Paths below are
verified against this repository as of 2026-08-25.

| Topic | Path | Notes |
|-------|------|-------|
| Prime laws | `core/00-THE-LAWS.md` | Universal laws; human approval required for changes |
| Coding doctrine | `core/01-HOW-TO-CODE.md` | Mimicry, diffs, correctness, testing, debugging, gates, git, performance |
| Planning doctrine | `core/02-HOW-TO-PLAN.md` | Intake, decomposition, ordering, Definition of Done |
| Session loop | `core/03-SESSION-RITUAL.md` | BOOT → RECON → PLAN → EXECUTE → VERIFY → REVIEW → SHIP → REFLECT → IDLE |
| Learning loop | `core/04-LEARN-FROM-MISTAKES.md` | KEEP/DISCARD and promotion discipline |
| Self-extension | `core/05-SELF-EXTENSION.md` | Skills, delegates, plugins, core-law promotion |
| Personas | `core/06-PERSONAS.md` | Six operating modes |
| Memory ledger | `memory/WHAT-WE-LEARNED.md` | Append-only learned lessons |
| KovanicaDAG project layer | `layers/KovanicaDAG.md` | Project-specific rules rescued from the legacy compiled brain |
| Compiled brain | `MARKDOWN.god` | Generated artifact; never hand-edit |
| Project Kit templates | `templates/project-kit/*.md` | 14 data surfaces used by downstream projects and this root |
| Brain compiler | `scripts/build-god-brain.sh` | core + memory + optional layers → `MARKDOWN.god` |
| Tool wiring | `scripts/sync-brain.sh` | Local Claude/OpenCode wiring |
| Project scaffolder | `scripts/init-project-kit.sh` | Copies missing Project Kit surfaces without overwriting existing files |
| Project validation | `scripts/validate-project-kit.sh` | Requires 14 surfaces + `.agent/tasks` |
| Repository doctor | `scripts/doctor.sh` | Checks compiled brain, scripts, templates, and project kit |
| Task gate | `scripts/agent-gate.sh` | Validates task-contract state/risk/evidence requirements |
| Checkpoint writer | `scripts/checkpoint.sh` | Writes resumable agent state |
| Task-contract creator | `scripts/new-task-contract.sh` | Creates task contracts under a project's `.agent/tasks` |
| Commit freshness guard | `scripts/pre-commit-check.sh` | Blocks stale compiled-brain commits |
| Repository guide | `AGENTS.md` | Agent editing and shipping rules |

## Test map

| What it covers | Where | Verified command |
|----------------|-------|------------------|
| Brain freshness | `scripts/build-god-brain.sh` | `./scripts/build-god-brain.sh --check` |
| Shell syntax | all `scripts/*.sh` | `bash -n scripts/*.sh` |
| Full Project Kit | `scripts/validate-project-kit.sh` | `./scripts/validate-project-kit.sh .` |
| Repository health | `scripts/doctor.sh` | `./scripts/doctor.sh .` |

## Generated / do-not-touch

- `MARKDOWN.god` — generated from source documents; edit source files and rebuild instead.
