#!/usr/bin/env bash
# Central code quality and test dispatcher for eDirStat.
# Shared between the git pre-commit hook and CI/CD pipelines.
#
# Usage:
#   ./scripts/check.sh [all|fmt|typos|shear|clippy|test|fonts]
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$REPO_ROOT" ]]; then
    SCRIPT_PATH="$(realpath "$0" 2>/dev/null || readlink -f "$0" 2>/dev/null || echo "$0")"
    REPO_ROOT="$(cd "$(dirname "$SCRIPT_PATH")/.." && pwd -P)"
fi
cd "$REPO_ROOT"

is_ci() {
    [[ "${CI:-false}" == "true" ]] || [[ "${GITHUB_ACTIONS:-false}" == "true" ]]
}

group_start() {
    local title="$1"
    if is_ci; then
        echo "::group::$title"
    else
        echo "==> $title"
    fi
}

group_end() {
    if is_ci; then
        echo "::endgroup::"
    fi
}

check_fmt() {
    group_start "Checking code formatting (cargo fmt)"
    cargo fmt --all -- --check
    group_end
}

check_typos() {
    group_start "Checking typos (typos)"
    if ! command -v typos >/dev/null 2>&1; then
        echo "error: 'typos' is required but not installed." >&2
        exit 1
    fi
    typos
    group_end
}

check_shear() {
    group_start "Checking unused dependencies (cargo shear)"
    if ! command -v cargo-shear >/dev/null 2>&1 && ! cargo shear --version >/dev/null 2>&1; then
        echo "error: 'cargo shear' is required but not installed." >&2
        exit 1
    fi
    cargo shear
    group_end
}

check_clippy() {
    group_start "Checking lints (cargo clippy)"
    cargo clippy --workspace --all-targets
    group_end
}

check_test() {
    group_start "Running test suite (cargo nextest)"
    if ! command -v cargo-nextest >/dev/null 2>&1 && ! cargo nextest --version >/dev/null 2>&1; then
        echo "error: 'cargo nextest' is required but not installed." >&2
        exit 1
    fi
    cargo nextest run --workspace
    echo "==> Running documentation tests (cargo test --doc)..."
    cargo test --doc --workspace
    group_end
}

check_fonts() {
    group_start "Validating translation & font consistency"
    # 1. Verify that egui parses all embedded fonts and covers the corpus
    cargo test -p edirstat-gui --quiet -- fonts::tests

    # 2. When raw fonts are present or in CI, verify subset files match exactly
    if is_ci || [[ -d "crates/edirstat-gui/assets/fonts/raw" ]]; then
        ./scripts/fetch_fonts.sh
        python3 scripts/subset_fonts.py
        git diff --exit-code crates/edirstat-gui/assets/fonts/
    fi
    group_end
}

check_all() {
    check_fmt
    check_typos
    check_shear
    check_clippy
    check_test
    check_fonts
    echo "==> All checks passed successfully!"
}

cmd="${1:-all}"
case "$cmd" in
    fmt)    check_fmt ;;
    typos)  check_typos ;;
    shear)  check_shear ;;
    clippy) check_clippy ;;
    test)   check_test ;;
    fonts)  check_fonts ;;
    all)    check_all ;;
    *)
        echo "Unknown command: $cmd" >&2
        echo "Usage: $0 [all|fmt|typos|shear|clippy|test|fonts]" >&2
        exit 2
        ;;
esac
