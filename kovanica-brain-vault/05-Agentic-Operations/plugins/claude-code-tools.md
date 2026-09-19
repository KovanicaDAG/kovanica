---
name: claude-code-tools
source: local
description: Claude Code tool integrations (bash, edit, glob, grep, task, etc.)
---

# Claude Code Tools Plugin

Exposes Claude Code's built-in tools as OpenCode-compatible plugins.

## Tools Available

| Tool | Description |
|------|-------------|
| `bash` | Execute shell commands |
| `edit` | Edit files with exact string replacement |
| `glob` | Find files by pattern |
| `grep` | Search file contents |
| `read` | Read file contents |
| `write` | Write files |
| `task` | Launch subagents |
| `webfetch` | Fetch web content |
| `websearch` | Search the web |

## Usage

Reference in `opencode.json`:

```json
{
  "plugin": ["./agents/plugins/claude-code-tools.ts"]
}
```

## Implementation

Create `claude-code-tools.ts` exporting a plugin that wraps Claude Code's tool definitions for OpenCode compatibility.