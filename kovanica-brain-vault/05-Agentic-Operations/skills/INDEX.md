# Skills Index

> **Links:** [[../INDEX]] · [[../subagents/INDEX]] · [[../plugins/INDEX]] · [[../commands/INDEX]]

| Skill | Description | Trigger Keywords |
|-------|-------------|------------------|
| [[vault-sync]] | Sync docs between kovanica-protocol and Obsidian vault | vault sync, documentation, snapshot |
| [[protocol-dev]] | kovanica-protocol development (consensus, DAG, node) | kovanica, consensus, GHOSTDAG, DAG, node |
| [[doc-writer]] | Write/update technical documentation in vault | documentation, docs, markdown, wiki-link |
| [[code-review]] | Review PRs for style, correctness, consensus safety | review, PR, consensus, safety |
| [[testing]] | Write/run tests, adversarial scenarios, property tests | test, adversarial, property, invariant |
| [[profiling]] | Benchmark and profile hot paths, find bottlenecks | benchmark, profile, flamegraph, throughput, bottleneck |
| [[security-audit]] | Audit crypto, consensus, and input handling | security, audit, vulnerability, CVE, exploit |
| [[api-design]] | Design node RPC, explorer, and wallet APIs | API, endpoint, RPC, GraphQL, OpenAPI, SDK |
| [[fuzzing]] | Fuzz deserialization and parsers with cargo-fuzz | fuzz, cargo-fuzz, crash, corpus |
| [[migration]] | Store schema migrations and protocol upgrades | migration, schema, upgrade, hard fork, rollback |

## Adding a Skill

Create `skill-name.md` with frontmatter:
```markdown
---
name: skill-name
description: One sentence covering what this skill does AND when to trigger it
---

# Skill Name

...
```

Then add a row to the table above.