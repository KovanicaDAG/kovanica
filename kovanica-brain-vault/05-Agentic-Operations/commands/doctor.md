---
description: Health-check the agent registry across all tools (symlinks, configs, drift)
agent: vault-sync
---

Diagnose the agent registry setup end-to-end. Read-only.

Run each check and report PASS/FAIL with fix hint:

```bash
# 1. Registry integrity (validation logic from sync script)
./scripts/sync-agents.sh --dry-run

# 2. OpenCode config exists at project root and matches registry source
diff <(cat agents/tools/opencode.json) opencode.json && echo "opencode: in sync" || echo "opencode: DRIFT — run ./scripts/sync-agents.sh"

# 3. Claude Code symlinks resolve
for d in ~/.claude/agents ~/.claude/skills ~/.claude/commands; do
  find -L "$d" -xtype l 2>/dev/null | grep . && echo "$d: BROKEN symlink" || echo "$d: OK"
done

# 4. New registry files missing symlinks in Claude dirs
comm -13 <(ls ~/.claude/agents 2>/dev/null | sort) <(ls agents/subagents/*.md | xargs -n1 basename | sort)

# 5. Gemini/Copilot config presence
[[ -f ~/.config/gemini/config.json ]] || echo "gemini config missing"
[[ -f ~/.config/github-copilot/config.json ]] || echo "copilot config missing"

# 6. Path sanity — referenced repo exists
[[ -d /root/kovanica-protocol ]] || echo "WARN: kovanica-protocol path not found on this machine"

# 7. Git state clean?
git status --porcelain agents/ | head -5
```

$ARGUMENTS — optional: `--fix` to apply obvious fixes (re-run sync script, recreate broken symlinks)

Output a summary table: check / status / action needed.

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
