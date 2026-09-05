#!/usr/bin/env python3
"""Tokenise a ShaDa file down to its msgpack encoding decisions.

A ShaDa file is a bare sequence of entries; each entry is three msgpack
integers -- type, timestamp, payload length -- followed by that many
bytes holding one more msgpack value.  shada.c hand-rolls the packing,
so the *choice* of encoding (fixint vs uint8 vs uint32, fixstr vs str8,
fixmap vs map16) is part of what a rewrite has to preserve, and a plain
`msgpack.unpack` comparison would not see it.

    ./shadatok.py <file.shada> [...]

prints one line per msgpack element carrying the format token and the
value, nested elements indented.  Two things are masked, because they
are the only parts of a ShaDa file that are not a function of its
inputs: every entry timestamp and the header entry's `pid`.  Both keep
their format token where the token is width-stable (timestamps are
seconds since the epoch and will not leave uint32 this century); the
pid's token is masked too, since a pid can land in any of three widths.

Also available:

    ./shadatok.py --pack <spec.json> <out.shada>

writes a ShaDa file from a JSON description, so the merge and
damaged-file paths get a *fixed* input instead of one carrying whatever
timestamps the previous run happened to produce.  The spec is a list of
`[type, timestamp, value]`; values are encoded with the canonical
minimal-width rules, which is all the reader needs.
"""

import json
import sys
from pathlib import Path

ENTRY_NAMES = {
    0: "Missing",
    1: "Header",
    2: "SearchPattern",
    3: "SubString",
    4: "HistoryEntry",
    5: "Register",
    6: "Variable",
    7: "GlobalMark",
    8: "Jump",
    9: "BufferList",
    10: "LocalMark",
    11: "Change",
}


class Truncated(Exception):
    pass


class Reader:
    def __init__(self, buf: bytes):
        self.buf = buf
        self.pos = 0

    def take(self, n: int) -> bytes:
        if self.pos + n > len(self.buf):
            raise Truncated("want %d at %d of %d" % (n, self.pos, len(self.buf)))
        out = self.buf[self.pos : self.pos + n]
        self.pos += n
        return out

    def uint(self, n: int) -> int:
        return int.from_bytes(self.take(n), "big")

    def sint(self, n: int) -> int:
        return int.from_bytes(self.take(n), "big", signed=True)


# (token, kind, extra) per leading byte range.  kind drives the reader:
#   v  scalar, value already known
#   u/i  unsigned/signed integer of `extra` bytes
#   s/b  string/binary whose length is `extra` bytes wide
#   S/B  string/binary of fixed length `extra`
#   a/m  array/map whose count is `extra` bytes wide
#   A/M  array/map of fixed count `extra`
#   f  float of `extra` bytes
#   x  ext of fixed size `extra`; X ext with `extra`-wide length
FORMATS = {
    0xC0: ("nil", "v", None),
    0xC2: ("false", "v", False),
    0xC3: ("true", "v", True),
    0xC4: ("bin8", "b", 1),
    0xC5: ("bin16", "b", 2),
    0xC6: ("bin32", "b", 4),
    0xC7: ("ext8", "X", 1),
    0xC8: ("ext16", "X", 2),
    0xC9: ("ext32", "X", 4),
    0xCA: ("float32", "f", 4),
    0xCB: ("float64", "f", 8),
    0xCC: ("uint8", "u", 1),
    0xCD: ("uint16", "u", 2),
    0xCE: ("uint32", "u", 4),
    0xCF: ("uint64", "u", 8),
    0xD0: ("int8", "i", 1),
    0xD1: ("int16", "i", 2),
    0xD2: ("int32", "i", 4),
    0xD3: ("int64", "i", 8),
    0xD4: ("fixext1", "x", 1),
    0xD5: ("fixext2", "x", 2),
    0xD6: ("fixext4", "x", 4),
    0xD7: ("fixext8", "x", 8),
    0xD8: ("fixext16", "x", 16),
    0xD9: ("str8", "s", 1),
    0xDA: ("str16", "s", 2),
    0xDB: ("str32", "s", 4),
    0xDC: ("array16", "a", 2),
    0xDD: ("array32", "a", 4),
    0xDE: ("map16", "m", 2),
    0xDF: ("map32", "m", 4),
}


def head(r: Reader):
    """Read one format byte and resolve it to (token, kind, extra)."""
    c = r.take(1)[0]
    if c <= 0x7F:
        return ("fixint", "v", c)
    if c >= 0xE0:
        return ("negfixint", "v", c - 0x100)
    if 0x80 <= c <= 0x8F:
        return ("fixmap", "M", c & 0x0F)
    if 0x90 <= c <= 0x9F:
        return ("fixarray", "A", c & 0x0F)
    if 0xA0 <= c <= 0xBF:
        return ("fixstr", "S", c & 0x1F)
    if c in FORMATS:
        return FORMATS[c]
    return ("reserved%#x" % c, "v", None)


def show_bytes(b: bytes) -> str:
    """Printable, stable, and never wrapped -- one element is one line."""
    return repr(b.decode("latin1"))


