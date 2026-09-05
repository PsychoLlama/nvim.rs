#!/usr/bin/env bash
# Build the working tree and diff its perssweep against the stored B10
# baseline.  Run from anywhere; everything is absolute.
#
#   persverify.sh [label]        # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit 54d1dd441d (B10-1), re-cut at
# 96d70c6f79 once shadamask.py gave the ShaDa header's version a fixed
# width -- before that the shada half could never match a later build --
# and again at 041fcdbea1 for the `twolevel` swap case.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
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
OUT=${SWEEP_OUT:-/tmp/perssweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log
# Cut mode.  `baseline.sh` re-enters this script as `--cut <nvim> <dir>` to
# cut the pinned baseline from the reference binary; it shares the ONE sweep
# call below with the head run, so the two sides of the differential cannot
# drift apart.  It skips the build and the diff.
CUT=
if [[ ${1:-} == --cut ]]; then CUT=$2; OUT=$3; LABEL=base; LOG=/dev/null; fi
NVIM_BIN=${CUT:-$REPO/target/debug/nvim}

mkdir -p "$OUT"
cd "$REPO"
if [[ -z $CUT ]] && ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -E '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi

rm -rf "${OUT:?}/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.hashes" \
  "$OUT/$LABEL-files"
"$HERE/perssweep.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -2

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${PERS_BASELINE:-$("$HERE/baseline.sh" pers)}
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
