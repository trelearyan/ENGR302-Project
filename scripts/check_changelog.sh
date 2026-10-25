#!/usr/bin/env bash
# NFR-15: every non-minor change must be recorded in CHANGELOG.md.
#
# Merge request pipelines: fails unless the MR adds at least one entry
# ("- ...") to CHANGELOG.md. MRs labelled "minor" (typos, formatting,
# comments, CI tweaks) are exempt.
#
# Tag pipelines: fails unless CHANGELOG.md has a "## [X.Y.Z]" section for
# the tag being released (tag "v1.2.0" or "1.2.0" -> "## [1.2.0]"), and the
# tag matches the Cargo.toml version.
#
# Always: fails unless the newest released version in CHANGELOG.md equals
# the version in Cargo.toml ([workspace.package]). Use scripts/release.sh
# to bump both together.
#
# Run locally against main with:  scripts/check_changelog.sh origin/main
# Only check the version sync with:  scripts/check_changelog.sh --version-only
set -euo pipefail

CHANGELOG="CHANGELOG.md"
SKIP_LABEL="minor"

fail() {
    echo "ERROR: $*" >&2
    exit 1
}

[[ -f "$CHANGELOG" ]] || fail "$CHANGELOG is missing."

# Cargo.toml and the changelog must agree on the current version.
cargo_version=$(awk '
    /^\[workspace\.package\]/ { in_section = 1; next }
    /^\[/                      { in_section = 0 }
    in_section && /^version[[:space:]]*=/ { gsub(/"/, "", $3); print $3; exit }
' Cargo.toml)
[[ -n "$cargo_version" ]] || fail "no version in the [workspace.package] section of Cargo.toml."

changelog_version=$(grep -Eo -m1 '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' "$CHANGELOG" | tr -d '#[] ' || true)
if [[ "$cargo_version" != "$changelog_version" ]]; then
    fail "Cargo.toml version is $cargo_version but the newest release in $CHANGELOG is" \
         "${changelog_version:-missing}. Use scripts/release.sh X.Y.Z to bump both together."
fi
echo "OK: Cargo.toml and $CHANGELOG are both at version $cargo_version."
[[ "${1:-}" == "--version-only" ]] && exit 0

# Release tags must have a matching section.
if [[ -n "${CI_COMMIT_TAG:-}" ]]; then
    version="${CI_COMMIT_TAG#v}"
    [[ "$version" == "$cargo_version" ]] ||
        fail "tag $CI_COMMIT_TAG does not match the Cargo.toml version $cargo_version."
    if grep -Eq "^## \[${version//./\\.}\]" "$CHANGELOG"; then
        echo "OK: $CHANGELOG has a section for $version."
        exit 0
    fi
    fail "tag $CI_COMMIT_TAG has no '## [$version]' section in $CHANGELOG." \
         "Move the [Unreleased] entries under a new '## [$version] - YYYY-MM-DD' heading."
fi

# Minor MRs are exempt.
if [[ ",${CI_MERGE_REQUEST_LABELS:-}," == *",$SKIP_LABEL,"* ]]; then
    echo "OK: merge request is labelled '$SKIP_LABEL'; changelog entry not required."
    exit 0
fi

base="${1:-${CI_MERGE_REQUEST_DIFF_BASE_SHA:-}}"
[[ -n "$base" ]] || fail "no base commit; pass one as an argument, e.g. origin/main."

# Only lines added to the changelog that are list entries count.
added_entries=$(git diff "$base" HEAD -- "$CHANGELOG" | grep -E '^\+\s*- ' || true)

if [[ -z "$added_entries" ]]; then
    cat >&2 <<MSG
ERROR: this merge request does not add an entry to $CHANGELOG (NFR-15).

Add a line under '## [Unreleased]' in the right section, for example:

    ### Added
    - Masthead with logo and GitLab link (#83)

Sections: Added, Changed, Deprecated, Removed, Fixed, Security.
If this change really is minor (typo, formatting, comments, CI tweak),
add the '$SKIP_LABEL' label to the merge request and re-run the pipeline.
MSG
    exit 1
fi

echo "OK: changelog entries added:"
echo "$added_entries"
