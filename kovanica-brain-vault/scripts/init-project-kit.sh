#!/usr/bin/env bash
# Scaffold the Project Kit (data docs) into a repository.
# Never overwrites existing files.
#
# Usage: ./scripts/init-project-kit.sh <target-dir>
set -euo pipefail

KIT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../templates/project-kit" && pwd)"
TARGET="${1:-}"

[[ -d "$TARGET" ]] || { echo "usage: $0 <existing-target-dir>" >&2; exit 1; }

created=0
skipped=0
for f in "$KIT"/*.md; do
    name="$(basename "$f")"
    if [[ -e "$TARGET/$name" ]]; then
        echo "skip (exists): $name"
        skipped=$((skipped+1))
    else
        cp "$f" "$TARGET/$name"
        echo "created: $name"
        created=$((created+1))
    fi
done

echo "done: $created created, $skipped skipped in $TARGET"
