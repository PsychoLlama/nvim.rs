#!/usr/bin/env bash
# Build the working tree and diff its cmdsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   cmdverify.sh [label]          # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit 4406cb4c93 -- the P0.4b slice's own crash-fix
# commit, which is the first revision at which s91 aborts nowhere --
# RE-TAKEN at 2ecc9b69f7 (B16-5's sixteen :runtime / :source / :packadd /
# :scriptnames shapes in s91) and RE-TAKEN AGAIN at **cda7f911f3**, the
# B17 prelude's last revision, when B17-5 added the `:s` / `:g` guard the
# batch was missing: s10's c10c/c10f/c10e/c10m/c10i, s11's c11n, s17's
# c17g/c17q (:helpgrep and :helptags), the new s18 (ex_substitute_preview,
# driven from an --embed child over RPC) and nineteen s91 shapes for
# ex_session / help / digraph.  1,294 report lines added; 23 removed, all
# accounted for -- six from clearing the undo history in `reset` and
# fourteen from clearing the command/search histories there (both were
# monotonic counters that made s13's and s17's answers a function of how
# many cases had run above them), and three from two new scrubs
# (`<SCRIPT>:N` in a Lua traceback and `<SCRIPT> line N` in `:command`'s
# "Last set from", either of which re-baselined a section on any edit to
# the sweep).  s91 still aborts nowhere, so an ABORTED row is a
# regression.  It is
# the *pre-rewrite* behaviour of everything B17 touches: ex_cmds.rs,
# ex_cmds2.rs, ex_eval.rs, ex_session.rs, usercmd.rs, debugger.rs,
# help.rs, digraph.rs, cmdhist.rs and arglist, plus ex_docmd's parser and
# the two api/command entry points.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
#
# All three artifacts are compared, stderr included: s20 runs a block of
# commands *uncaptured* with 'report' at 0, and that artifact is the only
# view of the message path ("N fewer lines", "N substitutions on N
# lines", the :command / :history listings, the E-numbers).  The report's
# final `exit N` line is the hang/crash assertion -- 124 is the harness
# timeout and 134 an abort -- and s91's rows are the per-input version of
# the same question.
#
# Requires `runtime/doc/tags` to exist (s17 drives `:help`).  It is
# generated and untracked, so a throwaway worktree needs it copied in;
# the report's `runtime-tags` header line says whether it was there.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/cmdsweep-out}
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
"$HERE/cmdsweep.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${CMD_BASELINE:-$("$HERE/baseline.sh" cmd)}
fail=0
for part in txt struct stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
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
