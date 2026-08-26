#!/usr/bin/env python3
"""Rewrite `(*curwin.get()).x` / `(*curbuf.get()).x` to the winlayer wrappers.

Phase 23's slices S6-S9 propagate `Win::current()`/`Buf::current()` over the
~3,000 inline dereferences of the two roots. The edit itself is mechanical
and this script does it; what it does *not* do is judge, so every run is
followed by `cargo check`, `just fmt` and a reading of the diff.

What it rewrites, per file:

  (*curwin.get()).field   ->  cur_win().field
  (*curbuf.get()).field   ->  cur_buf().field

and then, if the file gained a call and does not have them already, appends
the two file-local helpers the tree already uses in a dozen modules:

    /// The window the editor is working in.
    fn cur_win() -> Win {
        // SAFETY: `curwin` is set from startup to exit.
        unsafe { Win::current() }
    }

The helper, not `Win::current()` inline, is the point: `current()` is an
`unsafe fn`, so spelling it at every site would leave the call unchecked,
while one helper pays the promise once and every caller is ordinary safe
code. It is also what makes the *next* step possible -- dissolving the
blanket `unsafe {}` the transpiler wrapped each function body in, which is
where the unchecked-line count actually falls (see the notes at the bottom).

What it refuses to touch, and why:

  &raw mut (*curwin.get()).w_cursor     a borrow taken *through* the value
  &raw const (*curbuf.get()).b_chartab
  &mut (*curwin.get()).w_cursor.lnum

    `&raw mut cur_win().w_cursor` derives its provenance from the transient
    `&mut win_T` that `DerefMut` hands out, and the next read of `curwin`
    would invalidate it under Miri's aliasing model. Use the projection that
    already exists (`Win::cursor()` covers 42 of the 85 such sites) or leave
    the line alone.

    An ordinary `&mut (*curwin.get()).f` is refused for the same reason in a
    shorter window: `DerefMut` reborrows the *whole* `win_T`, so passing one
    field to a callee that reads any other part of `curwin` aliases. Convert
    it only after reading the callee, and say so in the SAFETY note.

  anything inside a string, a `//` comment or a `#[doc]`
  a file that does not compile before the run

Usage:

    scripts/root-deref.py crates/nvim/src/textobject/pair.rs ...
    scripts/root-deref.py --dry-run crates/nvim/src/normal/

Prints one line per file: what changed, and what it declined to change.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

ROOTS = {
    "curwin": ("cur_win", "Win", "window"),
    "curbuf": ("cur_buf", "Buf", "buffer"),
}

# `(*curwin.get()).` with nothing but whitespace variation allowed.
DEREF = re.compile(r"\(\s*\*\s*(curwin|curbuf)\s*\.\s*get\(\)\s*\)\s*\.")
# The same, preceded by a borrow of any kind: refused. `(?<![&\w])&(?!&)`
# keeps the second `&` of a `&&` chain from reading as a borrow.
ADDR_OF = re.compile(
    r"(?<![&\w])&(?!&)\s*(?:raw\s+(?:mut|const)\s+|mut\s+)?"
    r"\(\s*\*\s*(curwin|curbuf)\s*\.\s*get\(\)\s*\)\s*\."
)
# A `use crate::main::{...}` item list, so a root that is no longer named
# can be dropped from it.
MAIN_USE = re.compile(r"use crate::main::\{([^}]*)\};", re.S)

HELPER = """
/// The {noun} the editor is working in.
fn {fn}() -> {ty} {{
    // SAFETY: `{root}` is set from startup to exit.
    unsafe {{ {ty}::current() }}
}}
"""


def strip_noncode(line: str) -> str:
    """The line with `//` comments and string literals blanked out."""
    out, i, quote = [], 0, None
    while i < len(line):
        c = line[i]
        if quote is None and c == "/" and line[i : i + 2] == "//":
            break
        if quote is None and c in "\"'":
            quote, c = c, " "
        elif quote is not None and c == "\\":
            out.append("  ")
            i += 2
            continue
        elif quote is not None and c == quote:
            quote, c = None, " "
        elif quote is not None:
            c = " "
        out.append(c)
        i += 1
    return "".join(out)


def rewrite(text: str) -> tuple[str, dict[str, int], int]:
    """The rewritten text, how many sites each root gained, and how many were
    refused for taking an address."""
    used: dict[str, int] = {}
    refused = 0
    lines = text.split("\n")
    for n, line in enumerate(lines):
        code = strip_noncode(line)
        if not DEREF.search(code):
            continue
        refused += len(ADDR_OF.findall(code))
        if ADDR_OF.search(code):
            continue

        def one(match: re.Match[str]) -> str:
            root = match.group(1)
            used[root] = used.get(root, 0) + 1
            return f"{ROOTS[root][0]}()."

        # Rewrite only over the code span: a `//` comment keeps its wording.
        cut = len(code.rstrip()) if "//" not in line else line.index("//")
        head, tail = line[:cut], line[cut:]
        lines[n] = DEREF.sub(one, head) + tail
    return "\n".join(lines), used, refused


def drop_from_main_use(text: str, root: str) -> str:
    """Take `root` out of `use crate::main::{...}` when nothing names it.

    The search runs over code with comments and strings blanked out, so the
    helper's own SAFETY comment does not count as a use of the name.
    """
    rest = MAIN_USE.sub("", text)
    code = "\n".join(strip_noncode(line) for line in rest.split("\n"))
    if re.search(rf"\b{root}\b", code):
        return text

    def edit(match: re.Match[str]) -> str:
        items = [i.strip() for i in match.group(1).split(",") if i.strip()]
        kept = [i for i in items if i != root]
        if kept == items:
            return match.group(0)
        if not kept:
            return ""
        return "use crate::main::{" + ", ".join(kept) + "};"

    return MAIN_USE.sub(edit, text)


def process(path: pathlib.Path, dry_run: bool) -> bool:
    original = path.read_text()
    text, used, refused = rewrite(original)
    if not used:
        if refused:
            print(f"{path}: nothing rewritten, {refused} address-of site(s) left")
        return False

    wanted = []
    for root, count in sorted(used.items()):
        fn, ty, noun = ROOTS[root]
        if not re.search(rf"\bfn {fn}\(\)", text):
            text += HELPER.format(fn=fn, ty=ty, noun=noun, root=root)
        wanted.append(ty)
        text = drop_from_main_use(text, root)

    missing = [ty for ty in wanted if not re.search(rf"\b{ty}\b", original)]
    if missing:
        # In front of the first `use`, which is after the module doc and any
        # inner attributes. rustfmt sorts the block afterwards.
        item = "use crate::winlayer::{" + ", ".join(sorted(missing)) + "};"
        lines = text.split("\n")
        first = next((i for i, l in enumerate(lines) if l.startswith("use ")), None)
        if first is None:
            print(f"{path}: no `use` block to add {item} to; add it by hand")
        else:
            lines.insert(first, item)
            text = "\n".join(lines)

    counts = ", ".join(f"{root} x{n}" for root, n in sorted(used.items()))
    note = f", {refused} address-of site(s) left" if refused else ""
    print(f"{path}: {counts}{note}")
    if not dry_run:
        path.write_text(text)
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="+", type=pathlib.Path)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    files = []
    for path in args.paths:
        files.extend(sorted(path.rglob("*.rs")) if path.is_dir() else [path])
    touched = sum(process(f, args.dry_run) for f in files)
    verb = "would be rewritten" if args.dry_run else "rewritten"
    tail = "" if args.dry_run else " Now: cargo check, just fmt, read the diff."
    print(f"{touched} file(s) {verb}.{tail}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
