#!/usr/bin/env bash
# A/B of two nvim binaries on the input / command-line layer.
#
#   inbench.sh <nvim-A> <nvim-B> [rounds]
#   inbench.sh --cachegrind <nvim>
#
# MEASURED HARNESS NOISE (self-A/B, same binary both sides). At 8 rounds:
# TOTAL -0.2%, every phase within 0.9%. At 5 rounds the same setup gave
# TOTAL 1.0% with `cmdline` at 5.7% -- so 8 is the default and 5 is not
# enough. Two phases had to be resized to get there: `termcodes` and
# `msghist` were 21 ms and 12 ms and swung 3%, which resolves nothing; both
# now run ~60-90 ms.
#
# INBENCH_WORK names the fixture directory (20,000 files), default
# /tmp/inbench; the untimed warm-up run pays for building it.
#
# `--headless -c`, not `-l`: see the header of inbench.lua.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=8
. "$HERE/common.sh"

WORK=${INBENCH_WORK:-/tmp/inbench}

run() {
  INBENCH_WORK="$WORK" VIMRUNTIME="$RUNTIME" \
    $RUNNER "$1" --headless -u NONE -i NONE \
    -c "luafile $HERE/inbench.lua" -c 'qa!' \
    </dev/null 2>/dev/null |
    sed -n 's/^INBENCH\t//p'
}

ab_run inbench
