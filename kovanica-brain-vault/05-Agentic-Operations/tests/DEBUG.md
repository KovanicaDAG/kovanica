# DEBUG — Registry Troubleshooting Guide

> **Links:** [[INDEX]] · [[tests/run-tests.sh]] · commands: [[commands/doctor|doctor]]

Symptom → cause → fix for the agent registry. Run `./agents/tests/run-tests.sh -v` first — it localizes most problems.

## Quick Diagnosis

```bash
./agents/tests/run-tests.sh -v   # full suite
./scripts/sync-agents.sh --dry-run   # what would change
```

## Failure Modes

### 1. Agent not appearing in a tool

| Check | Fix |
|-------|-----|
| File in `agents/subagents/` has `mode:` frontmatter | Add it |
| Listed in `tools/opencode.json` → `agent{}` | Add entry, re-run sync |
| Claude symlink exists | `./scripts/sync-agents.sh` recreates symlinks (`ln -sf`) |
| OpenCode needs restart after config change | Restart opencode session |

### 2. Command fails with "unknown agent"

Command's `agent:` references nonexistent subagent. Test T5 catches this pre-commit.
Fix: create `subagents/<name>.md` or correct the `agent:` field.

### 3. Skill never triggers

- `description:` must front-load trigger keywords — LLMs match on it
- `name:` must equal filename exactly (T4 enforces)
- Skill path registered? OpenCode reads `skills.paths` from opencode.json

### 4. Sync script exits silently mid-run

Historic bug class: `set -e` + command returning non-zero.
- Bare `[[ ]] && x` as last statement of function returns 1 when condition false
- `pipefail` + `grep` finding nothing kills `$()` assignment
- Rule: end every function with explicit `return 0`; guard greps with `(grep ... || true)`

### 5. Broken symlinks in ~/.claude

```bash
find ~/.claude/{agents,skills,commands} -maxdepth 1 -xtype l   # list broken
./scripts/sync-agents.sh                                        # recreate all
```
Note: `find -L dir -xtype l` is WRONG here — `-L` inverts semantics and lists valid links too.

### 6. Config drift between vault and tool configs

`/doctor` diffs `agents/tools/opencode.json` vs root `opencode.json`.
Golden rule: edit ONLY under `agents/`, run sync. Never hand-edit `~/.claude/*` or tool dirs.

### 7. grep `--exclude` not excluding

`--` ends option parsing: `grep pat -- dir --exclude=x` treats exclude as filename.
Order matters: `grep --exclude=x -- pat dir`.

### 8. JSON config invalid after hand edit

```bash
python3 -m json.tool agents/tools/<file>.json   # shows error location
```

## Debugging an Agent Itself

1. **Reproduce**: invoke its command directly with minimal args
2. **Isolate permissions**: temporarily set `bash: allow` to rule out permission prompts blocking subagent
3. **Check context starvation**: agent md files are loaded whole — if bloated (>300 lines), split into skill references via wiki-links
4. **Trace tool loading**: `opencode config validate` from vault root; check startup output for rejected files

## Escalation Path

If tests pass but behavior is wrong in a specific tool, the bug is in that tool's connector config (`agents/tools/*.json`) or the tool itself — diff its schema against the connector file's `docs` link.
