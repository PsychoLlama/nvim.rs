#!/usr/bin/env python3
"""Zero the wall-clock stamps in a .spl/.sug file so two runs compare.

`:mkspell` calls `time(NULL)` and writes the result into both files it
produces: into .spl as the whole payload of the SN_SUGFILE section, and
into .sug right behind the version byte.  The .sug is only accepted if
its stamp matches the .spl's, which is what the pairing is for -- but it
means the bytes differ between any two runs, so a byte-for-byte
differential has to mask exactly those eight bytes and no others.

    ./splscrub.py <file> [...]      # rewrites in place

Prints one `<path> <what-was-scrubbed>` line per file.  Anything that is
not a recognised spell file is left alone and reported as `unknown`,
which is itself a signal: the sweep hashes what comes out, so a format
this script fails to parse shows up as a difference rather than being
silently passed through.
"""

import sys
from pathlib import Path

SPL_MAGIC = b"VIMspell"
SUG_MAGIC = b"VIMsug"
SN_SUGFILE = 11
SN_END = 255


def scrub_spl(b: bytearray) -> list[str]:
    """Walk the section table and zero SN_SUGFILE's eight-byte payload.

    Sections run: id byte, flags byte, 4-byte big-endian length, payload.
    The walk stops at SN_END, after which the word trees begin -- those
    carry no stamp."""
    hits = []
    i = len(SPL_MAGIC) + 1  # magic + version byte
    while i < len(b):
        sid = b[i]
        if sid == SN_END:
            break
        if i + 6 > len(b):
            hits.append("truncated-section-header")
            break
        n = int.from_bytes(b[i + 2 : i + 6], "big")
        payload = i + 6
        if payload + n > len(b):
            hits.append("truncated-section-body")
            break
        if sid == SN_SUGFILE:
            b[payload : payload + n] = b"\0" * n
            hits.append(f"SN_SUGFILE@{payload}")
        i = payload + n
    return hits


def scrub_sug(b: bytearray) -> list[str]:
    """The stamp sits immediately after magic + version byte."""
    off = len(SUG_MAGIC) + 1
    if len(b) < off + 8:
        return ["truncated"]
    b[off : off + 8] = b"\0" * 8
    return [f"sugtime@{off}"]


def main() -> int:
    for arg in sys.argv[1:]:
        p = Path(arg)
        b = bytearray(p.read_bytes())
        if b.startswith(SPL_MAGIC):
            hits = scrub_spl(b) or ["none"]
            kind = "spl"
        elif b.startswith(SUG_MAGIC):
            hits = scrub_sug(b)
            kind = "sug"
        else:
            print(f"{p} unknown")
            continue
        p.write_bytes(bytes(b))
        print(f"{p} {kind} {','.join(hits)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
