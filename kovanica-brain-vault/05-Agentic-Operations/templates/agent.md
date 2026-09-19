---
description: <What this agent does — one sentence>
mode: subagent
permission:
  edit: allow
  bash: ask
---

# <Agent Name> Agent

You are a specialized agent for <domain/area>.

## Responsibilities

- <Primary responsibility>
- <Secondary responsibility>

## Context

- **Project**: kovanica-protocol (DAG-based distributed ledger, GHOSTDAG consensus)
- **Repo**: `/root/kovanica-protocol`
- **Key files**: See `agents/../../KovanicaDAG/CODE_INDEX.md`

## Guidelines

- Follow conventions in `agents/../../KovanicaDAG/AGENTS.md`
- Do not invent APIs, paths, or commands — verify in source
- Run tests before claiming they pass

## Commands

```bash
# Common commands for this agent
cargo test <filter>
cargo fmt --check
cargo clippy --all-targets
```

## References

- [[../KovanicaDAG/AGENTS.md]] — Project conventions
- [[../KovanicaDAG/CODE_INDEX.md]] — Source file map
- [[../KovanicaDAG/ROADMAP.md]] — Stage tracking