#!/usr/bin/env bash
# Create a task contract without overwriting an existing task.
# Usage: ./scripts/new-task-contract.sh <project-dir> <slug>
set -euo pipefail

PROJECT="${1:-}"
SLUG="${2:-}"
[[ -d "$PROJECT" && -n "$SLUG" ]] || { echo "usage: $0 <project-dir> <slug>" >&2; exit 1; }

OUT="$PROJECT/.agent/tasks/$SLUG.md"
mkdir -p "$(dirname "$OUT")"
[[ ! -e "$OUT" ]] || { echo "ERROR: task already exists: $OUT" >&2; exit 1; }

cat > "$OUT" <<EOF
# Task Contract — $SLUG

## Goal
- <one sentence>

## Constraints
- <constraints>

## Scope
### GREEN
- <allowed>
### YELLOW
- <extra verification>
### RED
- <human authorization required>

## Non-goals
- <not solving>

## Acceptance
- [ ] <criterion>

## Verification
- <command/test>

## Risk
\`LOW\` / \`MEDIUM\` / \`HIGH\` / \`CRITICAL\`

## Rollback / containment
- <exit strategy>

## State
\`PLANNED\`
EOF

echo "created: $OUT"
