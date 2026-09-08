#!/usr/bin/env bash
# A/B of two nvim binaries on the spell subsystem.
#
#   spellbench.sh <nvim-A> <nvim-B> [rounds]
#   spellbench.sh --cachegrind <nvim>
#
# Runs the two alternately in one session and reports the minimum per phase
# for each, plus the percentage change. Cross-session minima drift several
# percent, so only an interleaved run means anything; pass the same binary
# twice to size the harness noise before quoting any result.
#
# The runtime is passed explicitly because the language is the shipped
# runtime/spell/en.utf-8.spl and both sides have to read the same 621 KB.
# VIMRUNTIME may be set to point at another tree's.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/common.sh"

COLS=${SPELLBENCH_COLS:-120}
LINES=${SPELLBENCH_LINES:-50}

run() {
  VIMRUNTIME="$RUNTIME" $RUNNER "$1" --headless \
    -c "set columns=$COLS lines=$LINES" \
    -c "luafile $HERE/spellbench.lua" \
    -c 'qa!' 2>/dev/null | sed -n 's/^SPELLBENCH\t//p'
}

ab_run spellbench
