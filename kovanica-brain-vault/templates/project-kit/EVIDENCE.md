# EVIDENCE — claim ledger

Material claims must be traceable to evidence. Use one row per claim.

| Claim | Status | Evidence | Verified at | Stability |
|-------|--------|----------|-------------|-----------|
| <atomic claim> | VERIFIED / INFERRED / HYPOTHESIS / UNKNOWN | `<file>`, `<command>`, `<test>`, `<commit>` | YYYY-MM-DD | high/medium/low |

## Rules

- `VERIFIED` requires reproducible evidence.
- `INFERRED` is a reasoned conclusion; never present it as observed fact.
- `HYPOTHESIS` is a testable guess.
- `UNKNOWN` means evidence is missing or conflicting.
- Volatile facts must be re-verified when their verification date is stale.

---

*From MARKDOWN.god: Law 11 — Evidence before confidence.*
