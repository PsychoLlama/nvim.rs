#!/usr/bin/env bash
# Build the working tree and diff its sessgold against the stored baseline.
#
#   sessverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in sessbase/, produced
# at commit cda7f911f3 (the B17 prelude's last rev, and B17-5's re-baseline
# point).  It is the pre-rewrite byte-for-byte behaviour of ex_session.rs:
# makeopens, put_view, ses_*, store_session_globals, get_view_file, ex_mkrc
# and ex_loadview.  `:mksession` is an ON-DISK FORMAT -- a session file that
# is subtly wrong still sources back cleanly, which is why the oldtest suite
# cannot see the regression this oracle exists to catch.  Regenerate it only
# when a behaviour change is intended and reviewed:
#
#   sessgold.sh <nvim> <runtime> \
#       test/battery/sessbase base
#
# ... and `just build` first: a mutation harness leaves the binary built from
# its last mutant, and a baseline taken from that compares mutant against
# mutant forever after.
#
# NOTHING but the work directory and $VIMRUNTIME is normalised.  Every digit
# in a session file is load-bearing -- window sizes, `badd +N`, `normal! 016|`,
# `exe '1resize ' . ((&lines * 11 + 12) / 24)`, the fold line numbers and the
# cursor -- so a digit-normalising scrub (cmdsweep's s17s) is blind to exactly
# what this file exists to gate.
#
# Requires `runtime/doc/tags` (the `buffers` scene opens `:help help.txt`).
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${SESS_BASELINE:-$HERE/sessbase}
OUT=${SWEEP_OUT:-/tmp/sessgold-out}
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

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.stderr"
"$HERE/sessgold.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
