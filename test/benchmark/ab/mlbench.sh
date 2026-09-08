#!/usr/bin/env bash
# A/B of two nvim binaries on the memline hot path.
#
#   mlbench.sh <nvim-A> <nvim-B> [rounds]
#   mlbench.sh --cachegrind <nvim>
#
# mlbench.lua prints "phase<space>ms" and is driven with `-l`, not `-c`:
# nothing it measures reaches the screen, so script mode is fine and is
# what the corpus builder expects.
#
# Env: MLBENCH_LINES (default 120000), MLBENCH_REPS (default 1); both are
# read by the Lua and apply to each side equally.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=12
. "$HERE/common.sh"

run() {
  VIMRUNTIME="$RUNTIME" $RUNNER "$1" --clean --headless \
    -l "$HERE/mlbench.lua" 2>/dev/null |
    awk '$1 != "TOTAL" && NF >= 2 { printf "%s\t%s\n", $1, $2 }'
}

ab_run mlbench
