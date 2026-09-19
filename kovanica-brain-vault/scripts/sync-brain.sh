#!/usr/bin/env bash
# Wire MARKDOWN.god into global tool configs (idempotent).
# Targets: Claude Code (~/.claude/CLAUDE.md), OpenCode (~/.config/opencode/opencode.jsonc)
set -euo pipefail

GOD="/root/MARKDOWN.god/MARKDOWN.god"
CLAUDE_MD="${HOME}/.claude/CLAUDE.md"
OC_JSONC="${HOME}/.config/opencode/opencode.jsonc"

[[ -f "$GOD" ]] || { echo "ERROR: $GOD missing — build it first" >&2; exit 1; }

# --- Claude Code: managed block in ~/.claude/CLAUDE.md ---
BEGIN="<!-- MARKDOWN.GOD:BEGIN (managed by scripts/sync-brain.sh) -->"
END="<!-- MARKDOWN.GOD:END -->"
BLOCK="$BEGIN
@${GOD}
$END"

if [[ ! -f "$CLAUDE_MD" ]]; then
    printf '# Global agent guidance\n\n%s\n' "$BLOCK" > "$CLAUDE_MD"
    echo "claude: created $CLAUDE_MD with god block"
elif grep -qF "$BEGIN" "$CLAUDE_MD"; then
    tmp="$(mktemp)"
    awk -v begin="$BEGIN" -v end="$END" -v block="$BLOCK" '
        $0 == begin { print block; skip=1; next }
        $0 == end   { skip=0; next }
        skip != 1   { print }' "$CLAUDE_MD" > "$tmp"
    mv "$tmp" "$CLAUDE_MD"
    echo "claude: god block refreshed in $CLAUDE_MD"
else
    printf '\n%s\n' "$BLOCK" >> "$CLAUDE_MD"
    echo "claude: god block appended to $CLAUDE_MD"
fi

# --- OpenCode: instructions entry in ~/.config/opencode/opencode.jsonc ---
if [[ ! -f "$OC_JSONC" ]]; then
    echo "opencode: $OC_JSONC not found — skipped"
elif python3 - "$OC_JSONC" "$GOD" <<'PY'
import json, re, sys

path, god = sys.argv[1], sys.argv[2]
raw = open(path).read()
stripped = re.sub(r'^\s*//.*$', '', raw, flags=re.M)      # line comments only
try:
    cfg = json.loads(stripped)
except json.JSONDecodeError as e:
    print(f"opencode: cannot auto-edit (block comments or syntax): {e}")
    print(f'          add "{god}" to "instructions" manually')
    sys.exit(0)

instr = cfg.get("instructions", [])
if isinstance(instr, str):
    instr = [instr]
elif not isinstance(instr, list):
    instr = []
if god in instr:
    print("opencode: already wired")
else:
    instr.append(god)
    cfg["instructions"] = instr
    with open(path, "w") as fh:
        json.dump(cfg, fh, indent=2)
        fh.write("\n")
    print(f"opencode: added to instructions -> {path}")
PY
then
    :  # python handled messaging; non-zero would abort under set -e
fi

echo "Sync complete."
