#!/usr/bin/env python3
"""Make ShaDa artifacts version-independent, in place.

Every ShaDa file nvim writes opens with a Header entry whose `version`
field is `LONG_VERSION` -- `nvim.rs ` plus whatever `build.rs` resolved:
a CalVer tag on a tagged checkout, else `dev-<short sha>[-dirty]`, else
`$NVIM_RS_VERSION`, else `unknown`.  None of those has a stable length,
and the length is not confined to the one line that prints the string:
it moves the header entry's payload length, the file's size, and with it
the byte offset of *every* later entry.  That is why `perssweep`'s shada
half was written off as "drifts by design" for four phases (p17-close
item 5, p19-close): a stored baseline could never match a later build.

This script removes the drift at the source.  It rewrites each file so
the version reads `<prefix> <VERSION>` -- the text before the first
space is kept, because "which generator wrote this" is real signal, and
only the varying tail is replaced -- and re-encodes the header entry's
length prefix to match.  Every other byte is spliced through untouched,
so encoding choices (fixmap vs map16, bin8 vs bin16, non-minimal
integers) are still compared exactly.  The result depends on the code
and not on the commit, which is all a stored baseline needs.

    shadamask.py [--report FILE] [--quiet] FILE.shada ...

`--report FILE` additionally rewrites the sweep's text report, where the
same version length leaks as three numbers the driver read back from a
file *before* this ran:

    keep <name>.shada <N> bytes        the artifact's size
    sh-dmg-<label> size <N>            the damaged file's size after :wshada
    ... it occupies <N> bytes ...      E576, quoting a header entry length

Each substitution is guarded by the value: a number is replaced only
when it equals a masked file's recorded pre-mask size, or a pre-mask
header length.  A number this script does not recognise is left alone,
so a real regression stays visible instead of being normalised away.
Files that do not parse (the sweep damages several on purpose) are left
untouched and reported as skipped.

Prints one line per file to stderr; the *masked* version text is fixed,
and the original is deliberately never printed, so nothing this writes
can carry the commit back into an artifact.
"""

import re
import sys
from pathlib import Path

# The replacement for everything after the first space of the version
# string.  Fixed width by construction: it is a literal.
CANON = "<VERSION>"


class Truncated(Exception):
    pass


# leading byte -> (kind, extra); kind drives the reader, mirroring
# shadatok.py's table:
#   v scalar (value in extra) · u/i int of `extra` bytes
#   s/b str/bin with an `extra`-wide length · f float
#   a/m array/map with an `extra`-wide count · x/X ext
FORMATS = {
    0xC0: ("v", None),
    0xC2: ("v", False),
    0xC3: ("v", True),
    0xC4: ("b", 1),
    0xC5: ("b", 2),
    0xC6: ("b", 4),
    0xC7: ("X", 1),
    0xC8: ("X", 2),
    0xC9: ("X", 4),
    0xCA: ("f", 4),
    0xCB: ("f", 8),
    0xCC: ("u", 1),
    0xCD: ("u", 2),
    0xCE: ("u", 4),
    0xCF: ("u", 8),
    0xD0: ("i", 1),
    0xD1: ("i", 2),
    0xD2: ("i", 4),
    0xD3: ("i", 8),
    0xD4: ("x", 1),
    0xD5: ("x", 2),
    0xD6: ("x", 4),
    0xD7: ("x", 8),
    0xD8: ("x", 16),
    0xD9: ("s", 1),
    0xDA: ("s", 2),
    0xDB: ("s", 4),
    0xDC: ("a", 2),
    0xDD: ("a", 4),
    0xDE: ("m", 2),
    0xDF: ("m", 4),
}


def take(b: bytes, pos: int, n: int) -> int:
    if pos + n > len(b):
        raise Truncated("want %d at %d of %d" % (n, pos, len(b)))
    return pos + n


