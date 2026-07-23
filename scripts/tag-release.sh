#!/usr/bin/env bash
# Cut a release by tagging the code commit at HEAD. This script never pushes:
# pushing the tag is what triggers .github/workflows/release.yml (build +
# publish), so that stays a deliberate, manual `git push`.
#
# Release notes (the current `## [Unreleased]` CHANGELOG.md section) are read
# from stdin and become the annotated tag's body; release.yml publishes that
# verbatim as the GitHub Release notes.
#
# The version is `<UTC date>-<short sha of HEAD>`, and the tag points at HEAD —
# the released code. The changelog stamp that records this version is committed
# afterward, on top of the tag, so the tagged commit never has to embed its own
# short sha. This is the sole place the version is computed; the changelog stamp
# reads it back from the tag.
#
# Usage: scripts/tag-release.sh < notes.md
set -euo pipefail

if (( $# != 0 )); then
  echo "usage: $0 < notes.md" >&2
  exit 2
fi

if [[ -t 0 ]]; then
  echo "error: release notes must be piped on stdin (e.g. \`$0 < notes.md\`)" >&2
  exit 1
fi

# Tag exactly the committed code: refuse if the tree is dirty.
if ! git diff --quiet HEAD; then
  echo "error: working tree has uncommitted changes; commit the release first" >&2
  exit 1
fi

version="$(date -u +'%Y.%m.%d')-$(git rev-parse --short HEAD)"

if git rev-parse --quiet --verify "refs/tags/$version" >/dev/null; then
  echo "error: tag $version already exists" >&2
  exit 1
fi

notes="$(cat)"
if [[ -z "$notes" ]]; then
  echo "error: release notes on stdin are empty" >&2
  exit 1
fi

# --cleanup=verbatim so Markdown headings survive: the default `strip` mode
# treats lines beginning with `#` (like `### Changed`) as comments and deletes
# them, silently mangling the release notes.
git tag --annotate --cleanup=verbatim --message "Release $version

$notes" "$version"

echo "Created tag $version"
