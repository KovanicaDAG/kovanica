# SCOPE — change firewall

This repository is the MARKDOWN.god doctrine itself: markdown source, memory,
project layers, generated brain, and shell tooling. Scope is therefore based on
what can alter agent behavior, generated output, or local wiring.

## GREEN

- Root Project Kit documentation (`VERIFIED-FACTS.md`, `EVIDENCE.md`, `CURRENT-STATE.md`, `AGENT-STATE.md`, `CODE-MAP.md`, `DECISION-LOG.md`, `KNOWN-TRAPS.md`, `COMMANDS.md`, `ROADMAP.md`, `SESSION-LOG.md`, `GLOSSARY.md`, `SCOPE.md`, `IMPACT-MAP.md`, `TASK-CONTRACT.md`).
- `README.md` and documentation-only metadata.
- Validation and documentation of existing scripts without changing their behavior.

## YELLOW

- `scripts/*.sh` — shared operational behavior; changes require syntax and functional verification.
- `layers/*.md` — project-specific behavior appended to the compiled brain; changes can alter agent behavior for a real downstream project.
- `templates/project-kit/*.md` — future scaffolding changes affect newly initialized projects.
- `MARKDOWN.god` — generated output; must only change as a consequence of source changes and a rebuild.

## RED

- `core/*.md` — doctrine/law changes; agent-draft only and human approval required by `AGENTS.md`.
- `memory/WHAT-WE-LEARNED.md` when promotion into core is being proposed; append-only learning evidence, with promotion requiring human approval.
- Credential/private-key/secret material, including `SECRETS.local.md`.
- Consensus or cryptographic rules inside any downstream KovanicaDAG project referenced by `layers/KovanicaDAG.md`.
- Destructive Git history rewriting or irreversible external deployment actions.

## Task-specific overrides

- Root Project Kit dogfooding is GREEN because it documents observed repository state and does not modify core doctrine.
- `SECRETS.local.md` is containment-only: it must never be copied into a shipped archive with live credentials.
- `layers/KovanicaDAG.md` remains project-layer data; do not treat its referenced external repository as verified unless that repository is available for inspection.

---

*From MARKDOWN.god: Law 12 — Scope is a boundary, not a suggestion.*