def read(b: bytes, pos: int):
    """Read one msgpack element; return (value, end).

    `value` is an int for integers, a latin1 str for str/bin, and None
    for everything whose value this script never needs to look at."""
    pos = take(b, pos, 1)
    c = b[pos - 1]
    if c <= 0x7F:
        return c, pos
    if c >= 0xE0:
        return c - 0x100, pos
    if 0x80 <= c <= 0x8F:
        kind, extra = "M", c & 0x0F
    elif 0x90 <= c <= 0x9F:
        kind, extra = "A", c & 0x0F
    elif 0xA0 <= c <= 0xBF:
        kind, extra = "S", c & 0x1F
    elif c in FORMATS:
        kind, extra = FORMATS[c]
    else:
        raise Truncated("reserved byte %#x at %d" % (c, pos - 1))

    if kind == "v":
        return extra, pos
    if kind in "ui":
        end = take(b, pos, extra)
        return int.from_bytes(b[pos:end], "big", signed=kind == "i"), end
    if kind == "f":
        return None, take(b, pos, extra)
    if kind in "sb":
        end = take(b, pos, extra)
        n = int.from_bytes(b[pos:end], "big")
        stop = take(b, end, n)
        return b[end:stop].decode("latin1"), stop
    if kind in "SB":
        end = take(b, pos, extra)
        return b[pos:end].decode("latin1"), end
    if kind == "x":
        return None, take(b, pos, extra + 1)
    if kind == "X":
        end = take(b, pos, extra)
        n = int.from_bytes(b[pos:end], "big")
        return None, take(b, end, n + 1)
    if kind in "aA":
        if kind == "a":
            end = take(b, pos, extra)
            n, pos = int.from_bytes(b[pos:end], "big"), end
        else:
            n = extra
        for _ in range(n):
            _, pos = read(b, pos)
        return None, pos
    if kind in "mM":
        if kind == "m":
            end = take(b, pos, extra)
            n, pos = int.from_bytes(b[pos:end], "big"), end
        else:
            n = extra
        for _ in range(n):
            _, pos = read(b, pos)
            _, pos = read(b, pos)
        return None, pos
    raise Truncated("unhandled kind %r" % kind)


def pack_uint(n: int) -> bytes:
    """Minimal-width unsigned int, the rule shada.c's writer follows."""
    if n <= 0x7F:
        return bytes([n])
    if n <= 0xFF:
        return b"\xcc" + n.to_bytes(1, "big")
    if n <= 0xFFFF:
        return b"\xcd" + n.to_bytes(2, "big")
    if n <= 0xFFFFFFFF:
        return b"\xce" + n.to_bytes(4, "big")
    return b"\xcf" + n.to_bytes(8, "big")


def pack_bin(text: str) -> bytes:
    """`bin`, the family shada.c packs the version string with."""
    raw = text.encode("utf-8")
    if len(raw) <= 0xFF:
        return b"\xc4" + bytes([len(raw)]) + raw
    return b"\xc5" + len(raw).to_bytes(2, "big") + raw


def canonical(version: str) -> str:
    """`nvim.rs dev-abc123` -> `nvim.rs <VERSION>`; `seed` -> `<VERSION>`."""
    head, sep, _ = version.partition(" ")
    return head + " " + CANON if sep else CANON


def mask_header_payload(payload: bytes):
    """Replace the `version` value of a Header map.  (new, old_version)."""
    pos = 0
    c = payload[pos] if payload else None
    if c is None:
        return None, None
    if 0x80 <= c <= 0x8F:
        n, pos = c & 0x0F, 1
    elif c == 0xDE:
        n, pos = int.from_bytes(payload[1:3], "big"), 3
    elif c == 0xDF:
        n, pos = int.from_bytes(payload[1:5], "big"), 5
    else:
        return None, None  # not a map: not a header this script understands
    for _ in range(n):
        key, pos = read(payload, pos)
        start = pos
        value, pos = read(payload, pos)
        if key == "version" and isinstance(value, str):
            new = canonical(value)
            if new == value:
                return None, value
            return payload[:start] + pack_bin(new) + payload[pos:], value
    return None, None


