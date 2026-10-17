#!/usr/bin/env bash
# A/B of two nvim binaries on the regexp engines.
#
#   rebench.sh <nvim-A> <nvim-B> [rounds]
#   rebench.sh --cachegrind <nvim>
#
# Runs the two alternately in one session and reports the minimum per phase
# for each, plus the percentage change. Pass the same binary twice to size
# the harness noise before quoting any result.
#
# Env: REBENCH_LINES (buffer size, default 6000) and REBENCH_ONLY (a
# comma-separated subset of the phases, for a per-phase instruction count);
# both are read by the Lua and apply to each side equally.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=8
. "$HERE/common.sh"

run() {
  VIMRUNTIME="$RUNTIME" $RUNNER "$1" --clean --headless \
    -l "$HERE/rebench.lua" </dev/null 2>/dev/null |
    sed -n 's/^REBENCH\t//p'
}

ab_run rebench
