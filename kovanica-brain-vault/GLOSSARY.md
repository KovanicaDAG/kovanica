# GLOSSARY — the project's language

| Term | Meaning | Where it appears |
|------|---------|------------------|
| Brain | The generated `MARKDOWN.god` file loaded by an AI agent | `MARKDOWN.god`, `README.md` |
| Core | Universal doctrine under `core/`; human approval required for law changes | `AGENTS.md`, `core/` |
| Layer | Project-specific markdown appended during compilation | `layers/`, `scripts/build-god-brain.sh` |
| Project Kit | The 14 operational data surfaces a project carries for agent continuity | `templates/project-kit/`, root docs |
| Dogfooding | Applying MARKDOWN.god's own operating system to the MARKDOWN.god repository itself | `ROADMAP.md`, `DECISION-LOG.md` |
| Evidence | A reproducible file/command/test backing a material claim | `EVIDENCE.md`, `VERIFIED-FACTS.md` |
| Freshness check | `build-god-brain.sh --check`, which detects generated-brain drift | `COMMANDS.md`, `scripts/` |
| Task Contract | Explicit goal/scope/risk/acceptance agreement for non-trivial work | `TASK-CONTRACT.md`, `.agent/tasks/` |
| Agent State | Resumable checkpoint describing current execution state | `AGENT-STATE.md`, `scripts/checkpoint.sh` |
| Gate | An explicit checkpoint in the v3 state machine (G0–G4) | `core/`, `AGENT-STATE.md`, `scripts/agent-gate.sh` |
| KovanicaDAG layer | The concrete downstream project knowledge stored in `layers/KovanicaDAG.md` | `layers/KovanicaDAG.md` |
| Root kit | The MARKDOWN.god repository's own 14 Project Kit surfaces | repository root |
