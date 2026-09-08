#!/usr/bin/env bash
# A/B of two nvim binaries on the screen pipeline.
#
#   scrbench.sh <nvim-A> <nvim-B> [rounds]
#   scrbench.sh --cachegrind <nvim>
#
# Runs the two alternately in one session and reports the minimum per phase
# for each, plus the percentage change. Cross-session minima drift several
# percent on this machine, so only an interleaved run means anything.
# Pass the same binary twice to size the harness noise before quoting any
# result -- a phase whose self-A/B swing exceeds the effect resolves nothing.
#
# This canary needs `--headless -c` rather than `--headless -l`, because only
# the former sets full_screen, and without full_screen 'columns' does not
# resize the grid. See the header of scrbench.lua.
#
# VIMRUNTIME may be set to measure against a runtime other than this tree's.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/common.sh"

COLS=${SCRBENCH_COLS:-200}
LINES=${SCRBENCH_LINES:-60}

run() {
  VIMRUNTIME="$RUNTIME" $RUNNER "$1" --headless \
    -c "set columns=$COLS lines=$LINES" \
    -c "luafile $HERE/scrbench.lua" \
    -c 'qa!' 2>/dev/null | sed -n 's/^SCRBENCH\t//p'
}

ab_run scrbench
