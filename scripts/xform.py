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

`lexed`    -- the one scanner for comments and literals; `masked` and `arg`
              both read it, because keeping two of them is how `arg` came to
              re-space eighteen message literals a sweep had no business
              touching.
`arg`      -- an argument's text folded onto one line, *except* inside a
              string or char literal, whose spacing is the program's output
              and not the sweep's to normalise.
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


def lexed(src: bytes):
    """`(kind, start, end)` for every comment and literal in `src`.

    `kind` is `b"//"`, `b"/*"`, `b"'"` or `b'"'`. One scanner, because two of
    them is how [`arg`] came to normalise the whitespace *inside* a string
    literal while [`masked`] knew perfectly well where it started.
    """
    out, i, n = [], 0, len(src)
    while i < n:
        c = src[i : i + 1]
        two = src[i : i + 2]
        if two == b"//":
            j = src.find(b"\n", i)
            j = n if j < 0 else j
            out.append((b"//", i, j))
            i = j
        elif two == b"/*":
            j = src.find(b"*/", i + 2)
            j = n if j < 0 else j + 2
            out.append((b"/*", i, j))
            i = j
        elif c == b"'":
            # a char literal, or a lifetime. `'"'` is why this arm exists:
            # without it the quote opens a string that swallows the rest of
            # the file, and every call site after it goes unseen. The
            # escaped form starts the search *past* the escaped byte:
            # `b'\\''` is a backslash, a quote and then the terminator, and
            # reading the middle one as the terminator left the last quote
            # to open a string that swallowed the rest of `indent_c/decl.rs`.
            k = i + 3 if src[i + 1 : i + 2] == b"\\" else i + 1
            while src[k : k + 1] not in (b"'", b"", b"\n") and k < i + 12:
                k += 1
            if src[k : k + 1] == b"'":
                out.append((b"'", i, k + 1))
                i = k + 1
            else:
                i += 1
        elif c == b'"':
            j = i + 1
            while j < n and src[j : j + 1] != b'"':
                j += 2 if src[j : j + 1] == b"\\" else 1
            j = min(j + 1, n)
            out.append((b'"', i, j))
            i = j
        else:
            i += 1
    return out


def masked(src: bytes) -> bytes:
    """`src` with comments and string literals blanked, byte for byte.

    A rewriter keyed on `strlen(` finds it in prose too: the strncmp and
    strlen passes each rewrote two dozen doc comments before this existed.
    """
    out = bytearray(src)
    for kind, lo, hi in lexed(src):
        for k in range(lo, hi):
            # A char literal is blanked whole, quotes and all; the others
            # keep their newlines so line numbers survive.
            if kind == b"'" or out[k] != 0x0A:
                out[k] = 0x20
    return bytes(out)


# `fn name(`, with the visibility and the `unsafe`/`const`/`async` that may
# stand between: the definition of the very function a sweep is rewriting the
# *calls* of. Anchored at the end so it only matches immediately before the
# name.
FN_HEADER = re.compile(rb"\bfn\s+$")


def call_spans(src: bytes, name: bytes):
    """(start, open, close, [(argstart, argend)]) for each `name(...)`.

    Matched against `masked(src)` so that prose naming a C function costs
    nothing, and offsets still index the real bytes.

    A `fn name(` header is refused: it is not a call, and rewriting it is how
    a `ret_string` retirement turned its own definition into something that
    did not parse, six local `fn strlen` shims before that.
    """
    pat = re.compile(rb"(?<![\w.])" + re.escape(name) + rb"\s*\(")
    hay = masked(src)
    out = []
    for m in pat.finditer(hay):
        if FN_HEADER.search(hay, 0, m.start()):
            continue
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
    """The argument's text, folded onto one line.

    Whitespace *inside* a string or char literal is left alone. The message
    sweep folded it along with everything else and quietly re-spaced eighteen
    literals -- column headers, indent runs, `"  (Already listed)"` -- which
    nothing but the legacy suite noticed, and only twelve tests later.

    An argument carrying a `//` comment raises instead: folded onto one line
    the comment swallows everything after it, which is how the `ret_string`
    retirement ate the tail of two files. Hoist the comment out of the call
    and run the sweep again.
    """
    lo, hi = span
    if any(kind == b"//" for kind, _, _ in lexed(src[lo:hi])):
        raise ValueError(
            "argument spans a `//` comment; folding it would comment out the "
            "rest of the line: " + src[lo:hi].decode(errors="replace")
        )
    out, at = [], lo
    for kind, a, b in lexed(src[lo:hi]):
        if kind in (b"//", b"/*"):
            continue
        a, b = lo + a, lo + b
        out.append((b" ".join(src[at:a].split()), src[a:b]))
        at = b
    out.append((b" ".join(src[at:hi].split()), b""))
    joined = b""
    for code, lit in out:
        if joined and code and not joined.endswith(b" "):
            joined += b" "
        joined += code + lit
    return joined.strip()


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
