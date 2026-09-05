#!/usr/bin/env bash
# Build the working tree and diff its stlsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   stlverify.sh [label]          # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit 5e23ad6128 -- B18-16, the last revision before any
# of statusline.rs is carved or rewritten.  It is the *pre-rewrite*
# behaviour of `build_stl_str_hl` (1,751 lines: the item switch, the
# group stack, the min/max width arithmetic, `%<`, `%=`, `%{}`/`%{%..%}`
# /`%!` and the click-definition arenas) plus `win_redr_custom`,
# `redraw_ruler` and `draw_tabline`.  It was also run against
# ~/agents/scratch/b19-1/nvim-5e23ad6128 (B19's pre-batch side),
# ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's) and
# ~/agents/scratch/p0-2/nvim-ed789235ab (phase 16's), and is IDENTICAL
# on all three -- which is the proof that this family has not moved
# since P0.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
#
# All three artifacts are compared, stderr included: s5 and s9 run
# failing redraws through `-c` in children, which is the only spelling
# under which nvim *displays* an error and keeps going, so that artifact
# is the only view of this family's message path (E539/E540/E542, E992,
# and the E117/E121/E605 an expression item raises).  In process,
# `vim.cmd` turns a Vimscript error into a Lua error and `pcall`
# swallows it.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  Those
# counts are the standing assertion that no section went silently empty;
# at the baseline they are:
#
#   s1-items 477 · s2-width 177 · s3-groups 100 · s4-trunc 392 ·
#   s5-eval 99 · s6-click 71 · s7-statuscol 1605 · s8-others 131 ·
#   s9-errors 146 · s91-crashprobe 43
#
# ... and the artifacts are 3,263 / 3,174 / 54 lines.  A `rows=0` with
# everything else unchanged is a harness bug, not a regression.
#
# TWO ROWS ARE THE LOAD-BEARING CONSTANTS.  `s5/depth/grow#count` is
# 120 -- one hundred `a`s plus the twenty-character literal left over
# when MAX_STL_EVAL_DEPTH runs out -- and it is the only direct
# measurement of that constant anywhere.  `k91 groups cases=42
# aborted=0`: s91 aborts NOWHERE at the baseline, so any ABORTED row is
# a regression, not a re-baseline.
#
# The report's final `exit N` line is the hang/crash assertion -- 124 is
# the harness timeout and 134 an abort.
#
# Does NOT need `runtime/doc/tags`.  It DOES read $VIMRUNTIME: `:help`
# is one of s1's nine buffer states, so `%F` answers `<RT>/doc/help.txt`
# -- pass the same runtime to both sides of a paired run.
#
# TAKES ~3 s.  s91's 42 children are the bulk of it.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/stlsweep-out}
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
"$HERE/stlsweep.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${STL_BASELINE:-$("$HERE/baseline.sh" stl)}
fail=0
for part in txt struct stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports escape high bytes, and diff would otherwise call
    # them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would abort on the first
    # differing artifact and the ones that say *which* layer moved would
    # never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
