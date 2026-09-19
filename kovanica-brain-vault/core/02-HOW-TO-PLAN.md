---
name: HOW-TO-PLAN
description: Master planning system — intake contract, decomposition, risk-first ordering, execution loop, contingency design, and definition of done for any non-trivial task
---

# Master Planning System

## 1. Intake contract

Before planning anything non-trivial:

- **Goal** — restate it in one sentence. If you can't, you don't understand
  the task yet.
- **Constraints** — time, stack, compat, budget, "don't touch X".
- **Done** — observable success criteria ("`cargo test` green", "page loads
  < 200ms"), never vague ("works better").

If the goal is ambiguous AND a wrong guess is expensive: ask now, with a
proposed default so silence = consent.

## 2. Recon before plan

Never plan against an imagined codebase. Ground every plan in recon:
the project's data docs first (VERIFIED-FACTS, CODE-MAP, CURRENT-STATE, KNOWN-TRAPS,
COMMANDS), then relevant modules, existing patterns for similar work, test
infrastructure, and prior art in the repo. Ten minutes of reading saves
ten hours of rewriting.

## 3. Decomposition

- Milestones → tasks. Each task:
  - single responsibility,
  - independently verifiable,
  - sized to one focused sitting,
  - paired with its **verification command** written down at planning time.
- A task that can't state its verification isn't ready to be a task.

## 4. Ordering

- **Risk first**: tackle the most uncertain piece early via a timeboxed
  spike (15–30 min) to convert unknowns into facts before committing
  architecture.
- Respect real dependencies; break fake ones (interfaces, stubs).
- Identify the critical path; flag what can run in parallel.
- Front-load integration risk; leave mechanical work for last.

## 5. Plan artifact

- Track multi-step work as a live todo list — created before starting,
  updated as states change, statuses truthful.
- New discoveries become todos, not drift. The plan absorbs reality;
  reality doesn't get ignored.

## 6. Execution loop

```
pick task → smallest complete slice → verify → next task
              ↑                              │
              └────── adjust plan ←──────────┘
```

- Smallest slice that stands alone beats big-bang assembly.
- When verification fails, fix understanding before fixing code.
- Replanning on evidence is strength; grinding on a broken plan is not.

## 7. Contingency design

Risky changes ship with an exit:

- migrations → reversible (down-path or compatible two-phase),
- behavior changes → feature flag or staged rollout,
- refactors → branch + easy diff review,
- anything touching data → backup/restore verified first.

Prefer additive over destructive; make rollback cheaper than panic.

## 8. Definition of Done checklist

- [ ] All gates green (lint, types, tests) — actually ran, not assumed
- [ ] New behavior covered by tests; fixed bugs by regressions
- [ ] Docs touched if behavior/interface changed
- [ ] Work shipped per project rules (commit/PR/deploy) or explicitly
      reported as pending with reason
- [ ] Close-out report: what shipped, how verified, what remains

## 9. Planning anti-patterns

- Planning forever without executing (plans are perishable).
- Executing without updating the plan (drift).
- Hidden scope creep — new ideas get noted, then consciously accepted or
  deferred, never smuggled in.
- Declaring victory without running gates.
- One giant task named "implement feature" — that's a milestone, not a task.

See also: *Session Operating Loop* (bootstrap loads project data docs that
ground recon; close-out updates them) and *Coding Mastery Doctrine §7*
(gates are part of every Definition of Done).

---

*Brain map: [The Laws](00-THE-LAWS.md) · [Coding Mastery Doctrine](01-HOW-TO-CODE.md) ·
**Master Planning System** ·
[Session Operating Loop](03-SESSION-RITUAL.md) ·
[Memory & Learning](04-LEARN-FROM-MISTAKES.md)*
