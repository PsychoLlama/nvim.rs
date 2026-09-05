#!/usr/bin/env bash
# Build the working tree and diff its fmtsweep against the stored B15
# baseline.  Run from anywhere; everything is absolute.
#
#   fmtverify.sh [label]          # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit bd1d648849 -- B15-2, the P0 prelude, which is the
# rev every B15 oracle baselines against.  It is the *pre-rewrite*
# behaviour of indent_c.rs, textformat.rs and indent.rs: none of them has
# been touched yet, which is the whole point of standing this oracle up
# first.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
#
# All three artifacts are compared, stderr included: nvim's own messages
# go to the prompt, which in a headless process is stderr, and s90 is the
# only place the E474/E487/E518/E524 arms and the 'equalprg'/'formatprg'
# shell-out failures are visible at all.  The report's final `exit N`
# line is the hang/crash assertion -- 124 is the harness timeout and 134
# an abort.
#
# s91 SPAWNS CHILDREN, and it used to expect eleven of them to die:
# `cinoptions` values outside int range aborted the process through
# `getdigits_int`'s `assert!` (charset.rs:586), in the RELEASE build as
# well as the debug one.  P0.3 (`0d483454cc`) made the strict arm clamp
# instead, so all eleven now answer a saturated indent profile and the
# baseline was regenerated with them.  An `"out":"ABORTED"` row appearing
# in s91 again is a regression, not a pass.
#
# THIS SWEEP RUNS NO EXTERNAL FILTER.  s90 points 'equalprg'/'formatprg'
# at a path that does not exist on purpose, because the *message* is the
# artifact; nothing here depends on the host having fmt(1) or indent(1).
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/fmtsweep-out}
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

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.stderr"
"$HERE/fmtsweep.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${FMT_BASELINE:-$("$HERE/baseline.sh" fmt)}
fail=0
for part in txt struct stderr; do
  if cmp -s "$BASELINE/base.$part" "$OUT/$LABEL.$part"; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports carry escaped high bytes, and diff would otherwise
    # call them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would otherwise abort on the
    # first differing artifact, and the remaining ones -- the ones that
    # say *which* layer moved -- would never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
