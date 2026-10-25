#!/usr/bin/env bash
# Release a new version: bump Cargo.toml and CHANGELOG.md together.
#
#   scripts/release.sh 0.7.0
#
# 1. Sets [workspace.package] version in Cargo.toml (all crates inherit it)
#    and refreshes Cargo.lock.
# 2. Renames the [Unreleased] entries in CHANGELOG.md to "## [0.7.0] - <today>"
#    and starts a new empty [Unreleased] section above them.
# 3. Runs scripts/check_changelog.sh to confirm they match.
#
# It does not commit or tag; it prints the commands to do that.
set -euo pipefail

cd "$(dirname "$0")/.."

new_version="${1:-}"
[[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
    { echo "usage: scripts/release.sh X.Y.Z   (e.g. 0.7.0)" >&2; exit 1; }

if grep -Eq "^## \[${new_version//./\\.}\]" CHANGELOG.md; then
    echo "ERROR: CHANGELOG.md already has a section for $new_version." >&2
    exit 1
fi

# The [Unreleased] section must have at least one entry to release.
unreleased_entries=$(awk '
    /^## \[Unreleased\]$/ { in_section = 1; next }
    /^## \[/              { in_section = 0 }
    in_section && /^[[:space:]]*- / { count++ }
    END { print count + 0 }
' CHANGELOG.md)
if [[ "$unreleased_entries" -eq 0 ]]; then
    echo "ERROR: the [Unreleased] section of CHANGELOG.md is empty; nothing to release." >&2
    exit 1
fi

today=$(date +%Y-%m-%d)

# Cargo.toml: replace the version line inside [workspace.package] only.
awk -v version="$new_version" '
    /^\[workspace\.package\]/ { in_section = 1 }
    /^\[/ && !/^\[workspace\.package\]/ { in_section = 0 }
    in_section && /^version[[:space:]]*=/ { print "version = \"" version "\""; next }
    { print }
' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

# CHANGELOG.md: keep an empty [Unreleased] and put the entries under the new version.
awk -v version="$new_version" -v today="$today" '
    /^## \[Unreleased\]$/ && !done { print; print ""; print "## [" version "] - " today; done = 1; next }
    { print }
' CHANGELOG.md > CHANGELOG.md.tmp && mv CHANGELOG.md.tmp CHANGELOG.md

cargo update --workspace --quiet

bash scripts/check_changelog.sh --version-only

echo
echo "Bumped to $new_version ($today) in Cargo.toml, Cargo.lock and CHANGELOG.md."
echo "Check the changes, then commit and tag:"
echo "  git diff"
echo "  git commit -am \"Release $new_version\""
echo "  git tag v$new_version"
echo "  git push && git push origin v$new_version"
