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
# `--cachegrind` answers TWO numbers: `evalbench` is the phases that
# existed before the parser ones (EVALBENCH_GROUP=old), so it stays
# comparable with every Ir recorded before them, and `evalbench-parser` is
# the four parser phases alone (EVALBENCH_GROUP=parser). Wall-clock mode
# runs every phase; set EVALBENCH_GROUP to narrow it.
#
# `--headless -c`, not `-l`: see the header of evalbench.lua.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=8
. "$HERE/common.sh"

run() {
  EVALBENCH_SCALE="${EVALBENCH_SCALE:-1}" VIMRUNTIME="$RUNTIME" \
    EVALBENCH_GROUP="${EVALBENCH_GROUP:-all}" \
    $RUNNER "$1" --headless -u NONE -i NONE \
    -c "luafile $HERE/evalbench.lua" -c 'qa!' \
    </dev/null 2>/dev/null |
    sed -n 's/^EVALBENCH\t//p'
}

if [ "$MODE" = cachegrind ]; then
  # Each in a subshell: ab_run's EXIT trap removes its own scratch.
  (EVALBENCH_GROUP=old ab_run evalbench)
  (EVALBENCH_GROUP=parser ab_run evalbench-parser)
else
  ab_run evalbench
fi
