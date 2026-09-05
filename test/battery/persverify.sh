#!/usr/bin/env bash
# Build the working tree and diff its perssweep against the stored B10
# baseline.  Run from anywhere; everything is absolute.
#
#   persverify.sh [label]        # default label: cur
#
# The baseline lives next to this script in persbase/; see its
# COMMIT file for where it was taken and what carried over from the
# original 54d1dd441d cut.  Regenerate it only when a behaviour change is
# *intended* and reviewed:
#
#   perssweep.sh <nvim> <runtime> test/battery/persbase base
#   rm -rf test/battery/persbase/base.{stderr,masklog} \
#       test/battery/persbase/base-files
#
# Both halves are real stored-baseline differentials.  The shada half used
# to drift with the commit -- the header embeds `nvim.rs <version>` and
# that string's *length* moves every entry offset -- and was worked around
# with a base-vs-HEAD self-differential (p19-close).  P20-4 removed the
# drift instead: shadamask.py rewrites the version to a fixed
# `<prefix> <VERSION>` before anything is decoded.  Proven by running the
# sweep against two binaries whose version strings differ by 28
# characters: txt, struct and hashes all IDENTICAL.
#
# Environment:
#   PERS_BASELINE     baseline directory      (default: beside this script)
#   SWEEP_OUT         where this run's artifacts land
#   PERSSWEEP_NO_MASK set to skip the version masking (pre-P20-4 behaviour)
#
# The sweep's own work directory is a fixed path (/tmp/psweep) because the
# swap files record it in b0_fname and the ShaDa marks record it too; two
# runs from different work directories compare as all-different.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${PERS_BASELINE:-$HERE/persbase}
OUT=${SWEEP_OUT:-/tmp/perssweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log

mkdir -p "$OUT"
cd "$REPO"
if ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -E '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi

rm -rf "${OUT:?}/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.hashes" \
  "$OUT/$LABEL-files"
"$HERE/perssweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -2

fail=0
for part in txt struct hashes; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports carry latin1 bytes, and diff would otherwise call
    # them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` otherwise aborts the script on
    # the first differing artifact, so the remaining ones -- the ones that
    # say *which* layer moved -- are never compared. (Found in B11-1.)
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
