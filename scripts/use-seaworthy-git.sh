#!/usr/bin/env bash
# use-seaworthy-git.sh — switch eDirStat from the local seaworthy checkout
# (the path dependencies marked "seaworthy path dependency (TODO)") to the
# pushed seaworthy repository, pinned to one commit, and pin the release
# workflow's reusable seaworthy workflow to the same commit.
#
# Usage:
#   ./scripts/use-seaworthy-git.sh <full commit SHA> [<repository URL>]
#
# The repository defaults to https://github.com/xangelix/seaworthy. It
# rewrites, between their >>> / <<< markers:
#   - Cargo.toml          [workspace.dependencies] seaworthy-launcher
#   - launcher/Cargo.toml [dependencies] seaworthy-launcher
#   - .github/workflows/release.yml  uses: …/dist.yml@<SHA>
# then refreshes Cargo.lock and launcher/Cargo.lock for the new source only
# (cargo metadata: nothing else is upgraded).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
cd "$REPO_ROOT"

rev="${1:-}"
url="${2:-https://github.com/xangelix/seaworthy}"
if [[ ! "$rev" =~ ^[0-9a-f]{40}$ ]]; then
    echo "usage: $0 <full 40-character commit SHA> [<repository URL>]" >&2
    exit 2
fi
if [[ ! "$url" =~ ^(https://[A-Za-z0-9./_-]+|file:///[A-Za-z0-9./_-]+)$ ]]; then
    echo "error: '$url' isn't an https (or file) repository URL" >&2
    exit 2
fi
# The workflow pin names the repository as owner/name.
owner_repo=""
if [[ "$url" =~ ^https://github\.com/([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)$ ]]; then
    owner_repo="${BASH_REMATCH[1]%.git}"
fi

# Replaces the lines between "# >>> <marker>" and "# <<< <marker>" (the
# markers included) in <file> with <replacement>.
replace_block() { # <file> <marker> <replacement>
    local file="$1" marker="$2" replacement="$3"
    grep -q "^ *# >>> $marker" "$file" || { echo "error: $file has no '# >>> $marker' block (already switched?)" >&2; exit 1; }
    REPLACEMENT="$replacement" MARKER="$marker" awk '
        index($0, "# >>> " ENVIRON["MARKER"]) { skip = 1; print ENVIRON["REPLACEMENT"]; next }
        index($0, "# <<< " ENVIRON["MARKER"]) { skip = 0; next }
        !skip { print }
    ' "$file" > "$file.new"
    mv "$file.new" "$file"
}

dep="seaworthy-launcher = { git = \"$url\", rev = \"$rev\" }"
replace_block Cargo.toml "seaworthy path dependency (TODO)" \
"# seaworthy, pinned to one commit (scripts/use-seaworthy-git.sh moves it).
[workspace.dependencies]
$dep"
replace_block launcher/Cargo.toml "seaworthy path dependency (TODO)" \
"# seaworthy, pinned to the commit the workspace pins (scripts/use-seaworthy-git.sh).
$dep"
if [ -n "$owner_repo" ]; then
    replace_block .github/workflows/release.yml "seaworthy pin (TODO)" \
"    # The commit Cargo.toml pins seaworthy to (scripts/use-seaworthy-git.sh).
    uses: $owner_repo/.github/workflows/dist.yml@$rev # seaworthy ${rev:0:12}"
else
    echo "note: $url isn't a github.com repository: .github/workflows/release.yml keeps its placeholder pin" >&2
fi

echo "==> Refreshing the lockfiles for the new source"
cargo metadata --format-version 1 >/dev/null
(cd launcher && cargo metadata --format-version 1 >/dev/null)
grep -A2 '^name = "seaworthy-launcher"' Cargo.lock launcher/Cargo.lock | grep source || true

echo "==> Done. Next:"
echo "    ./scripts/check.sh all      (the licenses check regenerates the third-party list)"
echo "    git diff"
