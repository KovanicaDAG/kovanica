# Agent Workflows — Multi-Agent Compositions

> **Links:** [[INDEX]] · [[subagents/INDEX]] · [[commands/INDEX]] · [[skills/INDEX]]

Standard pipelines chaining subagents. Invoke via commands or reference in prompts.

## 1. Release Pipeline (`/release` flow)

```
test-engineer (run-tests + fuzz)
    → code-reviewer (final review gate)
        → security-auditor (/audit)
            → release-engineer (tag, build, changelog)
                → devops-engineer (deploy canary → full)
```

**Gate rules:** each stage must pass before next starts. Security Critical finding aborts pipeline.

## 2. Performance Investigation

```
performance-engineer (/benchmark baseline)
    → performance-engineer (/profile bottleneck)
        → protocol-dev (implement fix on branch)
            → test-engineer (regression + determinism tests)
                → performance-engineer (/benchmark compare ≥10% win required)
                    → code-reviewer (approve PR)
```

## 3. Consensus Change (highest scrutiny)

```
protocol-dev (RFC note in vault first)
    → doc-writer (spec diff published for review)
        → code-reviewer (consensus checklist)
            → security-auditor (invariant verification)
                → test-engineer (adversarial suite: wide forks, partitions, tie-breaks)
                    → migration-engineer (activation plan if behavior changes)
                        → devops-engineer (testnet soak 48h before mainnet)
```

## 4. API Evolution

```
api-designer (design + OpenAPI spec update)
    → code-reviewer (compat check — no breaking change without /v2)
        → protocol-dev (implementation)
            → test-engineer (integration tests vs spec)
                → api-designer (SDK regeneration + sunset headers)
```

## 5. Store Migration

```
migration-engineer (/migrate plan <name>)
    → doc-writer (runbook update)
        → migration-engineer (/migrate dry-run on prod snapshot)
            → devops-engineer (backup verification + scheduled apply)
                → test-engineer (post-migrate deep verify + genesis-sync equivalence)
```

## 6. Vault Maintenance (weekly)

```
vault-sync (/sync-vault)
    → protocol-dev (/reindex CODE_INDEX.md)
        → doc-writer (/update-roadmap)
            → vault-sync (commit + push, docs: prefix)
```

## Invocation Patterns

| Pattern | How |
|---------|-----|
| Single stage | `/benchmark ghostdag` |
| Full pipeline | "Run release pipeline" — orchestrator walks stages top-down |
| Parallel fan-out | `/audit` and `/fuzz` are independent — run concurrently |
| Abort | Any Critical/High finding stops pipeline; report stage + artifact |

## Adding a Workflow

Add section here with numbered stages, owning agent per stage, and explicit gate criteria. Keep one responsibility per stage.
