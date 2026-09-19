# IMPACT-MAP — dependency and blast-radius map

| Component | Depends on | Consumed by | Public API? | Risk | Tests / verification |
|-----------|------------|-------------|-------------|------|----------------------|
| `core/*.md` | doctrine edits + human approval | `MARKDOWN.god` compiler | yes, behavioral | critical | `build-god-brain.sh --check`, frontmatter checks |
| `memory/WHAT-WE-LEARNED.md` | session evidence | `MARKDOWN.god` compiler | yes, behavioral | high | compiler freshness check |
| `layers/KovanicaDAG.md` | Kovanica-specific project knowledge | layered `MARKDOWN.god` builds | yes, behavioral | high | source inspection + layered build |
| `MARKDOWN.god` | `core/` + `memory/` + optional layers | AI agents/tools loading the brain | yes, generated artifact | high | `./scripts/build-god-brain.sh --check` |
| `scripts/build-god-brain.sh` | core/memory/layer filesystem | generated brain | operational | high | `bash -n`, `--check`, functional build |
| `scripts/sync-brain.sh` | local tool configuration + compiled brain | Claude/OpenCode wiring on a host | operational | high | script syntax + scratch-home test |
| `scripts/init-project-kit.sh` | `templates/project-kit/*` | downstream project roots | operational | medium | scaffold + `validate-project-kit.sh` |
| `scripts/validate-project-kit.sh` | 14 required surfaces + `.agent/tasks` | project health checks | operational | medium | direct validation run |
| `scripts/doctor.sh` | compiler, scripts, templates, project kit | repository health status | operational | medium | direct doctor run |
| `templates/project-kit/*.md` | doctrine requirements | newly initialized projects | indirect | medium | template inspection + scaffold validation |
| Root Project Kit | actual repository state | future agent sessions in this repo | yes, operational context | medium | `validate-project-kit.sh`, `doctor.sh` |

## Change checklist

- [x] Direct dependencies inspected
- [x] Direct dependents inspected
- [x] Compatibility impact considered
- [x] Relevant verification commands identified
- [x] Rollback/containment identified for high-risk surfaces

---

*From MARKDOWN.god: risk-first planning and verification gates.*
