#!/usr/bin/env bash
# scripts/bump_version.sh — automated version bumper & release preparation helper.
#
# Usage:
#   ./scripts/bump_version.sh <new_version>
#   ./scripts/bump_version.sh 2.3.0
#
# Updates:
#   - Cargo.toml (workspace.package.version)
#   - crates/edirstat/Cargo.toml (path dependencies)
#   - crates/edirstat-gui/Cargo.toml (path dependencies)
#   - runner/Cargo.toml (version and ProductVersion)
#   - runner/Cargo.lock (runner package version)
#   - web/config.toml (extra.version)
#   - web/package.json & web/package-lock.json (version)
#   - web/itch/index.html (title header version)
#   - CHANGELOG.md (converts [Unreleased] header to [vX.Y.Z] - YYYY-MM-DD)
#   - Cargo.lock (via cargo check)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
cd "$REPO_ROOT"

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <new_version>"
    echo "Example: $0 2.3.0"
    exit 1
fi

NEW_VER="$1"
NEW_VER="${NEW_VER#v}" # Strip leading 'v' if provided

# Validate semver-like format (X.Y.Z)
if [[ ! "$NEW_VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    echo "ERROR: Invalid version format: '$NEW_VER'. Expected semver (e.g. 2.3.0)." >&2
    exit 1
fi

CURRENT_VER="$(grep -E '^version = "[0-9]+\.[0-9]+\.[0-9]+"' Cargo.toml | head -n1 | cut -d'"' -f2)"
echo "==> Bumping version: $CURRENT_VER -> $NEW_VER"

# 1. Cargo.toml (workspace)
sed -i "s/^version = \"$CURRENT_VER\"/version = \"$NEW_VER\"/" Cargo.toml

# 2. crates/edirstat/Cargo.toml
sed -i "s/version = \"$CURRENT_VER\"/version = \"$NEW_VER\"/g" crates/edirstat/Cargo.toml

# 3. crates/edirstat-gui/Cargo.toml
sed -i "s/version = \"$CURRENT_VER\"/version = \"$NEW_VER\"/g" crates/edirstat-gui/Cargo.toml

# 4. runner/Cargo.toml
sed -i "s/^version = \"$CURRENT_VER\"/version = \"$NEW_VER\"/" runner/Cargo.toml
sed -i "s/ProductVersion = \"$CURRENT_VER\"/ProductVersion = \"$NEW_VER\"/" runner/Cargo.toml

# 5. runner/Cargo.lock (if present)
if [[ -f runner/Cargo.lock ]]; then
    sed -i "/name = \"edirstat-runner\"/{n;s/version = \".*\"/version = \"$NEW_VER\"/}" runner/Cargo.lock
fi

# 6. web/config.toml
sed -i "s/^version = \"$CURRENT_VER\"/version = \"$NEW_VER\"/" web/config.toml

# 7. web/package.json & lockfile
sed -i "s/\"version\": \"$CURRENT_VER\"/\"version\": \"$NEW_VER\"/" web/package.json
if command -v npm &>/dev/null && [[ -f web/package-lock.json ]]; then
    (cd web && npm install --package-lock-only --silent)
fi

# 8. web/itch/index.html
if [[ -f web/itch/index.html ]]; then
    sed -i "s/eDirStat v$CURRENT_VER/eDirStat v$NEW_VER/" web/itch/index.html
fi

# 9. CHANGELOG.md (convert [Unreleased] to [vX.Y.Z] - YYYY-MM-DD)
TODAY="$(date +%Y-%m-%d)"
if grep -q "## \[Unreleased\]" CHANGELOG.md; then
    sed -i "s/## \[Unreleased\]/## [v$NEW_VER] - $TODAY/" CHANGELOG.md
    echo "  ✓ Updated CHANGELOG.md header to [v$NEW_VER] - $TODAY"
fi

# 10. Update root Cargo.lock
echo "==> Updating root Cargo.lock via cargo check..."
cargo check --quiet

echo "==> Version bump complete: $CURRENT_VER -> $NEW_VER"
echo ""
echo "Next steps:"
echo "  1. Review changes:"
echo "       git diff"
echo ""
echo "  2. Run pre-release test suite & license verification:"
echo "       ./scripts/check.sh all"
echo ""
echo "  3. Commit and tag release:"
echo "       git commit -am \"release: v$NEW_VER\""
echo "       git tag -a \"v$NEW_VER\" -m \"Release v$NEW_VER\""
echo "       git push origin main --follow-tags"
echo ""
echo "  4. Publish to crates.io (strict order of operations due to dependencies):"
echo "       # Step 1: Base core data model (no workspace dependencies)"
echo "       cargo publish -p edirstat-core"
echo ""
echo "       # Wait ~30s for crates.io index to update, then Step 2:"
echo "       # GUI crate (depends on edirstat-core)"
echo "       cargo publish -p edirstat-gui"
echo ""
echo "       # Wait ~30s for crates.io index to update, then Step 3:"
echo "       # Main binary & engine (depends on edirstat-core and edirstat-gui)"
echo "       cargo publish -p edirstat"
echo ""
echo "       (Note: 'edirstat-runner' has publish = false and is not published to crates.io)"
echo ""
echo "  5. Deployments & Store Packages:"
echo "       - itch.io: Pushing tag triggers .github/workflows/release.yml (builds multivers & Inno Setup)"
echo "       - Mac App Store: Run ./scripts/package_macos.sh --appstore --build 1 --validate and upload .pkg"
