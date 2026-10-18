#!/usr/bin/env bash
# Build the working tree and diff its spellmemsweep (when `:mkspell`
# compresses its trees under forced 'mkspellmem' settings, and the bytes
# that come out) against the baseline cut from `test/battery/BASE`.
#
#   spellmemverify.sh [label]        # default label: cur
#
# The baseline is CUT, not committed (see README.md); a behaviour change
# is a BASE bump in a commit of its own. The cut is the slow half: the
# reference binary's .sug pass is quadratic in a run of words that fold
# alike, ~12 s per SAL setting in a debug build.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/spellmemsweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
# Cut mode: `baseline.sh` re-enters this script as `--cut <nvim> <dir>`,
# sharing the one sweep call below with the head run.
CUT=
if [[ ${1:-} == --cut ]]; then CUT=$2; OUT=$3; LABEL=base; fi
NVIM_BIN=${CUT:-$REPO/target/debug/nvim}

mkdir -p "$OUT"
cd "$REPO"
if [[ -z $CUT ]] && ! just build >"$OUT/build-$LABEL.log" 2>&1; then
  echo "BUILD FAILED -- see $OUT/build-$LABEL.log" >&2
  grep -E '^(error|warning)' "$OUT/build-$LABEL.log" | head -60 >&2
  exit 1
fi

rm -f "$OUT/$LABEL.txt"
SPELLMEMSWEEP_WORK=${SPELLMEMSWEEP_WORK:-/tmp/spellmemsweep-verify} \
  "$HERE/spellmemsweep.sh" "$NVIM_BIN" "$REPO/runtime" "$OUT" "$LABEL"

if [[ -n $CUT ]]; then exit 0; fi

BASELINE=${SPELLMEM_BASELINE:-$("$HERE/baseline.sh" spellmem)}
if diff -q "$BASELINE/base.txt" "$OUT/$LABEL.txt" >/dev/null; then
  echo "report: IDENTICAL"
else
  echo "report: DIFFERS"
  diff -a "$BASELINE/base.txt" "$OUT/$LABEL.txt" | head -60
  exit 1
fi
