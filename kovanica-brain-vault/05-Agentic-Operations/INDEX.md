# Agents Registry — Centralized Agent Configurations

> **Links:** [[skills/INDEX]] · [[plugins/INDEX]] · [[subagents/INDEX]] · [[commands/INDEX]] · [[tools/INDEX]] · [[templates/INDEX]] · [[WORKFLOWS]]

This directory stores all agent configurations, skills, plugins, and commands as Markdown files — portable across Claude, Grok, Gemini, Copilot, OpenCode, and other tools.

## Structure

```
agents/
├── INDEX.md                 # This file
├── MARKDOWN.god             # Compiled single-file brain (generated)
├── WORKFLOWS.md             # Multi-agent pipeline compositions
├── skills/                  # Reusable skill definitions
│   ├── INDEX.md
│   ├── vault-sync.md
│   ├── protocol-dev.md
│   ├── doc-writer.md
│   ├── code-review.md
│   ├── testing.md
│   └── ...
├── plugins/                 # Tool plugin configurations
│   ├── INDEX.md
│   ├── opencode-gemini-auth.md
│   ├── claude-code-tools.md
│   └── ...
├── subagents/               # Subagent definitions (OpenCode, Claude, etc.)
│   ├── INDEX.md
│   ├── vault-sync.md
│   ├── protocol-dev.md
│   ├── doc-writer.md
│   ├── code-reviewer.md
│   ├── test-engineer.md
│   ├── release-engineer.md
│   └── ...
├── commands/                # Slash commands / custom commands
│   ├── INDEX.md
│   ├── sync-vault.md
│   ├── run-tests.md
│   ├── lint.md
│   ├── deploy.md
│   ├── update-roadmap.md
│   ├── reindex.md
│   └── ...
├── tools/                   # Tool-specific configs (reference shared agents)
│   ├── INDEX.md
│   ├── opencode.json
│   ├── claude.json
│   ├── gemini.json
│   └── copilot.json
├── templates/               # Scaffolding for new agents/skills/commands
│   ├── INDEX.md
│   ├── agent.md
│   ├── skill.md
│   └── command.md
└── scripts/                 # Automation scripts (in repo root /scripts/)
    └── sync-agents.sh
```

## Usage

Each `.md` file uses frontmatter for metadata and Markdown body for the agent prompt/instructions.

### Skill Format
```markdown
---
name: skill-name
description: One sentence covering what this skill does AND when to trigger it
---

# Skill Name

(skill body: instructions, examples, references)
```

### Subagent Format (OpenCode / Claude compatible)
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

### Command Format
```markdown
---
description: One sentence describing what the command does
agent: vault-sync
---

(command body — the prompt to run, with $ARGUMENTS for user input)
```

## Compiled Brain (MARKDOWN.god)

`agents/MARKDOWN.god` is the **generated**, single-file compilation of the
entire registry — workflows, every subagent, skill, command, plugin, and
template — with frontmatter preserved as yaml fences. Drop it into any tool
that accepts one markdown context file, or reference it as a whole-brain
document.

- Never edit by hand; it is rebuilt by `scripts/build-god-brain.sh`
- `sync-agents.sh` rebuilds it automatically on every sync
- Verify freshness: `./scripts/build-god-brain.sh --check`

## Sync Workflow

### 1. Edit in Vault
Make all changes to agent/skill/command files under `agents/` in this vault.

### 2. Run Sync Script
```bash
cd /root/Obsidian-Vault
./scripts/sync-agents.sh          # Sync to all tools
./scripts/sync-agents.sh --dry-run  # Preview changes
```

The script:
- Validates frontmatter on all agent files
- Rebuilds `agents/MARKDOWN.god` from the registry
- Copies `opencode.json` to project root
- Verifies Claude Code symlinks (`~/.claude/agents/`, `skills/`, `commands/`)
- Copies Gemini/Copilot configs to their config directories
- Validates OpenCode config (if `opencode` CLI available)

### 3. Verify in Each Tool
- **OpenCode**: Restart to reload config (`opencode.json` in project root)
- **Claude Code**: Agents auto-loaded from `~/.claude/agents/`
- **Gemini/Copilot**: Reference their respective config files

### 4. Commit & Push
```bash
git add -A && git commit -m "docs: update agents registry (<what changed>)"
git push origin main
```

## Finish Line Convention

Every subagent, skill, and command ends with a **Finish Line** section: a task
is complete only when its output is shipped — vault edits committed (`docs:`)
and pushed to `main`; kovanica-protocol code changes on a feature branch +
draft PR per that repo's `AGENTS.md`; read-only tasks explicitly report "no
changes". Enforced by `agents/tests/run-tests.sh` (T15).

## Tool Setup (One-Time)

### OpenCode
Project config at `opencode.json` reads from `agents/skills`, `agents/subagents`, `agents/commands` automatically.

### Claude Code
```bash
mkdir -p ~/.claude/agents ~/.claude/skills ~/.claude/commands
ln -sf /root/Obsidian-Vault/agents/subagents/* ~/.claude/agents/
ln -sf /root/Obsidian-Vault/agents/skills/* ~/.claude/skills/
ln -sf /root/Obsidian-Vault/agents/commands/* ~/.claude/commands/
```

### Gemini / Copilot / Grok
Copy tool config from `agents/tools/` to their config directory, or set config path to this vault.

## Source of Truth

This vault is the **single source of truth** for all agent configurations. Tool-specific configs in `agents/tools/` reference these shared files rather than inlining everything.