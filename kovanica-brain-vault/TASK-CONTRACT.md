# TASK-CONTRACT — current task agreement

## Goal

Make the MARKDOWN.god repository genuinely dogfood its own 14-surface Project Kit by replacing root scaffolding/partial data with evidence-backed information about this actual repository.

## Constraints

- Do not modify `core/*.md` doctrine in this task.
- Do not hand-edit generated `MARKDOWN.god`.
- Do not invent facts about the absent `.git` history or absent KovanicaDAG source checkout.
- Do not preserve or redistribute plaintext credentials found in the input archive.
- Keep project-specific Kovanica knowledge in `layers/KovanicaDAG.md`, not in universal core doctrine.

## Scope firewall

### GREEN — may change

- Root Project Kit documents and README.
- `.gitignore` for local-secret containment.
- `.agent/tasks/README.md` required by the validator.

### YELLOW — extra verification required

- `templates/project-kit/*.md` only if a template defect is discovered while validating dogfooding.
- `scripts/*.sh` only if validation exposes an actual script defect.
- `MARKDOWN.god` only through a deterministic rebuild after a participating source change.

### RED — explicit human authorization required

- `core/*.md`.
- Live credential material.
- Consensus/cryptographic code in downstream KovanicaDAG repositories.
- Destructive history/deployment operations.

## Non-goals

- Implementing KovanicaDAG protocol code.
- Verifying external Kovanica repositories not present in this archive.
- Changing the universal doctrine merely to make this repository look complete.
- Establishing Git history or pushing to a remote from an archive without `.git` metadata.

## Acceptance

- [x] All 14 Project Kit surfaces exist at repository root.
- [x] Each root surface contains project-specific information or a project-specific operational contract.
- [x] `.agent/tasks` exists so the validator can validate the full kit.
- [x] No live credential is shipped in the resulting artifact.
- [ ] `./scripts/validate-project-kit.sh .` passes.
- [ ] `./scripts/doctor.sh .` passes.
- [ ] `./scripts/build-god-brain.sh --check` remains clean.

## Verification

- `bash -n scripts/*.sh`
- `./scripts/validate-project-kit.sh .`
- `./scripts/doctor.sh .`
- `./scripts/build-god-brain.sh --check`

## Risk

`MEDIUM`

## Rollback / containment

- Revert root documentation changes.
- Keep `SECRETS.local.md` out of shipped/git-tracked content and rotate any credentials that were present in the original archive.

## Status

`VERIFYING`

---

*From MARKDOWN.god: Master Planning System §1 and Law 12 — Scope is a boundary.*
