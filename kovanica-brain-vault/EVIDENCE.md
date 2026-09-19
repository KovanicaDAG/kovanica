# EVIDENCE — claim ledger

Material claims about this repository are recorded here with reproducible evidence.

| Claim                                                                                  | Status   | Evidence                                                                                            | Verified at | Stability |
| -------------------------------------------------------------------------------------- | -------- | --------------------------------------------------------------------------------------------------- | ----------- | --------- |
| This repository is a markdown + shell implementation of an AI-agent operating doctrine | VERIFIED | `README.md`, `core/*.md`, `scripts/*.sh`                                                            | 2026-08-25  | high      |
| The doctrine has 7 core documents                                                      | VERIFIED | `find core -maxdepth 1 -type f -name '*.md' \| wc -l` → `7`                                         | 2026-08-25  | high      |
| The memory ledger currently contains 1 markdown file                                   | VERIFIED | `find memory -maxdepth 1 -type f -name '*.md' \| wc -l` → `1`                                       | 2026-08-25  | high      |
| The repository has 9 operational shell scripts                                         | VERIFIED | `find scripts -maxdepth 1 -type f -name '*.sh' \| wc -l` → `9`                                      | 2026-08-25  | high      |
| The Project Kit defines 14 data-surface templates                                      | VERIFIED | `find templates/project-kit -maxdepth 1 -type f -name '*.md' \| wc -l` → `14`                       | 2026-08-25  | high      |
| `MARKDOWN.god` is generated from core + memory and supports project layers             | VERIFIED | `scripts/build-god-brain.sh`; successful `--check`                                                  | 2026-08-25  | high      |
| The compiled brain is currently fresh                                                  | VERIFIED | `./scripts/build-god-brain.sh --check` → `MARKDOWN.god is up to date (7 core docs)`                 | 2026-08-25  | medium    |
| All repository shell scripts pass Bash syntax checking                                 | VERIFIED | `bash -n scripts/*.sh` → exit 0                                                                     | 2026-08-25  | high      |
| The root Project Kit was previously incomplete                                         | VERIFIED | `./scripts/validate-project-kit.sh .` reported 5 missing surfaces                                   | 2026-08-25  | high      |
| `layers/KovanicaDAG.md` is a real project-specific layer present in this repository    | VERIFIED | file exists and contains KovanicaDAG-specific rules/snapshot                                        | 2026-08-25  | high      |
| KovanicaDAG protocol source is not present in this archive                             | VERIFIED | root inventory contains no `kovanica-protocol` checkout; only `layers/KovanicaDAG.md` references it | 2026-08-25  | high      |
| No external network service is required by the repository's shell scripts              | VERIFIED | source inspection of all 9 scripts; no network commands or service endpoints                        | 2026-08-25  | medium    |
| Git history/remote state is available in this archive                                  | UNKNOWN  | archive contains no `.git/` directory                                                               | 2026-08-25  | high      |

## Evidence rules

- `VERIFIED` means a claim was observed from a file, command, or test in this repository.
- `INFERRED` means a reasoned conclusion and must not be presented as observed fact.
- `UNKNOWN` means the archive does not contain enough evidence.
- Re-verify volatile claims before relying on them in a later session.
