#!/usr/bin/env bash
# Git pre-commit hook for eDirStat.
# Delegates to scripts/check.sh to ensure local commits pass all CI quality gates.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$REPO_ROOT" ]]; then
    SCRIPT_PATH="$(realpath "$0" 2>/dev/null || readlink -f "$0" 2>/dev/null || echo "$0")"
    SCRIPT_DIR="$(dirname "$SCRIPT_PATH")"
    if [[ "$SCRIPT_DIR" =~ \.git/hooks$ ]]; then
        REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd -P)"
    else
        REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd -P)"
    fi
fi

cd "$REPO_ROOT"

exec "$REPO_ROOT/scripts/check.sh" all
