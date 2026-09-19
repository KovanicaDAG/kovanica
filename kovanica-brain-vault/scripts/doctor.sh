#!/usr/bin/env bash
# Health check for MARKDOWN.god itself, or for a project using its kit.
# Usage: ./scripts/doctor.sh [project-dir]
set -euo pipefail

VAULT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-$VAULT}"
FAIL=0

ok(){ echo "[OK] $1"; }
warn(){ echo "[WARN] $1"; }
bad(){ echo "[FAIL] $1" >&2; FAIL=$((FAIL+1)); }

[[ -f "$VAULT/MARKDOWN.god" ]] && ok "compiled brain exists" || bad "MARKDOWN.god missing"
[[ -d "$VAULT/core" && -d "$VAULT/memory" ]] && ok "core and memory directories exist" || bad "core/memory missing"

if bash -n "$VAULT"/scripts/*.sh 2>/dev/null; then
    ok "all shell scripts parse"
else
    bad "shell script syntax error"
fi

if "$VAULT/scripts/build-god-brain.sh" --check >/dev/null 2>&1; then
    ok "compiled brain is fresh"
else
    bad "compiled brain is stale"
fi

required_scripts=(agent-gate.sh checkpoint.sh doctor.sh init-project-kit.sh new-task-contract.sh validate-project-kit.sh build-god-brain.sh)
for script in "${required_scripts[@]}"; do
    if [[ -x "$VAULT/scripts/$script" ]]; then ok "operational script: $script"; else bad "missing/non-executable script: $script"; fi
done

for template in AGENT-STATE.md EVIDENCE.md SCOPE.md IMPACT-MAP.md TASK-CONTRACT.md; do
    [[ -f "$VAULT/templates/project-kit/$template" ]] && ok "project template: $template" || bad "missing project template: $template"
done

for f in "$VAULT"/core/*.md; do
    base="$(basename "$f" .md | sed -E 's/^[0-9]+-//')"
    name="$(sed -n 's/^name: *//p' "$f" | head -1)"
    [[ -n "$name" && "${name,,}" == "${base,,}" ]] && ok "frontmatter: $base" || bad "frontmatter name mismatch: $f"
done

if [[ "$TARGET" != "$VAULT" ]]; then
    if "$VAULT/scripts/validate-project-kit.sh" "$TARGET" >/dev/null; then
        ok "project kit valid: $TARGET"
    else
        bad "project kit invalid: $TARGET"
    fi
fi

if [[ -d "$VAULT/.git" ]]; then
    if [[ -x "$VAULT/.git/hooks/pre-commit" || -L "$VAULT/.git/hooks/pre-commit" ]]; then
        ok "pre-commit freshness guard installed"
    else
        warn "pre-commit freshness guard not installed"
    fi
fi

if (( FAIL )); then
    echo "MARKDOWN.god doctor: $FAIL failure(s)" >&2
    exit 1
fi
echo "MARKDOWN.god doctor: healthy"
