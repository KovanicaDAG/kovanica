# Global agent guidance (preserved from ~/.claude/CLAUDE.md, removed 2026-09-05)

> This note preserves the useful content of the deleted `~/.claude/CLAUDE.md`
> with corrected paths. The original file was removed per owner decision; the
> MARKDOWN.god include directive (`@/root/MARKDOWN.god/MARKDOWN.god`) was a
> Claude Code mechanism and is not preserved here — the brain file itself
> still lives at `/root/MARKDOWN.god/MARKDOWN.god` and
> `Poslovno/30_Resources/MARKDOWN.god/`.

## Agent Registry (source of truth)

Before any work involving agents, skills, or commands, read the registry first:

1. `Poslovno/00_Vault-Meta/AGENTS.md` — repo guide and authority map
2. `Poslovno/30_Resources/agents/INDEX.md` — registry of subagents, skills, commands
3. `Poslovno/30_Resources/agents/WORKFLOWS.md` — multi-agent pipelines

Rules:
- Edit agent definitions ONLY under `Poslovno/30_Resources/agents/`; never hand-edit `~/.claude/*` symlinks or synced copies
- After edits run `Poslovno/30_Resources/scripts/sync-agents.sh`
- Validate with `Poslovno/30_Resources/agents/tests/run-tests.sh`

## Kovanica projects

- Protocol code lives at `/root/kovanica-protocol` — read its `AGENTS.md` before touching code
- Documentation hub: `/root/Obsidian-Vault` (pushes straight to `main`, commit prefix `docs:`)