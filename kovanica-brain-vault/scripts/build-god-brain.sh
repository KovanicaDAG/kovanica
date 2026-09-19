#!/usr/bin/env bash
# Compile core/*.md (+ optional project layers) into MARKDOWN.god — the
# single-file universal agent brain.
#
# Usage:
#   ./scripts/build-god-brain.sh                    # core only
#   ./scripts/build-god-brain.sh LAYER...           # core + extra files/dirs
#   ./scripts/build-god-brain.sh --check            # exit 1 if stale
set -euo pipefail

VAULT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$VAULT/MARKDOWN.god"
CHECK=false
LAYERS=()
for arg in "$@"; do
    [[ "$arg" == "--check" ]] && CHECK=true || LAYERS+=("$arg")
done

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

emit_md() {  # emit_md <file> — body with frontmatter fenced as yaml;
             # markdown links flattened to plain text (portable output)
    awk 'NR==1 && $0=="---" {infm=1; print "```yaml"; next}
         infm==1 && $0=="---" {infm=0; print "```"; next}
         {print}' "$1" | sed -E 's/\[([^]]+)\]\([^)]+\.md\)/\1/g'
}

NCORE=$(find "$VAULT/core" -maxdepth 1 -name '*.md' 2>/dev/null | wc -l)
NMEM=0
[[ -d "$VAULT/memory" ]] && NMEM=$(find "$VAULT/memory" -maxdepth 1 -name '*.md' 2>/dev/null | wc -l)
NLAY=${#LAYERS[@]}

# Deterministic stamp = newest source mtime, so --check stays valid over time.
NEWEST="$(ls -t "$VAULT"/core/*.md "$VAULT"/memory/*.md 2>/dev/null | head -1)"
STAMP="$(date -u '+%Y-%m-%d %H:%M UTC' -r "$NEWEST" 2>/dev/null || date -u '+%Y-%m-%d %H:%M UTC')"

cat >> "$TMP" <<EOF
# ⚡ MARKDOWN.god — The Universal Agent Brain

> Load this once per session. Any agent, any codebase, any tool.
> Doctrine: elite coding expert + master planner, bound by The Laws.
>
> **GENERATED FILE — DO NOT EDIT BY HAND.**
> Source: core/ + memory/ doctrine · Sources as of: $STAMP · Core: $NCORE · Memory: $NMEM · Layers: $NLAY

---

## Contents

EOF

i=0
for f in $(ls "$VAULT"/core/*.md 2>/dev/null | sort); do
    i=$((i+1))
    title="$(grep -m1 '^# ' "$f" | sed 's/^# //')"
    echo "$i. ${title:-$(basename "$f" .md)}" >> "$TMP"
done
if (( NMEM > 0 )); then
    echo "" >> "$TMP"
    echo "Memory (learned across sessions):" >> "$TMP"
    for f in $(find "$VAULT/memory" -maxdepth 1 -name '*.md' | sort); do
        echo "- $(basename "$f")" >> "$TMP"
    done
fi
if (( NLAY > 0 )); then
    echo "" >> "$TMP"
    echo "Project layers appended:" >> "$TMP"
    for l in "${LAYERS[@]}"; do echo "- $l" >> "$TMP"; done
fi

echo "" >> "$TMP"
echo "---" >> "$TMP"
echo "" >> "$TMP"

for f in $(ls "$VAULT"/core/*.md 2>/dev/null | sort); do
    emit_md "$f" >> "$TMP"
    echo "" >> "$TMP"
    echo "---" >> "$TMP"
    echo "" >> "$TMP"
done

if (( NMEM > 0 )); then
    echo "## Memory — What the Agent Learned" >> "$TMP"
    echo "" >> "$TMP"
    for f in $(find "$VAULT/memory" -maxdepth 1 -name '*.md' | sort); do
        emit_md "$f" >> "$TMP"
        echo "" >> "$TMP"
        echo "---" >> "$TMP"
        echo "" >> "$TMP"
    done
fi

if (( NLAY > 0 )); then
    echo "## Project Layers" >> "$TMP"
    echo "" >> "$TMP"
    for l in "${LAYERS[@]}"; do
        if [[ -d "$l" ]]; then
            for lf in $(ls "$l"/*.md 2>/dev/null | grep -v '/INDEX\.md$' || true); do
                echo "**Source: \`$lf\`**" >> "$TMP"
                echo "" >> "$TMP"
                emit_md "$lf" >> "$TMP"
                echo "" >> "$TMP"
                echo "---" >> "$TMP"
                echo "" >> "$TMP"
            done
        elif [[ -f "$l" ]]; then
            echo "**Source: \`$l\`**" >> "$TMP"
            echo "" >> "$TMP"
            emit_md "$l" >> "$TMP"
            echo "" >> "$TMP"
            echo "---" >> "$TMP"
            echo "" >> "$TMP"
        else
            echo "WARN: layer not found: $l" >&2
        fi
    done
fi

if $CHECK; then
    if [[ ! -f "$OUT" ]]; then
        echo "MARKDOWN.god missing — run build-god-brain.sh" >&2
        exit 1
    fi
    if diff -q "$TMP" "$OUT" >/dev/null; then
        echo "MARKDOWN.god is up to date ($NCORE core docs)"
    else
        echo "MARKDOWN.god is STALE — rebuild required" >&2
        exit 1
    fi
else
    mv "$TMP" "$OUT"
    chmod 644 "$OUT"
    trap - EXIT
    echo "Built $OUT ($NCORE core docs, $NMEM memory docs, $NLAY layers)"
fi
