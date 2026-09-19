#!/usr/bin/env bash
# Validate the Project Kit structure in a target project.
# Usage: ./scripts/validate-project-kit.sh <project-dir>
set -euo pipefail

PROJECT="${1:-}"
[[ -d "$PROJECT" ]] || { echo "usage: $0 <project-dir>" >&2; exit 1; }

required=(VERIFIED-FACTS.md EVIDENCE.md CODE-MAP.md CURRENT-STATE.md AGENT-STATE.md DECISION-LOG.md KNOWN-TRAPS.md COMMANDS.md ROADMAP.md SESSION-LOG.md GLOSSARY.md SCOPE.md IMPACT-MAP.md)
missing=0
for f in "${required[@]}"; do
    if [[ -f "$PROJECT/$f" ]]; then
        echo "[OK] $f"
    else
        echo "[MISSING] $f" >&2
        missing=$((missing+1))
    fi
done

if [[ -d "$PROJECT/.agent/tasks" ]]; then
    echo "[OK] .agent/tasks"
else
    echo "[MISSING] .agent/tasks" >&2
    missing=$((missing+1))
fi

if (( missing )); then
    echo "Project Kit invalid: $missing missing item(s)" >&2
    exit 1
fi
echo "Project Kit valid"
