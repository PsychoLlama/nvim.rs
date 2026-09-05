#!/usr/bin/env python3
"""Scrub and decode an undo file (.un~) so two runs compare byte for byte.

Exactly two fields in the file are functions of *when* the run happened
rather than of what the editor did: the file header's `b_u_time_cur` and
each undo header's `uh_time`, both written by `time_to_bytes` as eight
big-endian bytes.  Everything else is a pure function of the buffer
contents and the sequence of edits -- including the 32-byte hash, which
is a SHA-256 of the buffer text (`u_compute_hash`) and is therefore
stable and deliberately *not* masked.

    ./undoscrub.py <file.un~> [...]      # rewrites in place

Writes a field-by-field decode of each file to stdout: the file header,
then every undo header with its cursor, flags, 26 named marks, Visual
info, save number, entries and extmark objects.  That decode is the
readable half of the oracle -- a change in the *tree* shows up there
instead of only as a hash mismatch -- and the scrub is reported per
field, so a file this script cannot parse is announced (`UNPARSED`)
rather than silently passed through.

Layout (undo/file.rs; every integer is big-endian via `undo_write_bytes`,
so this is a stream format, not a struct image -- with one exception,
noted at `extmark` below):

  file header   UF_START_MAGIC[9] version(2) hash(32) ml_line_count(4)
                line_ptr_len(4) line_ptr[len] line_lnum(4) line_colnr(4)
                oldhead_seq(4) newhead_seq(4) curhead_seq(4) numhead(4)
                seq_last(4) seq_cur(4) time_cur(8)
                {len(1) what(1) payload[len]}* 0(1)
  uh record     UF_HEADER_MAGIC(2) next_seq(4) prev_seq(4)
                alt_next_seq(4) alt_prev_seq(4) uh_seq(4) cursor(12)
                cursor_vcol(4) flags(2) namedm[26](12 each)
                visual(32) time(8) {len(1) what(1) payload[len]}* 0(1)
                {UF_ENTRY_MAGIC(2) uep}* UF_ENTRY_END_MAGIC(2)
                {UF_ENTRY_MAGIC(2) extmark}* UF_ENTRY_END_MAGIC(2)
  uep           ue_top(4) ue_bot(4) ue_lcount(4) ue_size(4)
                {len(4) text[len]}*ue_size
  pos           lnum(4) col(4) coladd(4)
  visualinfo    start(pos) end(pos) vi_mode(4) vi_curswant(4)
  extmark       type(4, big-endian) then a *raw memory image* of
                ExtmarkSplice or ExtmarkMove -- 6 x int32 then 3 x
                int64, native (little) endian, 48 bytes, no padding.
                This is the one struct image in the format and the one
                place the file is not portable across endianness.
  end           UF_HEADER_END_MAGIC(2)
"""

import struct
import sys
from pathlib import Path

UF_START_MAGIC = b"Vim\x9fUnDo\xe5"
UF_HEADER_MAGIC = 0x5FD0
UF_HEADER_END_MAGIC = 0xE7AA
UF_ENTRY_MAGIC = 0xF518
UF_ENTRY_END_MAGIC = 0x3581
UF_VERSION = 3

UF_LAST_SAVE_NR = 1
UHP_SAVE_NR = 1

UNDO_HASH_SIZE = 32
NMARKS = 26

UH_FLAGS = [(1, "UH_CHANGED"), (2, "UH_EMPTYBUF"), (4, "UH_RELOAD")]

K_EXTMARK_SPLICE = 0
K_EXTMARK_MOVE = 1
# Both are six int32 then three int64: 24 + 24 = 48 bytes with no
# padding, since the int64s land on an already-8-aligned offset.
EXTMARK_FMT = "<6i3q"
EXTMARK_SIZE = struct.calcsize(EXTMARK_FMT)
assert EXTMARK_SIZE == 48
SPLICE_FIELDS = [
    "start_row",
    "start_col",
    "old_row",
    "old_col",
    "new_row",
    "new_col",
    "start_byte",
    "old_byte",
    "new_byte",
]
MOVE_FIELDS = [
    "start_row",
    "start_col",
    "extent_row",
    "extent_col",
    "new_row",
    "new_col",
    "start_byte",
    "extent_byte",
    "new_byte",
]

# The masked fields, as (name, why).  Both are `time_to_bytes` output.
MASKED = "b_u_time_cur uh_time"


class Truncated(Exception):
    """The stream ended inside a field."""


