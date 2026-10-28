#!/usr/bin/env bash
# use-seaworthy-git.sh — move eDirStat to another seaworthy commit: the
# seaworthy-launcher pins in Cargo.toml and launcher/Cargo.toml and the
# reusable seaworthy workflows (dist.yml, delete-artifacts.yml) that
# release.yml and packages.yml call, all to one commit of
# github.com/xangelix/seaworthy, then both
# lockfiles refreshed for the new source alone (cargo metadata: nothing else
# is upgraded).
#
# Usage:
#   ./scripts/use-seaworthy-git.sh <full commit SHA>
#
# A newer seaworthy can need a newer version of a crate eDirStat already
# locks; cargo then names the conflict, and `cargo update -p
# <crate>@<locked> --precise <needed>` (in the workspace that names it)
# moves that one crate before a second run.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
cd "$REPO_ROOT"

rev="${1:-}"
if [[ ! "$rev" =~ ^[0-9a-f]{40}$ ]]; then
    echo "usage: $0 <full 40-character commit SHA>" >&2
    exit 2
fi

# Rewrites <file> through sed's extended expression <expression>, and fails
# unless the result names the new commit (the pin was where expected).
pin() { # <file> <expression>
    sed -E "$2" "$1" > "$1.new"
    mv "$1.new" "$1"
    grep -q "$rev" "$1" || { echo "error: $1 holds no seaworthy pin to move" >&2; exit 1; }
}

pin Cargo.toml \
    "s#(seaworthy-launcher = \\{ git = \"https://github.com/xangelix/seaworthy\", rev = \")[0-9a-f]{40}(\" \\})#\\1$rev\\2#"
pin launcher/Cargo.toml \
    "s#(seaworthy-launcher = \\{ git = \"https://github.com/xangelix/seaworthy\", rev = \")[0-9a-f]{40}(\" \\})#\\1$rev\\2#"
for workflow in .github/workflows/release.yml .github/workflows/packages.yml; do
    pin "$workflow" \
        "s#(uses: xangelix/seaworthy/\\.github/workflows/[a-z-]+\\.yml@)[0-9a-f]{40}( \\# seaworthy )[0-9a-f]{12}#\\1$rev\\2${rev:0:12}#"
    # Every seaworthy workflow it calls, at the one commit.
    if grep -E 'xangelix/seaworthy/\.github/workflows/' "$workflow" | grep -v -q "@$rev # seaworthy ${rev:0:12}\$"; then
        echo "error: $workflow calls a seaworthy workflow that isn't pinned as \`@<sha> # seaworthy <12 hex>\`" >&2
        exit 1
    fi
done

echo "==> Refreshing the lockfiles for the new source"
cargo metadata --format-version 1 >/dev/null
(cd launcher && cargo metadata --format-version 1 >/dev/null)
grep -A2 '^name = "seaworthy-launcher"' Cargo.lock launcher/Cargo.lock | grep source || true

echo "==> Done. Next:"
echo "    ./scripts/check.sh all      (the licenses check regenerates the third-party list)"
echo "    git diff"
