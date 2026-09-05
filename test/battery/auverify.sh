#!/usr/bin/env bash
# Build the working tree and diff its ausweep against the stored B14
# baseline.  Run from anywhere; everything is absolute.
#
#   auverify.sh [label]           # default label: cur
#
# The baseline lives next to this script in aubase/ and was
# produced from commit 3273bf014a -- B14-4's assertion-demotion commit,
# i.e. after the eval prelude and before autocmd.rs is rewritten
# (B14-15/16).  Regenerate it only when a behaviour change is *intended*
# and reviewed:
#
#   ausweep.sh <nvim> <runtime> \
#       test/battery/aubase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# The sweep's own work directory is a fixed short path (/tmp/ausweep) and
# the report is scrubbed of it; the run is cwd-independent (verified from
# a second directory), but keep the fixed path anyway -- `<afile>`, the
# pattern-matching section and several stderr messages name files.
#
# All three artifacts are compared, stderr included: nvim's own messages
# go to the prompt, which in a headless process is stderr, and the
# `:autocmd` listings, W19 and two regexp errors from the pattern corpus
# reach only that artifact.  The report's final `exit N` line is the
# hang/crash assertion -- 124 is the harness timeout and 134 an abort,
# and a firing walk that fails to pop `active_apc_list` produces either.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${AU_BASELINE:-$HERE/aubase}
OUT=${SWEEP_OUT:-/tmp/ausweep-out}
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

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.stderr"
"$HERE/ausweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt struct stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports carry escaped bytes, and diff would otherwise call
    # them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would otherwise abort on the
    # first differing artifact, and the remaining ones -- the ones that
    # say *which* layer moved -- would never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
