#!/usr/bin/env bash
# Build the working tree and diff its sessgold against the stored baseline.
#
#   sessverify.sh [label]          # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit cda7f911f3 (the B17 prelude's last rev, and B17-5's re-baseline
# point).  It is the pre-rewrite byte-for-byte behaviour of ex_session.rs:
# makeopens, put_view, ses_*, store_session_globals, get_view_file, ex_mkrc
# and ex_loadview.  `:mksession` is an ON-DISK FORMAT -- a session file that
# is subtly wrong still sources back cleanly, which is why the oldtest suite
# cannot see the regression this oracle exists to catch.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
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
OUT=${SWEEP_OUT:-/tmp/sessgold-out}
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

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.stderr"
"$HERE/sessgold.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${SESS_BASELINE:-$("$HERE/baseline.sh" sess)}
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
