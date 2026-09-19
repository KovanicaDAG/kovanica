# Migration Guide — From Tool-Specific Configs to Centralized Registry

This guide helps migrate existing agent configurations from individual tools to the centralized `agents/` registry.

## Before You Start

1. **Backup existing configs**:
   ```bash
   cp ~/.config/opencode/opencode.json ~/.config/opencode/opencode.json.backup
   cp ~/.claude/agents/* ~/.claude/agents.backup/ 2>/dev/null || true
   ```

2. **Inventory current agents**:
   - List all agents in each tool
   - Note which are shared vs tool-specific

## Migration Steps

### 1. Extract Shared Agents

For each agent that exists in multiple tools:
1. Create `agents/subagents/<name>.md` with combined best practices
2. Use the template: `cp agents/templates/agent.md agents/subagents/<name>.md`
3. Fill in frontmatter and body

### 2. Extract Shared Skills

For reusable prompt patterns:
1. Create `agents/skills/<name>.md`
2. Use template: `cp agents/templates/skill.md agents/skills/<name>.md`
3. Add trigger keywords

### 3. Extract Shared Commands

For slash commands / custom commands:
1. Create `agents/commands/<name>.md`
2. Use template: `cp agents/templates/command.md agents/commands/<name>.md`

### 4. Update Tool Configs

Replace inline agent definitions with references:

**OpenCode** (`opencode.json`):
```json
{
  "skills": { "paths": ["agents/skills"] },
  "agent": { "my-agent": { "file": "agents/subagents/my-agent.md" } },
  "command": { "my-cmd": { "file": "agents/commands/my-cmd.md" } }
}
```

**Claude Code** — Use symlinks (already set up):
```bash
ln -sf /root/Obsidian-Vault/agents/subagents/* ~/.claude/agents/
ln -sf /root/Obsidian-Vault/agents/skills/* ~/.claude/skills/
ln -sf /root/Obsidian-Vault/agents/commands/* ~/.claude/commands/
```

### 5. Run Sync & Validate

```bash
cd /root/Obsidian-Vault
./scripts/sync-agents.sh
./scripts/health-check.sh
```

### 6. Test Each Tool

- **OpenCode**: Restart, verify agents appear in picker
- **Claude Code**: `/agents` should list all synced agents
- **Gemini/Copilot**: Verify config loads correctly

### 7. Commit & Push

```bash
git add -A
git commit -m "docs: migrate agents to centralized registry"
git push origin main
```

## Tool-Specific Notes

### OpenCode
- Project config (`opencode.json`) takes precedence over global
- Agents defined in `agent:` object override file-based ones
- Skills auto-loaded from `skills.paths` directories

### Claude Code
- Agents loaded from `~/.claude/agents/*.md`
- Skills from `~/.claude/skills/*/SKILL.md` (note: folder per skill)
- Commands from `~/.claude/commands/*.md`
- Frontmatter: `description`, `mode`, `model`, `permission`

### Gemini / Copilot
- Config location varies; check tool documentation
- Use `agents/tools/gemini.json` or `copilot.json` as reference

## Rollback

If issues arise:
```bash
# Restore OpenCode
cp ~/.config/opencode/opencode.json.backup ~/.config/opencode/opencode.json

# Restore Claude
rm -rf ~/.claude/agents ~/.claude/skills ~/.claude/commands
cp -r ~/.claude/agents.backup ~/.claude/agents
# etc.
```

## Best Practices Going Forward

1. **Single source of truth** — Edit only in `agents/`, never in tool configs directly
2. **Run sync after changes** — `./scripts/sync-agents.sh`
3. **Health check regularly** — `./scripts/health-check.sh`
4. **Version control everything** — All changes tracked in this vault