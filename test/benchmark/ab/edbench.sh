#!/usr/bin/env bash
# A/B of two nvim binaries on the editing core: yank/put, join, shift and
# indent, formatting, text objects, folds, diff, autocommands and marks.
#
#   edbench.sh <nvim-A> <nvim-B> [rounds]
#   edbench.sh --cachegrind <nvim>
#
# Env: EDBENCH_ONLY (a comma-separated subset of the phases, for a per-phase
# instruction count); read by the Lua and applied to each side equally.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=8
. "$HERE/common.sh"

run() {
  VIMRUNTIME="$RUNTIME" $RUNNER "$1" --clean --headless \
    -l "$HERE/edbench.lua" </dev/null 2>/dev/null |
    sed -n 's/^EDBENCH\t//p'
}

ab_run edbench
