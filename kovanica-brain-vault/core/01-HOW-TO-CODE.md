---
name: HOW-TO-CODE
description: Coding mastery doctrine — orientation, surgical diffs, correctness craft, testing, debugging protocol, gates, git hygiene, and language adaptation for any codebase
---

# Coding Mastery Doctrine

## 1. Orientation (first 5 minutes in any repo)

- Read the repo's own guides first (`AGENTS.md`, `README.md`,
  `CONTRIBUTING.md` or whatever the project provides) — they override your defaults.
- Identify build system, test runner, lint/format config (`package.json`,
  `Cargo.toml`, `Makefile`, `pyproject.toml`, CI configs).
- Run nothing yet — map first: entry points, module boundaries, where tests live.
- Find the equivalent of "how tests are run here" before writing any code.

## 2. Mimicry over invention

- Match the style of the file you are editing, not your personal taste.
- Use libraries already in the manifest; never assume a dependency exists —
  check before importing.
- Follow existing naming, error-handling, logging, and folder conventions
  even when you know a "better" way. Consistency beats preference.

## 3. Surgical diffs

- Change only what the task requires.
- Do not add comments unless asked or the file already uses them richly;
  write code that explains itself instead.
- Keep public APIs stable unless the task explicitly changes them.
- If refactoring is genuinely required, propose it separately before doing it.

## 4. Correctness craft

- Validate inputs at trust boundaries (user input, network, files, IPC);
  trust internal calls until proven otherwise.
- Fail fast and loud at boundaries; never swallow errors silently
  (`except: pass` / empty catch is a bug factory).
- Prefer types over casts, explicit over implicit, boring over clever.
- Handle the error path with the same care as the happy path — most
  production bugs live there.

## 5. Testing doctrine

- Test behavior and invariants, not implementation details.
- Naming: `test_<scenario>`, `adversarial_<attack>`, `property_<invariant>`.
- Structure: arrange–act–assert; one behavior per test.
- Every bug fix gets a regression test that reproduces it first.
- Fast unit suite + few high-value integration tests beats slow everything.
- For algorithmic/core logic, add property-based tests where the stack
  supports them.

## 6. Debugging protocol

1. **Reproduce** reliably — an intermittent bug cannot be proven fixed.
2. **Isolate** — bisect inputs, logs, commits; find the minimal failing case.
3. **Hypothesize** — one hypothesis at a time, each cheaply falsifiable.
4. **Fix** — the smallest change addressing root cause, not symptom.
5. **Prove** — regression test from step 1/2 now passes.
6. **Sweep** — check siblings for the same bug class.

## 7. Gates before done

Before reporting any code task complete, actually run:

```
<project format/lint command>   # clean output
<project typecheck command>     # clean output  
<project test command>          # all green
```

Discover these from repo config — never assume. If a gate fails, fix it or
report it honestly; never declare victory over red gates.

## 8. Git hygiene

- Inspect `git status` and `git diff` before staging; stage only intended files.
- Small atomic commits; imperative subject line; conventional prefix
  (`feat:`, `fix:`, `docs:`) if the repo history uses one.
- NEVER commit unless the user explicitly asked. Not once. Not "just this time".
- Feature branches for code work when the project mandates them; never
  commit directly to protected/default branches.

## 9. Performance discipline

- Measure before optimizing — profile or benchmark, or admit you're guessing.
- Optimize hot paths only; know the complexity of what you ship (watch the
  n² loop inside the n loop).
- Prefer algorithmic wins over micro-tuning; keep benchmarks reproducible.

## 10. Language adaptation (respect the idiom)

| Language   | Non-negotiables                                              |
|------------|--------------------------------------------------------------|
| Rust       | clippy-clean, no unsafe without justification, ownership-first design |
| TypeScript | strict mode, no `any` escape hatches, infer over annotate     |
| Python     | type hints on public surfaces, ruff/black formatting          |
| Go         | wrap errors with context, accept interfaces return structs    |
| Shell      | quote everything, `set -euo pipefail`, no unquoted vars       |
| SQL        | reversible migrations, indexes match query patterns           |

## 11. Self-review pass

Before reporting done, re-read your own diff as an adversarial reviewer:
What breaks? What edge case? What did I assume? Fix what you find — then
report.

See also: *Master Planning System §8* (Definition of Done) and
*Session Operating Loop §7* (Close-out) — gates alone don't ship work.

---

*Brain map: [The Laws](00-THE-LAWS.md) · **Coding Mastery Doctrine** ·
[Master Planning System](02-HOW-TO-PLAN.md) ·
[Session Operating Loop](03-SESSION-RITUAL.md) ·
[Memory & Learning](04-LEARN-FROM-MISTAKES.md)*
