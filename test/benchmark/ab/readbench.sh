#!/usr/bin/env bash
# A/B of two nvim binaries on reading a file.
#
#   readbench.sh <nvim-A> <nvim-B> [rounds]
#   readbench.sh --cachegrind <nvim>
#
# READBENCH_WORK names the fixture directory, default /tmp/readbench. It
# holds two 100 MB inputs (`big.txt`, UTF-8 with an `é` on 30 % of lines;
# `lat.txt`, the same with a Latin-1 `\xe9`), written once by a seeded
# generator and reused while their sizes match. They are not committed.
#
# Wall clock here is dominated by the kernel's page cache and by glibc
# returning freed blocks to the system; read Ir first (README.md), and take
# the wall-clock run on a quiet machine, both orders.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROUNDS_DEFAULT=6
. "$HERE/common.sh"

WORK=${READBENCH_WORK:-/tmp/readbench}
mkdir -p "$WORK"

# Python's `random` is specified to reproduce a seeded sequence across
# versions, so every machine builds the same bytes.
python3 - "$WORK" <<'PY'
import os
import random
import sys

work = sys.argv[1]
SIZE = 100_000_000
WORDS = [
    w.encode()
    for w in "the quick brown fox jumps over a lazy dog lorem ipsum dolor sit "
    "amet consectetur adipiscing elit".split()
]


def gen(hi):
    out = bytearray()
    while len(out) < SIZE:
        n = random.randint(0, 18)
        line = b" ".join(random.choice(WORDS) for _ in range(n))
        if random.random() < 0.3:
            line += b" caf" + hi
        out += line + b"\n"
    return bytes(out)


# One stream for both files, big first, so either is rebuilt only with both.
paths = [os.path.join(work, name) for name in ("big.txt", "lat.txt")]
if not all(os.path.exists(p) and os.path.getsize(p) >= SIZE for p in paths):
    random.seed(7)
    for path, hi in zip(paths, ("é".encode(), b"\xe9")):
        with open(path + ".tmp", "wb") as f:
            f.write(gen(hi))
        os.replace(path + ".tmp", path)
PY

run() {
  READBENCH_WORK="$WORK" VIMRUNTIME="$RUNTIME" $RUNNER "$1" --clean --headless \
    -l "$HERE/readbench.lua" </dev/null 2>/dev/null |
    sed -n 's/^READBENCH\t//p'
}

ab_run readbench
