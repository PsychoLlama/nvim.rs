#!/usr/bin/env bash
# Build the working tree and diff both halves of the Ex-command probe
# against the stored baseline.
#
#   exverify.sh [label]           # default label: cur
#
# The baseline lives next to this script in exbase/, produced at
# commit cda7f911f3 (the B17 prelude's last rev).  It replaces the phase-14
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
BASELINE=${EX_BASELINE:-$HERE/exbase}
OUT=${SWEEP_OUT:-/tmp/exprobe-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log

mkdir -p "$OUT"
cd "$REPO"
if ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -aE '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi

fail=0
for probe in parse excmd; do
  rm -f "$OUT/$LABEL-$probe.txt"
  "$HERE/ex-run.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
    "$OUT" "$LABEL" "$probe" | tail -1
  if diff -q "$BASELINE/base-$probe.txt" "$OUT/$LABEL-$probe.txt" >/dev/null; then
    echo "$probe: IDENTICAL"
  else
    echo "$probe: DIFFERS"
    diff -a "$BASELINE/base-$probe.txt" "$OUT/$LABEL-$probe.txt" | head -40 || true
    fail=1
  fi
done
exit $fail
