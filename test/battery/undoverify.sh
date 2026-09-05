#!/usr/bin/env bash
# Build the working tree and diff its undogold run against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   undoverify.sh [label]        # default label: cur
#
# The baseline lives next to this script in undobase/ (see its
# COMMIT file for where it was taken) -- deliberately beside the script
# and not in a session scratchpad, which is what cost 1785449630-spellverify
# a whole phase.  Regenerate it only when a behaviour change is
# *intended* and reviewed:
#
#   UNDOGOLD_WORK=/tmp/ugold-verify \
#     undogold.sh <nvim> <runtime> \
#       test/battery/undobase base
#
# The sweep reuses one work directory path for every run because 'undodir'
# munges that path into the undo file's *name* and the report prints it.
#
# Environment:
#   UNDO_BASELINE   baseline directory       (default: beside this script)
#   SWEEP_OUT       where this run's artifacts land (default: /tmp/undogold-out)
#   UNDOGOLD_WORK   the sweep's sandbox      (default: /tmp/ugold)
#   REPO            the tree to build        (default: the nvim.rs checkout)
#   NVIM            skip the build, use this binary instead
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${UNDO_BASELINE:-$HERE/undobase}
OUT=${SWEEP_OUT:-/tmp/undogold-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log

mkdir -p "$OUT"
if [[ -n ${NVIM:-} ]]; then
  BIN=$(realpath "$NVIM")
else
  cd "$REPO"
  if ! just build >"$LOG" 2>&1; then
    echo "BUILD FAILED -- see $LOG" >&2
    grep -E '^(error|warning)' "$LOG" | head -60 >&2
    exit 1
  fi
  BIN=$REPO/target/debug/nvim
fi

rm -rf "${OUT:?}/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.hashes" \
  "$OUT/$LABEL-files"
"$HERE/undogold.sh" "$BIN" "$REPO/runtime" "$OUT" "$LABEL" \
  2>&1 | tail -2

fail=0
for part in txt struct hashes; do
  if [[ ! -f $BASELINE/base.$part ]]; then
    echo "$part: NO BASELINE ($BASELINE/base.$part)"
    fail=1
  elif diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the report and the decode both carry latin1 bytes, and diff
    # would otherwise call them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` otherwise aborts on the first
    # differing artifact, so the remaining ones -- the ones that say
    # *which* layer moved -- are never compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
