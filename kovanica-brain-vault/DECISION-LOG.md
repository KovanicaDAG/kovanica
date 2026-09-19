# DECISION-LOG — append-only log

| Date | Decision | Why | Rejected alternatives |
|------|----------|-----|-----------------------|
| 2026-08-24 | MARKDOWN.god carries its own Project Kit | Dogfooding is the stated proof that the doctrine works on itself | Keeping the doctrine repo exempt from its own kit |
| 2026-08-24 | Core doctrine remains human-approval-only | Core laws are the tenure track; uncontrolled self-modification would undermine the authority model | Letting agents directly mutate `core/*.md` |
| 2026-08-24 | Compiler/wiring scripts derive paths from their own location | The vault must work after cloning to another path | Hardcoded absolute paths |
| 2026-08-24 | Shell file enumeration is space-safe | Real filesystem names can contain spaces | `for f in $(ls ...)` |
| 2026-08-25 | Root Project Kit must contain observed project data, not template placeholders | The purpose of dogfooding is to test whether the kit describes the project well enough for a cold-start agent | Leaving five surfaces absent or generic because templates already exist |
| 2026-08-25 | KovanicaDAG-specific facts stay in `layers/KovanicaDAG.md` | Core doctrine must remain project-agnostic; downstream facts need an explicit boundary | Baking Kovanica paths/rules into `core/*.md` |
| 2026-08-25 | Credential material from the input archive is excluded from the deliverable | Plaintext secrets are not project documentation and create unnecessary security exposure | Republishing `SECRETS.local.md` unchanged |
