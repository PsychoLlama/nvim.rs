#!/usr/bin/env bash
# A/B of two nvim binaries on undo.
#
#   undobench.sh <nvim-A> <nvim-B> [rounds]
#   undobench.sh --cachegrind <nvim>
#
# Env: UNDOBENCH_N (changes recorded, default 20000) and UNDOBENCH_ONLY (a
# comma-separated subset of the phases, for a per-phase instruction count;
# the recording always runs); both are read by the Lua and apply to each
# side equally.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=8
. "$HERE/common.sh"

run() {
  VIMRUNTIME="$RUNTIME" $RUNNER "$1" --clean --headless \
    -l "$HERE/undobench.lua" </dev/null 2>/dev/null |
    sed -n 's/^UNDOBENCH\t//p'
}

ab_run undobench
