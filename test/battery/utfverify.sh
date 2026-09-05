#!/usr/bin/env bash
# Build the working tree and diff its utfsweep against the stored B15
# baseline.  Run from anywhere; everything is absolute.
#
#   utfverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in utfbase/ and was
# produced at commit bd1d648849 -- B15-2, the P0 prelude, which is the
# rev every B15 oracle baselines against.  It is the *pre-rewrite*
# behaviour of mbyte.rs and strings.rs: neither has been touched yet,
# which is the whole point of standing this oracle up first.  Regenerate
# it only when a behaviour change is *intended* and reviewed:
#
#   utfsweep.sh <nvim> <runtime> \
#       test/battery/utfbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# ALL FOUR artifacts are compared.
#
#   .txt     the report.  Every byte outside printable ASCII is escaped
#            to \xNN, so a diff is readable.
#   .struct  the same answers as sorted-key JSON, LC_ALL=C sorted, so a
#            reordering shows up as nothing and a value change shows up
#            twice.
#   .bin     the RAW bytes of every string answer.  This is the artifact
#            that says an invalid or overlong sequence came back as the
#            bytes it went in as; the .txt's escaper could in principle
#            normalise a difference away and this one cannot.
#   .stderr  nvim's own messages.  s90 runs the error arms uncaptured
#            (E766/E767/E807/E1500/E1505/E5070/E474/E1174/E1114/E475 and
#            the ++enc failures), so this artifact carries signal.
#
# The report's final `exit N` line is the hang/crash assertion -- 124 is
# the harness timeout and 134 an abort.
#
# THIS SWEEP DEPENDS ON THE HOST'S iconv.  s08 and s09 convert between
# two dozen encoding names; which of them my_iconv_open accepts is a
# property of the host's iconv, not of nvim.  `has-iconv 1` in the
# preamble and `x08 have-iconv 1` at the top of s08 are the tell that
# they ran at all.
#
# s91 SPAWNS CHILDREN.  It was built expecting some of them to die --
# `printf('%3S', "\xe9\xe8\xfc")` underflowed min_field_width to ~2^64
# and nvim answered E41 and exited (O-B15-2) -- and ~62 ABORTED rows were
# the baseline until B15-7 fixed it.  **The baseline was regenerated at
# that commit and s91 now aborts nowhere.**  A new ABORTED row is a
# regression, not an expectation.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${UTF_BASELINE:-$HERE/utfbase}
OUT=${SWEEP_OUT:-/tmp/utfsweep-out}
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

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.bin" "$OUT/$LABEL.stderr"
"$HERE/utfsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt struct bin stderr; do
  if cmp -s "$BASELINE/base.$part" "$OUT/$LABEL.$part"; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports carry escaped high bytes and .bin carries raw ones,
    # and diff would otherwise call them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would otherwise abort on the
    # first differing artifact, and the remaining ones -- the ones that
    # say *which* layer moved -- would never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
