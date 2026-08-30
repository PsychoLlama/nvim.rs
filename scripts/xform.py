#!/usr/bin/env python3
"""A library for one-off tree-wide rewrites: find the calls, edit the bytes.

Not a tool -- there is nothing to run. It is the part of a mass-edit script
that is the same every time and that every previous sweep got wrong at least
once, kept here so the next one starts from the fixed version. Import it from
a throwaway script in a scratchpad; the script is the sweep and is thrown
away, this is not.

    import sys; sys.path.insert(0, "scripts")
    import xform

    for path in xform.rust_files():
        src = path.read_bytes()
        edits = []
        for start, open_paren, close, args in xform.call_spans(src, b"strlen"):
            lo, hi, _ = xform.outer_span(src, start, close)
            edits.append((lo, hi, b"cstr::bytes_at(" + xform.arg(src, args[0]) + b").len()"))
        if edits:
            path.write_bytes(xform.edit(src, edits))

What each piece exists to stop, all of them mistakes a sweep has actually
shipped:

`masked`   -- blanks comments, string literals and char literals byte for
              byte, so a rewriter keyed on `strlen(` does not find it in
              prose. Two dozen doc comments were rewritten before this
              existed. The char-literal arm is not decoration: `'"'` occurs
              in this tree, and without it that quote opens a string that
              swallows the rest of the file and every later call site goes
              unseen.
`call_spans` -- the call's own extent and its argument spans, matched
              against the masked text but indexing the real bytes. It refuses
              a `fn` header (six local `fn strlen` shims were rewritten into
              things that did not parse) and drops the empty argument a
              trailing comma leaves behind, which an arity check would
              otherwise skip.
`outer_span` -- widens a span over an `unsafe { ... }` that wraps *only* that
              call, so the replacement can drop the block. Answers whether it
              did.
`cmp_after` -- the `== 0` that so often follows, for turning a C comparison
              into a Rust one.
`edit`      -- applies the spans right to left, which is the only order that
              does not invalidate the offsets behind it.
`cstr_literal` / `byte_literal` -- a `c"..."` read into its bytes, and bytes
              written back out as `b"..."`.
`add_use`   -- one import, next to the file's existing ones. It deliberately
              cannot *remove* one: a cleanup regex like `use [\\w:]*;` matches
              every simple `use` in the file, which is how a sweep once ate
              forty of them. Let rustc name the unused imports and delete the
              spans it reports.

Everything works on `bytes`. The tree has non-ASCII in its comments, and a
pass that decoded to `str` and indexed by character corrupted 200 files
silently before anyone noticed.

Two rules no library can enforce, so they are written here instead:

- **Never run an unused-import fixer over a tree that does not parse.** rustc
  cannot see a use inside a body it failed to parse, calls the import unused,
  and the fixer deletes it.
- **Re-read every signature a span-driven fixer touches.** "expected i32,
  found Result" fires on functions that merely *contain* a converted call.
"""

import pathlib
import re
import subprocess

REPO = pathlib.Path(__file__).resolve().parent.parent
ROOT = REPO / "crates/nvim/src"