def tokenise(r: Reader, out: list, depth: int, mask=None) -> object:
    """Emit one element (recursively) and return its decoded value.

    `mask` names a replacement for this element's value; the caller uses
    it for the fields that cannot be reproduced between runs."""
    token, kind, extra = head(r)
    pad = "  " * depth

    def emit(text, value=None):
        out.append("%s%s %s" % (pad, token if mask is None else "masked", text))
        return value

    if kind == "v":
        if mask is not None:
            return emit(mask, extra)
        return emit(repr(extra), extra)
    if kind in "ui":
        v = r.uint(extra) if kind == "u" else r.sint(extra)
        if mask is not None:
            return emit(mask, v)
        return emit(repr(v), v)
    if kind == "f":
        import struct as _s

        v = _s.unpack(">f" if extra == 4 else ">d", r.take(extra))[0]
        return emit(repr(v), v)
    if kind in "sb":
        raw = r.take(r.uint(extra))
        return emit(show_bytes(raw), raw.decode("latin1"))
    if kind in "SB":
        raw = r.take(extra)
        return emit(show_bytes(raw), raw.decode("latin1"))
    if kind in "xX":
        n = extra if kind == "x" else r.uint(extra)
        t = r.sint(1)
        return emit("type=%d %s" % (t, show_bytes(r.take(n))))
    if kind in "aA":
        n = r.uint(extra) if kind == "a" else extra
        emit("[%d]" % n)
        for _ in range(n):
            tokenise(r, out, depth + 1)
        return None
    if kind in "mM":
        n = r.uint(extra) if kind == "m" else extra
        emit("{%d}" % n)
        for _ in range(n):
            key = tokenise(r, out, depth + 1)
            # The header entry's pid is the one map value in the format
            # that changes every run.  Keys are strings, so the decoded
            # key is compared as bytes-through-latin1 repr.
            tokenise(r, out, depth + 1, mask="<PID>" if key == "pid" else None)
        return None
    return emit("?")


def tokenise_file(path: Path, out: list) -> None:
    b = path.read_bytes()
    r = Reader(b)
    out.append("%s %d bytes" % (path.name, len(b)))
    n = 0
    while r.pos < len(b):
        at = r.pos
        try:
            head_out: list = []
            typ = tokenise(r, head_out, 0)
            ts_out: list = []
            tokenise(r, ts_out, 0, mask="<TS>")
            len_out: list = []
            length = tokenise(r, len_out, 0)
            out.append(
                "entry %d @%d type=%s(%s) %s len=%s"
                % (
                    n,
                    at,
                    typ,
                    ENTRY_NAMES.get(typ, "Unknown"),
                    ts_out[0].strip(),
                    len_out[0].strip(),
                )
            )
            payload = r.take(length)
        except Truncated as e:
            out.append("entry %d @%d TRUNCATED (%s)" % (n, at, e))
            return
        inner = Reader(payload)
        try:
            tokenise(inner, out, 1)
        except Truncated as e:
            out.append("  payload TRUNCATED (%s)" % e)
        if inner.pos != len(payload):
            out.append("  payload TRAILING %d bytes" % (len(payload) - inner.pos))
        n += 1
    out.append("%s: %d entries" % (path.name, n))


# --- packing side -----------------------------------------------------------


def pack(v, as_key: bool = False) -> bytes:
    """Encode minimally, matching shada.c's conventions.

    shada.c packs map *keys* as msgpack strings and every other string --
    register contents, file names, variable names and values -- as
    msgpack *bin*.  The reader's `unpack_string` is what it is; feeding
    it the wrong family is a different test than the one intended, so
    the packer follows the writer."""
    if v is None:
        return b"\xc0"
    if v is True:
        return b"\xc3"
    if v is False:
        return b"\xc2"
    if isinstance(v, int):
        if 0 <= v <= 0x7F:
            return bytes([v])
        if -32 <= v < 0:
            return bytes([v + 0x100])
        if 0 <= v <= 0xFF:
            return b"\xcc" + v.to_bytes(1, "big")
        if 0 <= v <= 0xFFFF:
            return b"\xcd" + v.to_bytes(2, "big")
        if 0 <= v <= 0xFFFFFFFF:
            return b"\xce" + v.to_bytes(4, "big")
        if v >= 0:
            return b"\xcf" + v.to_bytes(8, "big")
        return b"\xd3" + v.to_bytes(8, "big", signed=True)
    if isinstance(v, str):
        raw = v.encode("utf-8")
        if as_key:
            if len(raw) <= 31:
                return bytes([0xA0 | len(raw)]) + raw
            return b"\xd9" + bytes([len(raw)]) + raw
        if len(raw) <= 0xFF:
            return b"\xc4" + bytes([len(raw)]) + raw
        return b"\xc5" + len(raw).to_bytes(2, "big") + raw
    if isinstance(v, list):
        body = b"".join(pack(x) for x in v)
        if len(v) <= 15:
            return bytes([0x90 | len(v)]) + body
        return b"\xdc" + len(v).to_bytes(2, "big") + body
    if isinstance(v, dict):
        body = b"".join(pack(k, True) + pack(x) for k, x in v.items())
        if len(v) <= 15:
            return bytes([0x80 | len(v)]) + body
        return b"\xde" + len(v).to_bytes(2, "big") + body
    raise TypeError(type(v))


def do_pack(spec_path: str, out_path: str) -> int:
    spec = json.loads(Path(spec_path).read_text())
    chunks = []
    for typ, ts, value in spec:
        payload = pack(value)
        chunks += [pack(typ), pack(ts), pack(len(payload)), payload]
    Path(out_path).write_bytes(b"".join(chunks))
    return 0


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "--pack":
        return do_pack(sys.argv[2], sys.argv[3])
    for arg in sys.argv[1:]:
        out: list = []
        tokenise_file(Path(arg), out)
        print("\n".join(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
