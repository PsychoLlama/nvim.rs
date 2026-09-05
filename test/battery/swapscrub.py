#!/usr/bin/env python3
"""Scrub and describe a memline swap file (.swp) so two runs compare.

Three fields in block zero are functions of *when* and *where* the run
happened rather than of what the editor did: `b0_mtime` (the edited
file's mtime), `b0_ino` (its inode) and `b0_pid`.  `b0_uname`/`b0_hname`
are machine facts.  Everything else in the file -- the block tree, the
line index, the text -- is a pure function of the buffer contents and
the sequence of edits, so a byte-for-byte differential works once
exactly those fields are masked.

    ./swapscrub.py <file.swp> [...]      # rewrites in place

Writes a structural description of each file to stdout: block zero's
remaining fields decoded, then one line per block found walking the
pages.  That description is the readable half of the oracle -- a change
in the swap tree shape shows up there instead of only as a hash
mismatch -- and the scrub itself is reported per field, so a file this
script cannot parse is announced (`unknown`) rather than silently
passed through.

Layout (see memline.c; the Rust structs carry the same #[repr(C)]):

  ZeroBlock     b0_id[2] b0_version[10] b0_page_size[4] b0_mtime[4]
                b0_ino[4] b0_pid[4] b0_uname[40] b0_hname[40]
                b0_fname[900] b0_magic_long(8) b0_magic_int(4)
                b0_magic_short(2) b0_magic_char(1)  -> 1024 bytes
  DataBlock     db_id(u16)@0 db_free@4 db_txt_start@8 db_txt_end@12
                db_line_count(i64)@16 db_index[]@24
  PointerBlock  pb_id@0 pb_count@2 pb_count_max@4 pb_pointer[]@8
                PointerEntry = pe_bnum(i64) pe_line_count(i32)
                pe_old_lnum(i32) pe_page_count(i32) pad -> 24 bytes
"""

import struct
import sys
from pathlib import Path

B0_ID = b"b0"
DATA_ID = b"ad"  # ('d' << 8) + 'a', little-endian on the wire
PTR_ID = b"tp"  # ('p' << 8) + 't'

B0_FNAME_SIZE_ORG = 900
B0_DIRTY = 0x55

# Offsets into block zero.
O_VERSION, O_PAGE_SIZE, O_MTIME, O_INO, O_PID = 2, 12, 16, 20, 24
O_UNAME, O_HNAME, O_FNAME = 28, 68, 108
O_MAGIC_LONG = 108 + B0_FNAME_SIZE_ORG

DATA_HEADER = 24
PTR_HEADER = 8
PTR_ENTRY = 24

# What gets zeroed, and how many bytes.  b0_uname/b0_hname are stable on
# one machine but not across machines; masking them keeps a golden tree
# portable, and their lengths are still reported so a change in how they
# are filled in is visible.
SCRUB = [
    ("b0_mtime", O_MTIME, 4),
    ("b0_ino", O_INO, 4),
    ("b0_pid", O_PID, 4),
    ("b0_uname", O_UNAME, 40),
    ("b0_hname", O_HNAME, 40),
]


def cstr(b: bytes) -> str:
    """Decode up to the first NUL the way the C code reads these fields."""
    end = b.find(b"\0")
    if end >= 0:
        b = b[:end]
    return b.decode("latin1")


def char_to_long(b: bytes) -> int:
    """memline's portable 4-byte encoding: least significant byte first."""
    return int.from_bytes(b[:4], "little")


