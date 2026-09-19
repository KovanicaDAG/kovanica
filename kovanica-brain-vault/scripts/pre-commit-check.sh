#!/usr/bin/env bash
# Git pre-commit hook: block a commit if core/ or memory/ changed but
# MARKDOWN.god wasn't rebuilt to match.
#
# Install (once per clone):
#   ln -sf ../../scripts/pre-commit-check.sh .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit
set -euo pipefail

VAULT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! git -C "$VAULT" diff --cached --name-only | grep -qE '^(core|memory)/.*\.md$'; then
    exit 0  # core/memory untouched in this commit — nothing to check
fi

if ! "$VAULT/scripts/build-god-brain.sh" --check >/dev/null 2>&1; then
    echo "pre-commit: core/ or memory/ changed but MARKDOWN.god is stale." >&2
    echo "            run ./scripts/build-god-brain.sh, git add MARKDOWN.god, retry." >&2
    exit 1
fi
