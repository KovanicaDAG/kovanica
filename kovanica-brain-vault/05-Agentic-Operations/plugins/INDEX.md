# Plugins Index

> **Links:** [[../INDEX]] · [[../skills/INDEX]] · [[../subagents/INDEX]] · [[../commands/INDEX]]

| Plugin | Source | Description |
|--------|--------|-------------|
| opencode-gemini-auth | npm | Gemini authentication for OpenCode |
| claude-code-tools | local | Claude Code tool integrations |
| git-tools | npm | Git workflow automation |
| test-runner | local | Custom test runners for kovanica-protocol |

## Adding a Plugin

Create `plugin-name.md` with frontmatter:
```markdown
---
name: plugin-name
source: npm | local | url
description: What this plugin does
---

# Plugin Name

Configuration, hooks, usage examples...
```

Then add a row to the table above.