def describe_block0(b: bytes, out: list) -> int:
    """Decode block zero and return the page size it declares."""
    page_size = char_to_long(b[O_PAGE_SIZE : O_PAGE_SIZE + 4])
    fname = b[O_FNAME : O_FNAME + B0_FNAME_SIZE_ORG]
    flags = fname[B0_FNAME_SIZE_ORG - 2]
    dirty = fname[B0_FNAME_SIZE_ORG - 1]
    magic_long, magic_int, magic_short, magic_char = struct.unpack_from(
        "<qih", b, O_MAGIC_LONG
    ) + (b[O_MAGIC_LONG + 14],)
    # The 'fileencoding' is tucked in at the end of b0_fname behind a NUL
    # when B0_HAS_FENC is set; the name itself is the leading string.
    fenc = ""
    if flags & 8:
        fenc = cstr(fname[len(cstr(fname)) + 1 : B0_FNAME_SIZE_ORG - 2])
    out.append(
        "  b0 version=%r page_size=%d ff=%d same_dir=%d has_fenc=%d "
        "dirty=%s fname=%r fenc=%r"
        % (
            cstr(b[O_VERSION : O_VERSION + 10]),
            page_size,
            flags & 3,
            (flags >> 2) & 1,
            (flags >> 3) & 1,
            "yes" if dirty == B0_DIRTY else ("no" if dirty == 0 else hex(dirty)),
            cstr(fname),
            fenc,
        )
    )
    out.append(
        "  b0 magic long=%#x int=%#x short=%#x char=%#x ok=%s"
        % (
            magic_long & 0xFFFFFFFFFFFFFFFF,
            magic_int & 0xFFFFFFFF,
            magic_short & 0xFFFF,
            magic_char,
            magic_long == 0x30313233
            and magic_int == 0x20212223
            and magic_short == 0x1213  # B0_MAGIC_SHORT truncated to int16_t
            and magic_char == 0x55,
        )
    )
    return page_size


def describe_blocks(b: bytes, page_size: int, out: list) -> None:
    """Walk pages from block 1 on, reporting each block's header.

    Data blocks may span several pages; their `db_txt_end` gives the byte
    length, which is what advances the walk.  Anything else is reported
    by its first bytes so an unexpected page is a visible difference and
    not a resync failure."""
    if page_size <= 0 or len(b) % page_size:
        out.append(
            "  pages: page_size=%d file=%d bytes (not a multiple)" % (page_size, len(b))
        )
        return
    off = page_size
    while off + DATA_HEADER <= len(b):
        page = off // page_size
        ident = b[off : off + 2]
        if ident == DATA_ID:
            free, txt_start, txt_end = struct.unpack_from("<III", b, off + 4)
            line_count = struct.unpack_from("<q", b, off + 16)[0]
            span = max(txt_end, page_size)
            idx = (
                struct.unpack_from("<%dI" % max(line_count, 0), b, off + DATA_HEADER)
                if 0 <= line_count <= 4096
                else ()
            )
            marked = sum(1 for i in idx if i & 0x80000000)
            out.append(
                "  page %d data lines=%d free=%d txt_start=%d txt_end=%d "
                "pages=%d marked=%d"
                % (
                    page,
                    line_count,
                    free,
                    txt_start,
                    txt_end,
                    -(-span // page_size),
                    marked,
                )
            )
            off += -(-span // page_size) * page_size
        elif ident == PTR_ID:
            count, count_max = struct.unpack_from("<HH", b, off + 2)
            out.append("  page %d ptr count=%d count_max=%d" % (page, count, count_max))
            for i in range(min(count, 512)):
                at = off + PTR_HEADER + i * PTR_ENTRY
                if at + PTR_ENTRY > len(b):
                    out.append("    entry %d truncated" % i)
                    break
                bnum, lines, old_lnum, pages = struct.unpack_from("<qiii", b, at)
                out.append(
                    "    entry %d bnum=%d lines=%d old_lnum=%d pages=%d"
                    % (i, bnum, lines, old_lnum, pages)
                )
            off += page_size
        elif not any(b[off : off + page_size]):
            out.append("  page %d zero" % page)
            off += page_size
        else:
            out.append("  page %d other id=%r" % (page, ident))
            off += page_size


def main() -> int:
    for arg in sys.argv[1:]:
        p = Path(arg)
        b = bytearray(p.read_bytes())
        if not b.startswith(B0_ID) or len(b) < 1024:
            print("%s unknown (%d bytes)" % (p.name, len(b)))
            continue
        out = ["%s %d bytes" % (p.name, len(b))]
        scrubbed = []
        for name, off, size in SCRUB:
            raw = bytes(b[off : off + size])
            if size == 4:
                shown = "set" if char_to_long(raw) else "zero"
            else:
                shown = "len=%d" % len(cstr(raw))
            scrubbed.append("%s(%s)" % (name, shown))
            b[off : off + size] = b"\0" * size
        page_size = describe_block0(bytes(b), out)
        out.append("  scrubbed " + " ".join(scrubbed))
        describe_blocks(bytes(b), page_size, out)
        p.write_bytes(bytes(b))
        print("\n".join(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
