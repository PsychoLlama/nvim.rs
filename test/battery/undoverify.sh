#!/usr/bin/env bash
# Build the working tree and diff its undogold run against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   undoverify.sh [label]        # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit 96d70c6f79 (p20-3).  It lives in
# the CHECKOUT either way -- not in a session scratchpad, which is what
# cost 1785449630-spellverify a whole phase.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
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
OUT=${SWEEP_OUT:-/tmp/undogold-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log
# Cut mode.  `baseline.sh` re-enters this script as `--cut <nvim> <dir>` to
# cut the pinned baseline from the reference binary; it shares the ONE sweep
# call below with the head run, so the two sides of the differential cannot
# drift apart.  It skips the build and the diff.
CUT=
if [[ ${1:-} == --cut ]]; then CUT=$2; OUT=$3; LABEL=base; NVIM=$2; fi

mkdir -p "$OUT"
cd "$REPO"
if [[ -n ${NVIM:-} ]]; then
  BIN=$(realpath "$NVIM")
else
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

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${UNDO_BASELINE:-$("$HERE/baseline.sh" undo)}
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
