# Commands Index

> **Links:** [[../INDEX]] · [[../skills/INDEX]] · [[../subagents/INDEX]] · [[../plugins/INDEX]]

| Command | Agent | Description |
|---------|-------|-------------|
| [[sync-vault]] | vault-sync | Sync vault snapshots from kovanica-protocol |
| [[run-tests]] | test-engineer | Run cargo test with common filters |
| [[lint]] | code-reviewer | Run cargo fmt --check && cargo clippy --all-targets |
| [[deploy]] | release-engineer | Deploy to testnet (requires DEPLOY_ENABLED) |
| [[update-roadmap]] | doc-writer | Update ROADMAP.md with stage progress |
| [[reindex]] | protocol-dev | Rebuild CODE_INDEX.md from source tree |
| [[benchmark]] | performance-engineer | Run criterion benchmarks, compare vs baseline |
| [[profile]] | performance-engineer | Profile binary with flamegraph/perf, find bottlenecks |
| [[fuzz]] | test-engineer | Run cargo-fuzz targets on parsers |
| [[audit]] | security-auditor | Security audit battery: deps, unsafe, secrets |
| [[migrate]] | migration-engineer | Plan/dry-run/apply store migrations & upgrades |
| [[doctor]] | vault-sync | Health-check registry across all tools (drift, symlinks) |

## Adding a Command

Create `command-name.md` with frontmatter:
```markdown
---
description: One sentence describing what the command does
agent: vault-sync
---

(command body — the prompt to run, with $ARGUMENTS for user input)
```

Then add a row to the table above.