class Reader:
    """A cursor over the file, remembering where every field started."""

    def __init__(self, data: bytes):
        self.data = data
        self.pos = 0
        # Offsets of the eight-byte time fields, filled in as they are
        # read, so the caller can mask exactly those bytes.
        self.time_offsets: list[tuple[str, int]] = []

    def take(self, n: int) -> bytes:
        if self.pos + n > len(self.data):
            raise Truncated(
                f"want {n} bytes at {self.pos}, have {len(self.data) - self.pos}"
            )
        out = self.data[self.pos : self.pos + n]
        self.pos += n
        return out

    def be(self, n: int) -> int:
        """`undo_write_bytes` reads back: `n` bytes, most significant first."""
        return int.from_bytes(self.take(n), "big")

    def signed(self, n: int) -> int:
        """The same, reinterpreted as two's complement.

        Fields holding a `linenr_T`/`colnr_T`/`int` are written by
        widening to `uintmax_t` and keeping the low `n` bytes, so a
        negative value arrives as its two's-complement tail.
        """
        v = self.be(n)
        return v - (1 << (8 * n)) if v >= (1 << (8 * n - 1)) else v

    def time(self, name: str) -> int:
        off = self.pos
        v = self.be(8)
        self.time_offsets.append((name, off))
        return v

    def pos_t(self) -> str:
        lnum, col, coladd = self.signed(4), self.signed(4), self.signed(4)
        return f"({lnum},{col},{coladd})"


def esc(b: bytes) -> str:
    """One printable line, so a byte difference shows in the diff."""
    return "".join(chr(c) if 32 <= c <= 126 and c != 0x5C else f"\\x{c:02x}" for c in b)


def flags_str(flags: int) -> str:
    names = [n for bit, n in UH_FLAGS if flags & bit]
    rest = flags & ~sum(bit for bit, _ in UH_FLAGS)
    if rest:
        names.append(f"0x{rest:x}")
    return "|".join(names) if names else "0"


def read_optional(r: Reader, out: list, indent: str, known: dict) -> None:
    """The `{len what payload}* 0` trailer both header kinds carry.

    Unknown tags are skipped by length, exactly as the reader does, and
    reported by tag so a newly-emitted field is visible.
    """
    while True:
        length = r.be(1)
        if length == 0:
            out.append(f"{indent}optional-end")
            return
        what = r.be(1)
        name = known.get(what)
        if name is not None and length == 4:
            out.append(f"{indent}{name} = {r.signed(4)}")
        else:
            out.append(
                f"{indent}unknown-optional tag={what} len={length} {esc(r.take(length))}"
            )


def read_uep(r: Reader, out: list, idx: int) -> None:
    ue_top = r.signed(4)
    ue_bot = r.signed(4)
    ue_lcount = r.signed(4)
    ue_size = r.signed(4)
    out.append(
        f"    uep[{idx}] ue_top={ue_top} ue_bot={ue_bot} "
        f"ue_lcount={ue_lcount} ue_size={ue_size}"
    )
    if ue_size < 0:
        raise Truncated(f"negative ue_size {ue_size}")
    for i in range(ue_size):
        length = r.be(4)
        out.append(f"      line[{i}] len={length} {esc(r.take(length))}")


def read_extmark(r: Reader, out: list, idx: int) -> None:
    type_ = r.be(4)
    if type_ == K_EXTMARK_SPLICE:
        fields = SPLICE_FIELDS
        kind = "kExtmarkSplice"
    elif type_ == K_EXTMARK_MOVE:
        fields = MOVE_FIELDS
        kind = "kExtmarkMove"
    else:
        raise Truncated(f"unknown extmark type {type_}")
    vals = struct.unpack(EXTMARK_FMT, r.take(EXTMARK_SIZE))
    body = " ".join(f"{n}={v}" for n, v in zip(fields, vals))
    out.append(f"    extmark[{idx}] {kind} {body}")


