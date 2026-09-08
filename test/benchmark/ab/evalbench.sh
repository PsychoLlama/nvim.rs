#!/usr/bin/env bash
# A/B of two nvim binaries on the eval substrate.
#
#   evalbench.sh <nvim-A> <nvim-B> [rounds]
#   evalbench.sh --cachegrind <nvim>
#
# The eval half of what inbench.sh does for input: same method, same
# reporting, a disjoint set of phases. inbench has no eval phase at all.
#
# EVALBENCH_SCALE multiplies every phase's round count; use it to make a
# quick run (0.3) or a quieter one (3). It scales both sides equally, so a
# percentage stays comparable, but the *noise floor* does not -- do not
# compare a scaled run's percentages against an unscaled run's floor.
#
# `--headless -c`, not `-l`: see the header of evalbench.lua.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=8
. "$HERE/common.sh"

run() {
  EVALBENCH_SCALE="${EVALBENCH_SCALE:-1}" VIMRUNTIME="$RUNTIME" \
    $RUNNER "$1" --headless -u NONE -i NONE \
    -c "luafile $HERE/evalbench.lua" -c 'qa!' \
    </dev/null 2>/dev/null |
    sed -n 's/^EVALBENCH\t//p'
}

ab_run evalbench
