#!/usr/bin/env bash
# Agent registry test suite
# Usage: ./agents/tests/run-tests.sh [-v]
set -uo pipefail

VAULT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AGENTS="$VAULT/agents"
VERBOSE=false
[[ "${1:-}" == "-v" ]] && VERBOSE=true

PASS=0; FAIL=0; SKIP=0
t() {  # t <name> <cmd...>  — pass/fail wrapper, always returns 0
    local name="$1"; shift
    if "$@" >/tmp/agt.out 2>&1; then
        PASS=$((PASS+1))
        $VERBOSE && echo "  ok    $name"
    else
        FAIL=$((FAIL+1))
        echo "  FAIL  $name"
        sed 's/^/        | /' /tmp/agt.out | head -10
    fi
    return 0
}
s() { SKIP=$((SKIP+1)); $VERBOSE && echo "  skip  $1"; return 0; }

list_md() { ls "$AGENTS/$1"/*.md 2>/dev/null | xargs -n1 basename | sed 's/\.md$//' | grep -v '^INDEX$' | sort; }

echo "=== Agent Registry Test Suite ==="
echo "vault: $VAULT"
echo ""

# --- T1-T2: frontmatter + descriptions ---
bad_fm=""; missing_desc=""
for f in "$AGENTS"/subagents/*.md "$AGENTS"/skills/*.md "$AGENTS"/commands/*.md; do
    [[ "$(basename "$f")" == INDEX.md ]] && continue
    head -20 "$f" | grep -q '^---$' || bad_fm+="$(basename "$f") "
    grep -q '^description:' "$f" || missing_desc+="$(basename "$f") "
done
t "T1 all agent files have frontmatter" test -z "$bad_fm"
t "T2 all agent files have description:" test -z "$missing_desc"

# --- T3: subagent mode ---
bad_mode=""
for f in "$AGENTS"/subagents/*.md; do
    [[ "$(basename "$f")" == INDEX.md ]] && continue
    grep -q '^mode:' "$f" || bad_mode+="$(basename "$f") "
done
t "T3 subagents declare mode:" test -z "$bad_mode"

# --- T4: skill name matches filename ---
bad_name=""
for f in "$AGENTS"/skills/*.md; do
    [[ "$(basename "$f")" == INDEX.md ]] && continue
    base="$(basename "$f" .md)"
    name="$( (grep -m1 '^name:' "$f" || true) | sed 's/^name:[[:space:]]*//' )"
    [[ "$base" == "$name" ]] || bad_name+="$base(name=$name) "
done
t "T4 skill name == filename" test -z "$bad_name"

# --- T5: command -> subagent resolution ---
bad_ref=""
for f in "$AGENTS"/commands/*.md; do
    [[ "$(basename "$f")" == INDEX.md ]] && continue
    a="$( (grep -m1 '^agent:' "$f" || true) | sed 's/^agent:[[:space:]]*//' )"
    [[ -z "$a" || -f "$AGENTS/subagents/$a.md" ]] || bad_ref+="$(basename "$f")->$a "
done
t "T5 commands reference existing subagents" test -z "$bad_ref"

# --- T6: tool JSON validity ---
check_json() {
    local ok=0
    for j in "$AGENTS"/tools/*.json; do
        python3 -m json.tool "$j" >/dev/null 2>&1 || { echo "$(basename "$j") invalid"; ok=1; }
    done
    return $ok
}
t "T6 all tools/*.json parse" check_json

# --- T7: opencode.json <-> subagents consistency ---
SUBS="$(list_md subagents | tr '\n' ' ')"
oc_check() {
    python3 -c "
import json, sys
d = json.load(open('$AGENTS/tools/opencode.json'))
known = '''$SUBS'''.split()
missing = set(d.get('agent', {})) - set(known)
print(' '.join(missing))
sys.exit(1 if missing else 0)"
}
t "T7 opencode.json agents exist as files" oc_check

# --- T8: mcp.json used_by references valid agents ---
mcp_check() {
    python3 -c "
import json, sys
d = json.load(open('$AGENTS/tools/mcp.json'))
known = '''$SUBS'''.split()
bad = [f'{s}:{u}' for s, c in d.get('servers', {}).items() for u in c.get('used_by', []) if u not in known]
print(' '.join(bad))
sys.exit(1 if bad else 0)" 2>/dev/null
}
t "T8 mcp.json used_by -> valid agents" mcp_check

# --- T9: no stale absolute paths (pattern built to avoid self-match) ---

stale=$(grep -rl --exclude=run-tests.sh --exclude=DEBUG.md -- "/home/" "$AGENTS" 2>/dev/null || true)
t "T9 no stale /home/* paths" test -z "$stale"

# --- T10: protocol repo reachable ---
if [[ -d /root/kovanica-protocol ]]; then
    t "T10 kovanica-protocol exists at /root/kovanica-protocol" test -d /root/kovanica-protocol
else
    s "T10 kovanica-protocol not on this machine"
fi

# --- T11: claude symlinks resolve ---
broken=$(find ~/.claude/agents ~/.claude/skills ~/.claude/commands -maxdepth 1 -xtype l 2>/dev/null || true)
t "T11 no broken claude symlinks" test -z "$broken"

# --- T12: symlink coverage complete ---
if [[ -d ~/.claude/agents ]]; then
    missing_ln=$(comm -13 \
        <(ls ~/.claude/agents 2>/dev/null | sort) \
        <(ls "$AGENTS"/subagents/*.md | xargs -n1 basename | sort))
    t "T12 claude agents dir covers all subagents" test -z "$missing_ln"
else
    s "T12 ~/.claude/agents not present"
fi

# --- T13: indexes list every entry ---
idx_check() {
    local ok=0 pair dir idx n
    for pair in "subagents" "skills" "commands"; do
        dir="$pair"; idx="$AGENTS/$pair/INDEX.md"
        for n in $(list_md "$dir"); do
            grep -q "\[\[$n\]\]" "$idx" || { echo "$idx missing [[$n]]"; ok=1; }
        done
    done
    return $ok
}
t "T13 INDEX files reference every entry" idx_check

# --- T14: WORKFLOWS.md links resolve ---
wf_check() {
    local bad
    bad=$(grep -o '\[\[[a-z-]*\]\]' "$AGENTS/WORKFLOWS.md" 2>/dev/null | tr -d '[]' | sort -u | while read -r n; do
        [[ -f "$AGENTS/subagents/$n.md" || -f "$AGENTS/commands/$n.md" || -f "$AGENTS/skills/$n.md" || "$n" == index ]] || echo "$n"
    done)
    [[ -z "$bad" ]]
}
t "T14 WORKFLOWS.md links resolve" wf_check

# --- T15: every agent file ends with the Finish Line clause ---
no_fl=""
for f in "$AGENTS"/subagents/*.md "$AGENTS"/skills/*.md "$AGENTS"/commands/*.md; do
    [[ "$(basename "$f")" == INDEX.md ]] && continue
    grep -q '^## Finish Line$' "$f" || no_fl+="agents/${f#$AGENTS/} "
done
t "T15 all agent files have Finish Line section" test -z "$no_fl"

# --- Summary ---
echo ""
echo "==============================="
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped"
if [[ $FAIL -eq 0 ]]; then
    echo "ALL TESTS PASSED"
else
    echo "TESTS FAILED"
fi
exit $(( FAIL > 0 ? 1 : 0 ))
