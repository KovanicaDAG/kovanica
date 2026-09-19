#!/usr/bin/env bash
# Create/update a truthful resumable checkpoint.
# Usage: ./scripts/checkpoint.sh <project-dir> <state> <goal> <next-action>
set -euo pipefail
PROJECT="${1:-}"; STATE="${2:-}"; GOAL="${3:-}"; NEXT="${4:-}"
OUT="$PROJECT/AGENT-STATE.md"
[[ -d "$PROJECT" && -n "$STATE" && -n "$GOAL" && -n "$NEXT" ]] || { echo "usage: $0 <project-dir> <state> <goal> <next-action>" >&2; exit 1; }
case "$STATE" in BOOT|RECON|PLAN|EXECUTE|VERIFY|REVIEW|SHIP|REFLECT|IDLE) ;; *) echo "invalid state: $STATE" >&2; exit 1 ;; esac
[[ -f "$OUT" ]] && cp "$OUT" "$OUT.bak"
cat > "$OUT" <<EOF2
# AGENT-STATE — resumable execution checkpoint

## State

$STATE

## Goal

- $GOAL

## Completed

- <verified completed work>

## In progress

- <current action>

## Changed files

- <path> — <why>

## Evidence

- <command/test/file/commit>

## Open hypotheses / unknowns

- <unknown>

## Failures / blockers

- <failure and current understanding>

## Next action

1. $NEXT

## Risk

LOW / MEDIUM / HIGH / CRITICAL

## Rollback / containment

- <exit strategy>

## Gate status

- G0 Understanding: PENDING / PASS / BLOCKED
- G1 Change: PENDING / PASS / BLOCKED
- G2 Behavior: PENDING / PASS / BLOCKED
- G3 Integration: PENDING / PASS / BLOCKED
- G4 Ship: PENDING / PASS / BLOCKED
EOF2
printf 'checkpoint: %s (%s)\n' "$OUT" "$STATE"
