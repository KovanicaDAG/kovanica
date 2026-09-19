# IMPACT-MAP — dependency and blast-radius map

Use this map to answer “what else can this change affect?” before editing a
shared component. Keep entries concrete and update them when architecture moves.

| Component | Depends on | Consumed by | Public API? | Risk | Tests |
|-----------|------------|-------------|-------------|------|-------|
| `<component>` | `<component>` | `<component>` | yes/no | low/med/high/critical | `<path>` |

## Change checklist

- [ ] Direct dependents inspected
- [ ] Direct dependencies inspected
- [ ] Compatibility impact considered
- [ ] Relevant tests identified
- [ ] Rollback/containment identified for HIGH/CRITICAL changes

---

*From MARKDOWN.god: risk-first planning and verification gates.*
