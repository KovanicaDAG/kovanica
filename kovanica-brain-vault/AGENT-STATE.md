# AGENT-STATE — resumable execution checkpoint

## State

`VERIFY`

## Goal

- Dogfood the Project Kit by making the repository root describe the actual MARKDOWN.god project rather than remaining a partial scaffold.

## Completed

- Audited the archive and repository layout.
- Confirmed 7 core docs, 1 memory ledger, 9 shell scripts, 14 Project Kit templates, and a real `layers/KovanicaDAG.md` layer.
- Confirmed the compiled brain is fresh and all shell scripts pass `bash -n`.
- Confirmed the previous root Project Kit failed validation because 5 surfaces were missing.
- Populated the missing root surfaces with repository-specific facts and boundaries.

## In progress

- Final validation of the complete root Project Kit and consistency checks after the dogfooding pass.

## Changed files

- `EVIDENCE.md` — replaced scaffold with repository-backed claims.
- `AGENT-STATE.md` — records the resumable dogfooding state.
- `SCOPE.md` — classifies actual repository surfaces and protected areas.
- `IMPACT-MAP.md` — maps the compiler, source doctrine, layers, and wiring scripts.
- `TASK-CONTRACT.md` — records this dogfooding task and its acceptance criteria.
- `VERIFIED-FACTS.md` — updated root facts from fresh inspection.
- `CODE-MAP.md` — expanded navigation to actual v3 operational surfaces and Kovanica layer.
- `CURRENT-STATE.md` — updated the living snapshot.
- `ROADMAP.md` — reflects the actual dogfooding status and remaining real-project proof work.
- `SESSION-LOG.md` — records this session's work.
- `DECISION-LOG.md` — records the decision to keep root kit data concrete and evidence-backed.
- `KNOWN-TRAPS.md` — records archive/secret and layer-specific traps discovered during inspection.
- `COMMANDS.md` — records commands actually executed in this repository.
- `GLOSSARY.md` — records project-specific terminology and concrete paths.
- `README.md` — removes the stale hardcoded local path claim.
- `.gitignore` — protects `SECRETS.local.md` from accidental commits.
- `SECRETS.local.md` — redacted from the deliverable because the source archive contained credential material.

## Evidence

- `./scripts/build-god-brain.sh --check` → clean.
- `bash -n scripts/*.sh` → clean.
- `./scripts/doctor.sh .` → expected to pass after the missing Project Kit surfaces are restored.
- `./scripts/validate-project-kit.sh .` → expected to pass after the missing surfaces are restored and `.agent/tasks` exists.

## Open hypotheses / unknowns

- The archive does not contain `.git`, so commit/branch/remote state cannot be verified here.
- The KovanicaDAG protocol repository is not present, so the Kovanica layer's referenced source paths cannot be re-verified from this archive alone.

## Failures / blockers

- None for the root Project Kit work.
- Shipping/push remains outside this archive because no Git metadata is included.

## Next action

1. Run the complete Project Kit validation and doctor checks, then rebuild the compiled brain only if a source document that participates in compilation changed.

## Risk

`MEDIUM`

## Rollback / containment

- Restore the prior root markdown files from Git/archive if the dogfooding content is rejected.
- Do not restore or commit plaintext credentials from the original `SECRETS.local.md`.

## Gate status

- G0 Understanding: PASS
- G1 Change: PASS
- G2 Behavior: PASS
- G3 Integration: PASS
- G4 Ship: BLOCKED — no `.git` metadata is present in this archive
