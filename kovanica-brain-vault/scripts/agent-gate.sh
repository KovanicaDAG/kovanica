#!/usr/bin/env bash
# Validate a Task Contract and its execution gates.
# Usage: ./scripts/agent-gate.sh <project-dir> <task-slug> [state]
set -euo pipefail

PROJECT="${1:-}"
SLUG="${2:-}"
STATE="${3:-}"
[[ -d "$PROJECT" && -n "$SLUG" ]] || { echo "usage: $0 <project-dir> <task-slug> [state]" >&2; exit 1; }

TASK="$PROJECT/.agent/tasks/$SLUG.md"
[[ -f "$TASK" ]] || { echo "[FAIL] missing task contract: $TASK" >&2; exit 1; }

FAIL=0
ok(){ echo "[OK] $1"; }
bad(){ echo "[FAIL] $1" >&2; FAIL=$((FAIL+1)); }

for section in '## Goal' '## Constraints' '## Scope' '## Non-goals' '## Acceptance' '## Verification' '## Risk' '## Rollback / containment' '## State'; do
  grep -qF "$section" "$TASK" && ok "task section: ${section#\#\# }" || bad "missing task section: $section"
done

risk="$(sed -n '/^## Risk$/,/^## /p' "$TASK" | grep -Eo 'LOW|MEDIUM|HIGH|CRITICAL' | head -1 || true)"
[[ -n "$risk" ]] && ok "risk declared: $risk" || bad "risk not declared"

if [[ "$risk" == "HIGH" || "$risk" == "CRITICAL" ]]; then
  if grep -A3 -q '^## Rollback / containment$' "$TASK" && ! grep -A3 -qE '<exit strategy>|^$' "$TASK"; then
    ok "rollback/containment declared for $risk risk"
  else
    bad "$risk task lacks a concrete rollback/containment strategy"
  fi
fi

if [[ -n "$STATE" ]]; then
  case "$STATE" in
    BOOT|RECON|PLAN|EXECUTE|VERIFY|REVIEW|SHIP|REFLECT|IDLE) ok "requested state valid: $STATE" ;;
    *) bad "invalid state: $STATE" ;;
  esac
fi

# Claims ledger is required for material claims once the project kit exists.
if [[ -f "$PROJECT/EVIDENCE.md" ]]; then
  if grep -q '^| Claim | Status | Evidence | Verified at | Stability |' "$PROJECT/EVIDENCE.md"; then
    ok "evidence ledger schema present"
  else
    bad "EVIDENCE.md schema is invalid"
  fi
fi

if (( FAIL )); then
  echo "agent-gate: $FAIL failure(s)" >&2
  exit 1
fi
echo "agent-gate: ready"