def rust_files(under=None):
    """Every tracked `.rs` file, so a stray copy under `target/` is invisible."""
    listed = subprocess.run(
        ["git", "ls-files", "*.rs"],
        cwd=REPO,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    paths = [REPO / f for f in listed]
    if under is None:
        return paths
    under = pathlib.Path(under).resolve()
    return [p for p in paths if under in p.parents or p == under]


def masked(src: bytes) -> bytes:
    """`src` with comments and string literals blanked, byte for byte.

    A rewriter keyed on `strlen(` finds it in prose too: the strncmp and
    strlen passes each rewrote two dozen doc comments before this existed.
    """
    out = bytearray(src)
    i, n = 0, len(src)
    while i < n:
        c = src[i : i + 1]
        two = src[i : i + 2]
        if two == b"//":
            j = src.find(b"\n", i)
            j = n if j < 0 else j
            out[i:j] = b" " * (j - i)
            i = j
        elif two == b"/*":
            j = src.find(b"*/", i + 2)
            j = n if j < 0 else j + 2
            for k in range(i, j):
                if out[k] != 0x0A:
                    out[k] = 0x20
            i = j
        elif c == b"'":
            # a char literal, or a lifetime. `'"'` is why this arm exists:
            # without it the quote opens a string that swallows the rest of
            # the file, and every call site after it goes unseen.
            k = i + 2 if src[i + 1 : i + 2] == b"\\" else i + 1
            while src[k : k + 1] not in (b"'", b"", b"\n") and k < i + 12:
                k += 1
            if src[k : k + 1] == b"'":
                for q in range(i, k + 1):
                    out[q] = 0x20
                i = k + 1
            else:
                i += 1
        elif c == b'"':
            j = i + 1
            while j < n and src[j : j + 1] != b'"':
                j += 2 if src[j : j + 1] == b"\\" else 1
            j = min(j + 1, n)
            for k in range(i, j):
                if out[k] != 0x0A:
                    out[k] = 0x20
            i = j
        else:
            i += 1
    return bytes(out)


def call_spans(src: bytes, name: bytes):
    """(start, open, close, [(argstart, argend)]) for each `name(...)`.

    Matched against `masked(src)` so that prose naming a C function costs
    nothing, and offsets still index the real bytes.
    """
    pat = re.compile(rb"(?<![\w.])" + re.escape(name) + rb"\s*\(")
    hay = masked(src)
    out = []
    for m in pat.finditer(hay):
        i, j, n = m.end() - 1, m.end() - 1, len(hay)
        depth, args, argstart = 0, [], m.end()
        while j < n:
            c = hay[j : j + 1]
            if c == b"'":
                k = j + 2 if hay[j + 1 : j + 2] == b"\\" else j + 1
                if hay[k : k + 1] == b"'":
                    j = k
            elif c in b"([{":
                depth += 1
            elif c in b")]}":
                depth -= 1
                if depth == 0:
                    args.append((argstart, j))
                    # a trailing comma leaves an empty last argument
                    if len(args) > 1 and not hay[args[-1][0] : args[-1][1]].strip():
                        args.pop()
                    out.append((m.start(), i, j, args))
                    break
            elif c == b"," and depth == 1:
                args.append((argstart, j))
                argstart = j + 1
            j += 1
    return out


UNSAFE_OPEN = re.compile(rb"unsafe\s*\{\s*$")


def outer_span(src: bytes, start: int, close: int):
    """Widen a call span over an `unsafe { ... }` that wraps only it.

    Answers (lo, hi, wrapped).
    """
    m = UNSAFE_OPEN.search(src, 0, start)
    if m and m.end() == start:
        k = close + 1
        while src[k : k + 1] in b" \n\t":
            k += 1
        if src[k : k + 1] == b"}":
            return m.start(), k + 1, True
    return start, close + 1, False


CMP = re.compile(rb"\s*([=!]=|[<>]=?)\s*0(?![\w.])")


def cmp_after(src: bytes, hi: int):
    """The `== 0` (etc.) that follows a span, as (op, end) or None."""
    m = CMP.match(src, hi)
    return (m.group(1), m.end()) if m else None


def arg(src, span):
    return b" ".join(src[span[0] : span[1]].split())


def edit(src: bytes, edits):
    """Apply [(lo, hi, replacement)] to `src`, right to left."""
    for lo, hi, rep in sorted(edits, reverse=True):
        src = src[:lo] + rep + src[hi:]
    return src


ESCAPES = {b"\\n": b"\n", b"\\t": b"\t", b"\\r": b"\r", b'\\"': b'"', b"\\\\": b"\\"}


def cstr_literal(a: bytes):
    """The bytes of a `c"..."` literal spelled as a pointer, or None."""
    m = re.fullmatch(rb'c"((?:[^"\\]|\\.)*)"(?:\s*\.\s*as_ptr\(\))?', a)
    if not m:
        return None
    body, out, i = m.group(1), b"", 0
    while i < len(body):
        if body[i : i + 1] == b"\\":
            e = ESCAPES.get(body[i : i + 2])
            if e is None:
                return None
            out += e
            i += 2
        else:
            if body[i] > 0x7F:
                return None
            out += body[i : i + 1]
            i += 1
    return out


def byte_literal(raw: bytes) -> bytes:
    """`raw` as a Rust `b"..."` literal."""
    out = b""
    for ch in raw:
        c = bytes([ch])
        if c == b"\\":
            out += b"\\\\"
        elif c == b'"':
            out += b'\\"'
        elif c == b"\n":
            out += b"\\n"
        elif c == b"\t":
            out += b"\\t"
        elif c == b"\r":
            out += b"\\r"
        elif 0x20 <= ch < 0x7F:
            out += c
        else:
            out += b"\\x%02x" % ch
    return b'b"' + out + b'"'


def add_use(src: bytes, item: bytes) -> bytes:
    """Insert `use <item>;` next to the file's other top-level `use`s."""
    line = b"use " + item + b";\n"
    if re.search(rb"(?m)^use " + re.escape(item) + rb"\s*;", src):
        return src
    m = re.search(rb"(?m)^use ", src)
    if not m:
        raise SystemExit("no `use` to anchor next to")
    return src[: m.start()] + line + src[m.start() :]
