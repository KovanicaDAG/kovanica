# Subagents Index

> **Links:** [[../INDEX]] · [[../skills/INDEX]] · [[../plugins/INDEX]] · [[../commands/INDEX]]

| Subagent | Mode | Description | Permissions |
|----------|------|-------------|-------------|
| [[vault-sync]] | primary | Sync docs between kovanica-protocol and Obsidian vault | edit: allow, bash: allow |
| [[protocol-dev]] | subagent | kovanica-protocol development (consensus, DAG, node, CLI) | edit: allow, bash: allow |
| [[doc-writer]] | subagent | Write/update technical documentation in vault | edit: allow, bash: ask |
| [[code-reviewer]] | subagent | Review PRs for consensus safety, style, correctness | edit: deny, bash: ask |
| [[test-engineer]] | subagent | Write/run adversarial tests, property tests, invariants | edit: allow, bash: allow |
| [[release-engineer]] | subagent | Manage releases, versioning, deployments | edit: allow, bash: allow |
| [[performance-engineer]] | subagent | Benchmarking, profiling, optimization of hot paths | edit: allow, bash: allow |
| [[security-auditor]] | subagent | Security auditing, crypto review, threat modeling | edit: deny, bash: ask |
| [[devops-engineer]] | subagent | CI/CD, infrastructure, monitoring, observability | edit: allow, bash: allow |
| [[api-designer]] | subagent | API design — REST/GraphQL/WebSocket, OpenAPI, SDKs | edit: allow, bash: ask |
| [[migration-engineer]] | subagent | Store migrations, protocol upgrades, state transitions | edit: allow, bash: allow |

## Adding a Subagent

Create `agent-name.md` with frontmatter:
```markdown
---
description: What this agent does
mode: subagent
permission:
  edit: allow
  bash: ask
---

You are a specialized agent for...
```

Then add a row to the table above.