def mask_file(path: Path) -> dict:
    """Rewrite one .shada in place.  Raises Truncated if it does not parse."""
    b = path.read_bytes()
    out = []
    pos = 0
    hdr_lens = []
    changed = False
    while pos < len(b):
        entry_start = pos
        typ, pos = read(b, pos)
        _ts, pos = read(b, pos)
        len_start = pos
        length, pos = read(b, pos)
        payload_start = pos
        pos = take(b, pos, length)
        payload = b[payload_start:pos]
        if typ == 1:
            new_payload, _old = mask_header_payload(payload)
            if new_payload is not None:
                out.append(
                    b[entry_start:len_start] + pack_uint(len(new_payload)) + new_payload
                )
                hdr_lens.append((length, len(new_payload)))
                changed = True
                continue
        out.append(b[entry_start:pos])
    new = b"".join(out)
    if changed:
        path.write_bytes(new)
    return {
        "path": path,
        "orig_size": len(b),
        "new_size": len(new),
        "hdr_lens": hdr_lens,
        "changed": changed,
    }


KEEP_RE = re.compile(r"^(\s*keep )(\S+\.shada)( )(\d+)( bytes)$")
DMG_RE = re.compile(r"^(sh-dmg-)(\S+)( size )(\d+)$")
OCCUPIES_RE = re.compile(r"(it occupies )(\d+)( bytes)")


def rewrite_report(report: Path, info: list) -> int:
    """Fix the three places the pre-mask sizes reached the text report."""
    by_name = {}
    for i in info:
        by_name.setdefault(i["path"].name, i)
    hdr_map = {}
    for i in info:
        for old, new in i["hdr_lens"]:
            hdr_map[old] = new

    hits = 0
    lines = report.read_text(encoding="latin1").split("\n")
    for n, line in enumerate(lines):
        m = KEEP_RE.match(line)
        if m:
            i = by_name.get(m.group(2))
            if i and int(m.group(4)) == i["orig_size"]:
                lines[n] = "%s%s%s%d%s" % (
                    m.group(1),
                    m.group(2),
                    m.group(3),
                    i["new_size"],
                    m.group(5),
                )
                hits += 1
            continue
        m = DMG_RE.match(line)
        if m:
            i = by_name.get("dmg-%s.shada" % m.group(2))
            if i and int(m.group(4)) == i["orig_size"]:
                lines[n] = "%s%s%s%d" % (
                    m.group(1),
                    m.group(2),
                    m.group(3),
                    i["new_size"],
                )
                hits += 1
            continue

        # `subn`'s count would include the matches this leaves alone, so
        # the substitutions are counted by hand instead.
        taken = []

        def sub(m):
            new = hdr_map.get(int(m.group(2)))
            if new is None:
                return m.group(0)
            taken.append(new)
            return "%s%d%s" % (m.group(1), new, m.group(3))

        fixed = OCCUPIES_RE.sub(sub, line)
        if taken:
            lines[n] = fixed
            hits += len(taken)
    report.write_text("\n".join(lines), encoding="latin1")
    return hits


def main() -> int:
    args = sys.argv[1:]
    report = None
    quiet = False
    files = []
    while args:
        a = args.pop(0)
        if a == "--report":
            report = Path(args.pop(0))
        elif a == "--quiet":
            quiet = True
        else:
            files.append(Path(a))
    if not files:
        sys.stderr.write(__doc__)
        return 2

    info = []
    for path in sorted(files):
        try:
            i = mask_file(path)
        except (Truncated, IndexError) as e:
            if not quiet:
                print("shadamask: %s SKIPPED (%s)" % (path.name, e), file=sys.stderr)
            continue
        info.append(i)
        if not quiet:
            print(
                "shadamask: %s %s %d -> %d bytes"
                % (
                    path.name,
                    "masked" if i["changed"] else "unchanged",
                    i["orig_size"],
                    i["new_size"],
                ),
                file=sys.stderr,
            )
    if report is not None and info:
        hits = rewrite_report(report, info)
        if not quiet:
            print(
                "shadamask: report %s, %d numbers" % (report.name, hits),
                file=sys.stderr,
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
