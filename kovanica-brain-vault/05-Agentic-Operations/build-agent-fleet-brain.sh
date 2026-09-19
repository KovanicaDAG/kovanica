#!/usr/bin/env bash
# Build agents/MARKDOWN.god — the single-file compiled agent brain.
#
# Inlines every registry .md (workflows, subagents, skills, commands,
# plugins, templates) into one portable document usable by any tool.
# Excluded: INDEX.md files (superseded by the Brain Map), agents/MIGRATION.md
# and agents/tests/DEBUG.md (historical/internal).
#
# Usage:
#   ./scripts/build-god-brain.sh           regenerate MARKDOWN.god
#   ./scripts/build-god-brain.sh --check   exit 1 if MARKDOWN.god is stale

set -euo pipefail

VAULT="/root/Obsidian-Vault"
AGENTS="$VAULT/agents"
OUT="$AGENTS/MARKDOWN.god"
CHECK=false
[[ "${1:-}" == "--check" ]] && CHECK=true

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

list_files() {  # list_files <dir> — all *.md except INDEX.md, sorted
    ls "$1"/*.md 2>/dev/null | grep -v '/INDEX\.md$' | sort || true
}

emit_md() {  # emit_md <file> — body; frontmatter wrapped in a yaml fence
    awk 'NR==1 && $0=="---" {infm=1; print "```yaml"; next}
         infm==1 && $0=="---" {infm=0; print "```"; next}
         {print}' "$1"
}

emit_entry() {  # emit_entry <file> <vault-relative-label>
    local base
    base="$(basename "$1")"
    {
        echo ""
        echo "#### ${base%.md}"
        echo ""
        echo "*Source: \`agents/$2\`*"
        echo ""
        emit_md "$1"
        echo ""
    } >> "$TMP"
}

NSUB=$(list_files "$AGENTS/subagents" | wc -l)
NSKIL=$(list_files "$AGENTS/skills" | wc -l)
NCMD=$(list_files "$AGENTS/commands" | wc -l)
NPLUG=$(list_files "$AGENTS/plugins" | wc -l)
NTPL=$(list_files "$AGENTS/templates" | wc -l)
TOTAL=$((1 + NSUB + NSKIL + NCMD + NPLUG + NTPL))

# Deterministic stamp = newest source mtime, so --check stays valid over time.
NEWEST="$(ls -t "$AGENTS"/subagents/*.md "$AGENTS"/skills/*.md "$AGENTS"/commands/*.md "$AGENTS"/plugins/*.md "$AGENTS"/templates/*.md "$AGENTS"/WORKFLOWS.md 2>/dev/null | head -1)"
STAMP="$(date -u '+%Y-%m-%d %H:%M UTC' -r "$NEWEST" 2>/dev/null || date -u '+%Y-%m-%d %H:%M UTC')"

cat >> "$TMP" <<EOF
# ⚡ MARKDOWN.god — The Ultimate Agent Brain

> One file. Every .md. The complete operational knowledge of the KovanicaDAG
> agent fleet ($TOTAL documents), portable across OpenCode, Claude Code,
> Gemini CLI, Copilot, Cursor, Cline and any other markdown-fed tool.
>
> **GENERATED FILE — DO NOT EDIT BY HAND.**
> Source of truth: \`/root/Obsidian-Vault/agents/**\`
> Rebuild: \`./scripts/build-god-brain.sh\` · Drift check: \`./scripts/build-god-brain.sh --check\`
> Compiled: $STAMP

---

## The Laws

These override everything below.

1. **Finish Line** — work is done when *shipped*, never when merely edited.
   Vault edits → commit (\`docs:\` prefix) and push \`main\` in
   \`/root/Obsidian-Vault\`. Code edits → feature branch in
   \`/root/kovanica-protocol\`, gated by
   \`cargo fmt --check && cargo clippy --all-targets && cargo test\`,
   then draft PR. Nothing changed? Say so explicitly and stop.
2. **Authority** — protocol/code truth lives in \`/root/kovanica-protocol\`
   (read its \`AGENTS.md\` first). This vault holds docs and snapshots;
   prefer \`Poslovno/KovanicaDAG/myObsidianVaultDAG.md\` when snapshots disagree.
3. **Registry discipline** — edit agent definitions ONLY under
   \`agents/\` in this vault, never the synced copies (\`~/.claude/*\`,
   tool config dirs). After edits run \`./scripts/sync-agents.sh\`,
   validate with \`./agents/tests/run-tests.sh\`.
4. **Determinism is sacred** — consensus output is a pure function of the
   DAG. No HashMap order, wall-clock time, or unstable sorts in consensus
   paths. Tie-breaks fall back to BlockId byte order.
5. **Consensus changes demand proof** — written rationale naming protocol
   semantics plus deterministic AND adversarial tests (wide forks beyond
   k, Byzantine parents, tie-breaks, partitions).
6. **Never invent** — do not fabricate APIs, paths, commands, or roadmap
   items. Verify before claiming: run commands, don't assume output.
7. **Trackers Precede PRs** — always explicitly verify and check off
   completed items in \`TODO.md\` and \`ROADMAP.md\` *before* committing
   and opening a Draft PR.

---

## Brain Map

| Part  | Section                        | Docs |
|-------|--------------------------------|------|
| I     | Multi-Agent Workflows          | 1    |
| II    | Subagents                      | $NSUB |
| III   | Skills                         | $NSKIL |
| IV    | Commands                       | $NCMD |
| V     | Plugins                        | $NPLUG |
| VI    | Templates (reproduce the brain)| $NTPL |

Subagents: $(list_files "$AGENTS/subagents" | xargs -n1 basename | sed 's/\.md$//' | tr '\n' ' ')
Skills: $(list_files "$AGENTS/skills" | xargs -n1 basename | sed 's/\.md$//' | tr '\n' ' ')
Commands: /$(list_files "$AGENTS/commands" | xargs -n1 basename | sed 's/\.md$//' | tr '\n' ' /')
EOF

echo "" >> "$TMP"
echo "---" >> "$TMP"
echo "" >> "$TMP"
echo "## Part I — Multi-Agent Workflows" >> "$TMP"
echo "" >> "$TMP"
echo "*Source: \`agents/WORKFLOWS.md\`*" >> "$TMP"
echo "" >> "$TMP"
cat "$AGENTS/WORKFLOWS.md" >> "$TMP"

echo "" >> "$TMP"
echo "---" >> "$TMP"
echo "" >> "$TMP"
echo "## Part II — Subagents" >> "$TMP"
for f in $(list_files "$AGENTS/subagents"); do
    emit_entry "$f" "subagents/$(basename "$f")"
done

echo "" >> "$TMP"
echo "---" >> "$TMP"
echo "" >> "$TMP"
echo "## Part III — Skills" >> "$TMP"
for f in $(list_files "$AGENTS/skills"); do
    emit_entry "$f" "skills/$(basename "$f")"
done

echo "" >> "$TMP"
echo "---" >> "$TMP"
echo "" >> "$TMP"
echo "## Part IV — Commands" >> "$TMP"
for f in $(list_files "$AGENTS/commands"); do
    emit_entry "$f" "commands/$(basename "$f")"
done

echo "" >> "$TMP"
echo "---" >> "$TMP"
echo "" >> "$TMP"
echo "## Part V — Plugins" >> "$TMP"
for f in $(list_files "$AGENTS/plugins"); do
    emit_entry "$f" "plugins/$(basename "$f")"
done

echo "" >> "$TMP"
echo "---" >> "$TMP"
echo "" >> "$TMP"
echo "## Part VI — Templates (reproduce the brain)" >> "$TMP"
for f in $(list_files "$AGENTS/templates"); do
    emit_entry "$f" "templates/$(basename "$f")"
done

if $CHECK; then
    if [[ ! -f "$OUT" ]]; then
        echo "MARKDOWN.god missing — run build-god-brain.sh to create it" >&2
        exit 1
    fi
    if diff -q "$TMP" "$OUT" >/dev/null; then
        echo "MARKDOWN.god is up to date ($TOTAL docs)"
    else
        echo "MARKDOWN.god is STALE — registry changed since last build" >&2
        exit 1
    fi
else
    mv "$TMP" "$OUT"
    chmod 644 "$OUT"
    trap - EXIT
    echo "Built $OUT ($TOTAL docs)"
fi
