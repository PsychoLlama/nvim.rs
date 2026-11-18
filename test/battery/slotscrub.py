#!/usr/bin/env python3
"""Mask the battery slot's digit out of a row's artifacts, in place.

    ./slotscrub.py <file-or-dir> [...]

Every run keeps its work directories under one slot root, /tmp/nvb1 ...
/tmp/nvb9 (tmproot.sh), and most rows print paths under it unscrubbed:
`:lcd` echoes, `"<file>" written`, `Last set from`, swap `b0_fname`, an
encoded `tmpslnvb1sl`, a bare `'nvb1'`. Every slot root has the same
length, so nvim lays a message out -- and elides it to fit the headless
screen -- identically in any slot; only the digit differs. This rewrites
that digit to `N` (same length again, so a binary artifact keeps its
layout), which makes a baseline cut in one slot compare with a head run in
another, and lets baseline.sh keep one cut cache for every slot.

The slot root appears whole (`nvb2`) or with its head elided by nvim's
message truncation (`<vb2/`, `<b2/`, `<2/`) or by a middle `...`.
Both sides of a differential pass through this, so a line it masks wrongly
masks the same way on both: it can hide a slot digit, never invent a
difference.
"""

import os
import re
import sys

PATTERN = re.compile(rb"nvb[1-9]|(?:<|\.\.\.)(?:vb|b)?[1-9](?=/)")


def mask(match: re.Match) -> bytes:
    return match.group(0)[:-1] + b"N"


def scrub(path: str) -> None:
    with open(path, "rb") as f:
        data = f.read()
    new = PATTERN.sub(mask, data)
    if new != data:
        with open(path, "wb") as f:
            f.write(new)


def main() -> None:
    for arg in sys.argv[1:]:
        if os.path.isdir(arg):
            for root, _, files in os.walk(arg):
                for name in files:
                    path = os.path.join(root, name)
                    if os.path.isfile(path) and not os.path.islink(path):
                        scrub(path)
        elif os.path.isfile(arg):
            scrub(arg)


if __name__ == "__main__":
    main()
