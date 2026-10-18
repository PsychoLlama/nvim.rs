#!/usr/bin/env bash
# A/B of two nvim binaries on `:mkspell` without a .sug file.
#
#   mkspellbench.sh <nvim-A> <nvim-B> [rounds]
#   mkspellbench.sh --cachegrind <nvim>
#
# MKSPELLBENCH_WORK names the work directory (default /tmp/mkspellbench):
# the generated .aff/.dic and the .spl land there. MKSPELLBENCH_ONLY runs a
# subset of the phases; MKSPELLBENCH_WORDS sizes the input (default 100000).
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/common.sh"

WORK=${MKSPELLBENCH_WORK:-/tmp/mkspellbench}
mkdir -p "$WORK"

run() {
  MKSPELLBENCH_WORK="$WORK" VIMRUNTIME="$RUNTIME" $RUNNER "$1" --clean --headless \
    -l "$HERE/mkspellbench.lua" 2>/dev/null | sed -n 's/^MKSPELLBENCH\t//p'
}

ab_run mkspellbench
