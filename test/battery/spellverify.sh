#!/usr/bin/env bash
# Build the working tree and diff its spellsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   spellverify.sh [label]        # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit 47e44ab7c6 (the phase-18 close, i.e. phase 19's base)
# by the P19 close, after the original baseline was lost with the dead
# session scratchpad this script used to hardcode.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
#
# The sweep reuses one work directory path for every run because the
# generated .spl/.sug bytes and the report both record it.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/spellsweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
# Cut mode.  `baseline.sh` re-enters this script as `--cut <nvim> <dir>` to
# cut the pinned baseline from the reference binary; it shares the ONE sweep
# call below with the head run, so the two sides of the differential cannot
# drift apart.  It skips the build and the diff.
CUT=
if [[ ${1:-} == --cut ]]; then CUT=$2; OUT=$3; LABEL=base; LOG=/dev/null; fi
NVIM_BIN=${CUT:-$REPO/target/debug/nvim}

mkdir -p "$OUT"
cd "$REPO"
if [[ -z $CUT ]] && ! just build >"$OUT/build-$LABEL.log" 2>&1; then
  echo "BUILD FAILED -- see $OUT/build-$LABEL.log" >&2
  grep -E '^(error|warning)' "$OUT/build-$LABEL.log" | head -60 >&2
  exit 1
fi

rm -rf "$OUT/$LABEL.txt" "$OUT/$LABEL.hashes" "$OUT/$LABEL-files"
SPELLSWEEP_WORK=${SPELLSWEEP_WORK:-/tmp/spellsweep-verify} \
  "$HERE/spellsweep.sh" "$NVIM_BIN" \
  "$REPO/runtime" "$OUT" "$LABEL" 2>&1 | tail -2

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${SPELL_BASELINE:-$("$HERE/baseline.sh" spell)}
fail=0
if diff -q "$BASELINE/base.txt" "$OUT/$LABEL.txt" >/dev/null; then
  echo "report: IDENTICAL"
else
  echo "report: DIFFERS"
  diff "$BASELINE/base.txt" "$OUT/$LABEL.txt" | head -60
  fail=1
fi
if diff -q "$BASELINE/base.hashes" "$OUT/$LABEL.hashes" >/dev/null; then
  echo "bytes: IDENTICAL"
else
  echo "bytes: DIFFERS"
  diff "$BASELINE/base.hashes" "$OUT/$LABEL.hashes" | head -40
  fail=1
fi
exit $fail
