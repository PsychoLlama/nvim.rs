#!/usr/bin/env bash
# Build the working tree and diff both halves of the Ex-command probe
# against the stored baseline.
#
#   exverify.sh [label]           # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit cda7f911f3 (the B17 prelude's last rev).  It replaces the phase-14
# `REF-{parse,excmd}-base.txt` taken at 341d60cfea, against which the current
# tree differs on exactly nine rows, all of them accounted for:
#
#   * parse: NONE.  1,946 rows byte-identical across six months and a
#     sandbox change -- that half has never moved.
#   * excmd 1001, 1303, 1304, 1315, 1316: the five recorded ABORTs
#     (`:later 2147483647` and four `nvim_cmd{count=0}` forms), all fixed by
#     P0.3 / P0.4b and now ordinary rows.
#   * excmd 1058 `pwd` and 1064/1065 `checkhealth`: environment leaks,
#     scrubbed and narrowed at B17-5 (see ex-run.sh).
#   * excmd 937/938 `:put +` / `:put! +`: a THIRD environment leak, found
#     while sandboxing.  The old runner inherited the caller's environment,
#     so the clipboard provider was live and `:put +` recorded whatever was
#     in the system clipboard -- two lines of this probe's own scene, on the
#     day the REF was taken.  Under `env -i` there is no provider and the
#     register is empty, every time.
#
# The probe is DIFFERENTIAL, not paired: two artifacts per side, both diffed
# whole.  The `.cases` file exists because `--headless` writes :print/:ls
# output straight to stdout, so a row has to announce itself.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/exprobe-out}
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
  grep -aE '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi

# The baseline is CUT, not committed: `baseline.sh` runs this same probe
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  Cut mode never diffs, so it never resolves a
# baseline -- and must not, since that would re-enter baseline.sh.
BASELINE=
if [[ -z $CUT ]]; then
  BASELINE=${EX_BASELINE:-$("$HERE/baseline.sh" ex)}
fi

fail=0
for probe in parse excmd; do
  rm -f "$OUT/$LABEL-$probe.txt"
  "$HERE/ex-run.sh" "$NVIM_BIN" "$REPO/runtime" \
    "$OUT" "$LABEL" "$probe" | tail -1
  if [[ -n $CUT ]]; then continue; fi
  if diff -q "$BASELINE/base-$probe.txt" "$OUT/$LABEL-$probe.txt" >/dev/null; then
    echo "$probe: IDENTICAL"
  else
    echo "$probe: DIFFERS"
    diff -a "$BASELINE/base-$probe.txt" "$OUT/$LABEL-$probe.txt" | head -40 || true
    fail=1
  fi
done
exit $fail
