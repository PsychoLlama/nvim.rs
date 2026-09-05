#!/usr/bin/env bash
# Build the working tree and diff its resweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   reverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in rebase/ and was
# produced at commit 5c829b911e -- phase 17's base, the last revision
# before anything in the regexp family is threaded, split or rewritten.
# It is the *pre-rewrite* behaviour of `vim_regcomp` over every magic
# level, of `vim_regexec_nl` / `vim_regexec_multi` / `vim_regexec_prog`,
# of the backtracking engine AND the NFA engine independently, of the
# zero-width and position atoms, of the submatch bookkeeping, of the
# `substitute()` replacement alphabet and of every `E` code the parsers
# raise.
#
# WHY IT EXISTS.  Of the twenty-two differentials that came before it,
# exactly one -- `opsweep` -- reaches the regexp engines at all, and only
# through `:s` on a handful of patterns.  NOTHING anywhere set
# `'regexpengine'`, so one of the two engines was covered by accident and
# the other not at all.  `rex`, the `regexec_T` both engines thread
# through themselves, is the largest single `cell_ptr` consumer in the
# tree; a bug in it shows on ONE engine and not the other, and before
# this sweep nothing could tell them apart.
#
# THE AXIS IS `'regexpengine'`.  Every corpus row runs three times:
# `re=0` (NFA, falling back to BT when compilation fails -- the default,
# and the only setting that exercises the fallback), `re=1` (BT only)
# and `re=2` (NFA only).
#
# THREE ARTIFACTS, TWO LAYERS, and which one moves tells you what
# happened:
#
#   base.txt     one line per case: the answers of every driver the case
#                ran, with the pattern and subject spelled out.  Moving
#                ALONE is a match-behaviour change a user can see.
#   base.err     the MESSAGE layer: every error any driver raised, in
#                emission order, normalised to its `E<n>: <text>` tail.
#                Moving ALONE means an `E` code or a message text moved
#                with every match answer still right -- which is what a
#                rewritten parser gets wrong first.
#   base.stderr  EMPTY at the baseline, and that is the assertion.
#                `:g`/`:v` report "Pattern not found" and "Pattern found
#                in every line" through `smsg`, NOT `emsg`, so `pcall`
#                never sees them and a headless run writes them to the
#                prompt: r7 runs those two under `silent`.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  At the
# baseline they are:
#
#   r0-canary 10 · r1-magic 183 · r2-atoms 201 · r3-multi 81 ·
#   r4-zero 114 · r5-case 204 · r6-drivers 105 · r7-multiline 54 ·
#   r8-subst 93 · r9-errors 132 · r10-levers 3 · r91-abortprobe 17
#
# ... summing to `## TOTAL rows=1197`, with artifacts 1,211 / 262 / 0
# lines (the report adds the twelve section lines, `## TOTAL` and
# `exit 0`).  A `rows=0` with everything else unchanged is a harness
# bug, not a regression.
#
# BASELINED FOUR-WAY ON DAY ONE, WITH NO EXCEPTION SET.  All three
# artifacts are byte-identical on every kept side:
# ~/agents/scratch/b22-2/nvim-44c6e8d630, b21-2/nvim-74ff723db9,
# b17-19/nvim-b00f1ef7e0 and p0-2/nvim-ed789235ab.  The
# `cinoptions=>2147483648` abort that separates `ed789235ab` from the
# others lives in charset.rs and no case here goes near it.  ANY diff on
# ANY side is a regression.
#
# SIX ROWS ARE THE LOAD-BEARING ASSERTIONS.
#
#   * `r0/enginemark-*` -- the two engines' own fingerprints, spelled
#     out: `a**` is E61 to the backtracking parser and E871 to the NFA
#     one, `\{1}` is E64 vs E866, and a NESTED `\%[]` is E369 to BT and
#     compiles to NFA.  All three say `differ=true`.  If they stop
#     differing, `'regexpengine'` is selecting nothing and two thirds of
#     this sweep is measuring one engine twice.
#   * `r0/override/{0,1,2}` -- `\%#=N` beats the option.  Row 1 answers
#     E61 under BOTH settings of `'regexpengine'` and rows 0 and 2
#     answer E871 under both: that IS the override.
#   * `r10/enginedelta rows=17` and `r10/deltalist` -- the corpus rows on
#     which `re=1` and `re=2` disagree, and their names.  Fifteen is not
#     a target, it is a fingerprint: if it collapses toward zero one
#     engine has stopped being reached, and if the LIST changes without
#     the count changing, a divergence moved.  FIFTEEN of the sixteen are
#     `E`-code disagreements; the sixteenth, `r2/equi-A-u200`, is a
#     BEHAVIOURAL divergence and an upstream defect (the `[[=x=]]` tables
#     are kept once per engine and drifted -- `regexp-equi-class-engines-
#     disagree.md`).  Do not "fix" it: the port reproduces it deliberately
#     and `regexp/equi_class.rs` names it in `EquiClass::nfa_only`.
#   * `r0/magicagree agree=9/9` -- `\v`, `\m`, `\M` and `\V` spelling ONE
#     meaning answer one thing.  The magic level is a parser-wide mode;
#     this is the cheapest possible statement that it is still one.
#   * `r0/stable same=true` -- the same probe twice is byte-identical.
#     The oracle's own determinism.
#   * `r91/groups cases=16 aborted=0`.  THE SWEEP HAS NO EXPECTED ABORT:
#     any abort at all is a regression, by construction.  `r91/
#     catastrophic-bt` deliberately feeds `\(a*\)*b` to the backtracking
#     engine, so every child runs under `timeout -k 2 30` and a hang
#     shows as `rc=124`, not as a wedged battery.
#
# TWO ENGINE DIVERGENCES ARE BASELINE FACTS, not bugs to chase:
# `r91/brace-huge` (`a\{1,100000}` matches under BT and does NOT match
# under NFA) and `r91/invalid-utf8` (`match("a\xff\xfeb", "a\xff")` is
# -1 under BT and 0 under NFA).  Both are upstream's behaviour; see
# `p17-2-oracles.md`.
#
# TAKES ~8 s in the sweep plus the build.  Sixteen `--headless` children
# (r91), each reaped before its row is printed.  If a run is killed, look
# for children with ppid 1 and KILL THEM BY PID -- never by pattern.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change.  Nothing here prints a buffer or window HANDLE, and no row
# carries a wall clock, an address or a `pairs()` order.
#
# Regenerate the baseline only when a behaviour change is *intended* and
# reviewed:
#
#   resweep.sh <nvim> <runtime> \
#       test/battery/rebase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${RE_BASELINE:-$HERE/rebase}
OUT=${SWEEP_OUT:-/tmp/resweep-out}
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

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.err" "$OUT/$LABEL.stderr"
"$HERE/resweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt err stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # `|| true`: `set -e` plus `pipefail` would abort on the first
    # differing artifact and the ones that say *which* layer moved would
    # never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
