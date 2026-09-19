# Tools Config Index

> **Links:** [[../INDEX]] · [[../skills/INDEX]] · [[../subagents/INDEX]] · [[../commands/INDEX]]

| Config | Tool | Description |
|--------|------|-------------|
| [[opencode.json]] | OpenCode | Project config referencing shared agents/skills/commands |
| [[claude.json]] | Claude Code | References symlinked agents from `~/.claude/` |
| [[gemini.json]] | Gemini CLI | References shared agents directory |
| [[copilot.json]] | GitHub Copilot | References shared agents directory |
| [[cursor.json]] | Cursor | Rules dir `.cursor/rules`, synced via sync script |
| [[codex.json]] | Codex CLI | AGENTS.md-driven, references shared subagents |
| [[zed.json]] | Zed | Agent files from `agents/subagents/` (markdown frontmatter) |
| [[aider.json]] | Aider | Reads AGENTS.md conventions + shared agents |
| [[cline.json]] | Cline | Rules dir `.clinerules/agents` |
| [[mcp.json]] | MCP (all tools) | Shared MCP server registry: filesystem, git, github, fetch, memory, sequential-thinking |

## Test & Debug

- `../tests/run-tests.sh` — 14-test suite validating the whole registry
- `../tests/DEBUG.md` — failure-mode troubleshooting guide

## Adding a Tool Config

Create `tool-name.json` in this directory with an `agents` map pointing at
`agents/subagents/<name>.md` for each agent the tool supports, then add a row above.
Run `../tests/run-tests.sh` to verify JSON validity.