def read_uhp(r: Reader, out: list, idx: int) -> None:
    out.append(f"  uh[{idx}] @{r.pos - 2}")
    nxt, prev = r.signed(4), r.signed(4)
    alt_next, alt_prev = r.signed(4), r.signed(4)
    uh_seq = r.signed(4)
    out.append(
        f"    seq={uh_seq} next={nxt} prev={prev} "
        f"alt_next={alt_next} alt_prev={alt_prev}"
    )
    cursor = r.pos_t()
    vcol = r.signed(4)
    flags = r.be(2)
    out.append(f"    cursor={cursor} cursor_vcol={vcol} flags={flags_str(flags)}")
    marks = [r.pos_t() for _ in range(NMARKS)]
    set_marks = [f"{chr(97 + i)}{m}" for i, m in enumerate(marks) if m != "(0,0,0)"]
    out.append(f"    namedm[{NMARKS}] set={len(set_marks)} " + " ".join(set_marks))
    vi_start, vi_end = r.pos_t(), r.pos_t()
    vi_mode, vi_curswant = r.signed(4), r.signed(4)
    out.append(
        f"    visual start={vi_start} end={vi_end} "
        f"mode={vi_mode}{f' ({chr(vi_mode)!r})' if 32 <= vi_mode <= 126 else ''} "
        f"curswant={vi_curswant}"
    )
    r.time("uh_time")
    out.append("    uh_time = <MASKED>")
    read_optional(r, out, "    ", {UHP_SAVE_NR: "uh_save_nr"})

    n = 0
    while True:
        magic = r.be(2)
        if magic != UF_ENTRY_MAGIC:
            break
        read_uep(r, out, n)
        n += 1
    if magic != UF_ENTRY_END_MAGIC:
        raise Truncated(f"entry end: expected {UF_ENTRY_END_MAGIC:#x}, got {magic:#x}")
    out.append(f"    entries = {n}")

    n = 0
    while True:
        magic = r.be(2)
        if magic != UF_ENTRY_MAGIC:
            break
        read_extmark(r, out, n)
        n += 1
    if magic != UF_ENTRY_END_MAGIC:
        raise Truncated(
            f"extmark end: expected {UF_ENTRY_END_MAGIC:#x}, got {magic:#x}"
        )
    out.append(f"    extmarks = {n}")


def decode(data: bytes, out: list) -> Reader:
    r = Reader(data)
    magic = r.take(len(UF_START_MAGIC))
    if magic != UF_START_MAGIC:
        raise Truncated(f"start magic {esc(magic)}")
    version = r.be(2)
    out.append(f"  magic = {esc(magic)}  version = {version}")
    if version != UF_VERSION:
        raise Truncated(f"version {version}")
    out.append(f"  hash = {r.take(UNDO_HASH_SIZE).hex()}")
    out.append(f"  ml_line_count = {r.signed(4)}")
    line_len = r.be(4)
    line = r.take(line_len)
    out.append(f"  b_u_line_ptr len={line_len} {esc(line)}")
    out.append(f"  b_u_line_lnum = {r.signed(4)}  b_u_line_colnr = {r.signed(4)}")
    oldhead, newhead, curhead = r.signed(4), r.signed(4), r.signed(4)
    out.append(f"  oldhead_seq={oldhead} newhead_seq={newhead} curhead_seq={curhead}")
    out.append(
        f"  b_u_numhead={r.signed(4)} b_u_seq_last={r.signed(4)} "
        f"b_u_seq_cur={r.signed(4)}"
    )
    r.time("b_u_time_cur")
    out.append("  b_u_time_cur = <MASKED>")
    read_optional(r, out, "  ", {UF_LAST_SAVE_NR: "b_u_save_nr_last"})

    n = 0
    while True:
        magic = r.be(2)
        if magic != UF_HEADER_MAGIC:
            break
        read_uhp(r, out, n)
        n += 1
    if magic != UF_HEADER_END_MAGIC:
        raise Truncated(
            f"header end: expected {UF_HEADER_END_MAGIC:#x}, got {magic:#x}"
        )
    out.append(f"  headers = {n}")
    if r.pos != len(data):
        out.append(f"  TRAILING {len(data) - r.pos} bytes {esc(data[r.pos :])}")
    return r


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__, file=sys.stderr)
        return 2
    rc = 0
    for name in argv:
        path = Path(name)
        data = path.read_bytes()
        out = [f"== {path.name} {len(data)} bytes (masked: {MASKED})"]
        try:
            r = decode(data, out)
        except Truncated as exc:
            out.append(f"  UNPARSED {exc}")
            print("\n".join(out))
            rc = 1
            continue
        # Mask in place, so the hash the wrapper takes is of the scrubbed
        # bytes and the copied-out file can be diffed by hand.
        masked = bytearray(data)
        for field, off in r.time_offsets:
            masked[off : off + 8] = b"\0" * 8
        if bytes(masked) != data:
            path.write_bytes(bytes(masked))
        out.append(f"  scrubbed {len(r.time_offsets)} time fields")
        print("\n".join(out))
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
