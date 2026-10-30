#!/usr/bin/env bash
# A/B of two nvim binaries on extmarks and the marktree.
#
#   markbench.sh <nvim-A> <nvim-B> [rounds]
#   markbench.sh --cachegrind <nvim>
#
# Runs the two alternately in one session and reports the minimum per phase
# for each, plus the percentage change; see the header of markbench.lua for
# the phase map and the measured noise. Like scrbench this needs
# `--headless -c` (full_screen, so the `decor` phase draws a real grid).
#
# Env: MARKBENCH_SCALE multiplies every phase's round count; MARKBENCH_ONLY
# runs a comma-separated subset of the phases after the fixture.
# VIMRUNTIME may be set to measure against a runtime other than this tree's.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=8
. "$HERE/common.sh"

run() {
  MARKBENCH_SCALE="${MARKBENCH_SCALE:-1}" VIMRUNTIME="$RUNTIME" \
    $RUNNER "$1" --headless -u NONE -i NONE \
    -c 'set columns=200 lines=60' \
    -c "luafile $HERE/markbench.lua" \
    -c 'qa!' </dev/null 2>/dev/null | sed -n 's/^MARKBENCH\t//p'
}

ab_run markbench
