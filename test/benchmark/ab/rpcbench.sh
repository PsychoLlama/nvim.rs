#!/usr/bin/env bash
# A/B of two nvim binaries on the RPC transport, the job channel and the
# remote UI.
#
#   rpcbench.sh <nvim-A> <nvim-B> [rounds]
#   rpcbench.sh --cachegrind <nvim>
#
# Each phase talks to a child process -- an `--embed` instance of the same
# binary (`v:progpath`) or `cat` -- and only this process is measured; see
# the header of rpcbench.lua for the phase map.
#
# RPCBENCH_SCALE multiplies every phase's round count; RPCBENCH_ONLY runs a
# comma-separated subset of the phases after the setup.
#
# Noise, measured: three `--cachegrind` runs of one codegen-units = 1 release
# binary spread 0.24 % on the whole (4,302-4,312 M Ir). Per phase, three runs
# each, setup (`RPCBENCH_ONLY=none`, 166 M) subtracted: chanjob 0.39 %,
# rpcreq 0.33 %, uilua 0.22 %, rpcnotify 0.19 %, uiattach 0.05 %, objarr
# 0.03 %, ctl 0.01 %. The phases that wait on a child read its timing back
# as event-loop turns, so treat a difference under ~0.5 % as none.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=8
. "$HERE/common.sh"

run() {
  RPCBENCH_SCALE="${RPCBENCH_SCALE:-1}" VIMRUNTIME="$RUNTIME" \
    $RUNNER "$1" --headless -u NONE -i NONE \
    -c "luafile $HERE/rpcbench.lua" -c 'qa!' \
    </dev/null 2>/dev/null |
    sed -n 's/^RPCBENCH\t//p'
}

ab_run rpcbench
