#!/usr/bin/env bash
# Git pre-commit hook for eDirStat.
# Verifies that any modified translation files have matching, updated font subsets.
set -euo pipefail

# Check if any .ftl locale files are staged for commit
STAGED_FTL=$(git diff --cached --name-only -- 'crates/edirstat-gui/assets/locales/*.ftl' 'crates/edirstat-gui/assets/locales/**/*.ftl' 2>/dev/null || true)

if [[ -n "$STAGED_FTL" ]]; then
    echo "==> [pre-commit] Translation files staged for commit. Validating fallback font subsets..."
    
    # Run font corpus coverage tests
    if ! cargo test -p edirstat-gui --quiet -- fonts::tests; then
        echo "error: [pre-commit] Font subsets are outdated for the staged translations!" >&2
        echo "       Run './scripts/update_fonts.sh' and stage the updated fonts:" >&2
        echo "       git add crates/edirstat-gui/assets/fonts/" >&2
        exit 1
    fi
fi
