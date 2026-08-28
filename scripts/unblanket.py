#!/usr/bin/env python3
"""Delete the blanket `unsafe {` / `}` c2rust wrapped each body in.

Phase 23's S6-S9 dissolve those blankets: `metrics/ratchet.json`'s
`unsafe_lines` counts *lines of code inside an `unsafe` region*, so one
blanket costs the whole function body whether it holds one unsafe operation
or fifty. This script does the mechanical half -- remove the block, dedent
its body by four -- and then `cargo check` lists the residue, which the
author wraps tightly, one operation at a time, with a SAFETY note.

    scripts/unblanket.py crates/nvim/src/indent_c/toplevel.rs
    scripts/unblanket.py --lines=42,118 crates/nvim/src/register/yank.rs

Only a line that is exactly `<indent>unsafe {` whose matching close is
exactly `<indent>}` is touched, so a tight `unsafe { expr }` already on one
line is left alone. `--lines=` restricts it to blocks opening on those
(1-based) lines. Structure is read off `ratchet.mask`ed text, so braces
inside strings and comments cannot confuse the matching.

**Build replacement text from the file this leaves behind**, not from the
pre-run file: everything moved left by four columns.

Files here carry `#![deny(unsafe_op_in_unsafe_fn)]`, which is what makes the
exercise pay -- without it an `unsafe fn`'s whole body scores as unchecked
and removing the inner block changes nothing. Land the deny first.
"""

from __future__ import annotations

import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import ratchet  # noqa: E402

OPEN = re.compile(r"^(\s*)unsafe \{$")


def whole_body(lines: list[str], open_at: int, close_at: int) -> bool:
    """Whether the block at `open_at` is a function's *entire* body.

    A blanket is the transpiler's, and removing it is the exercise. A tight
    region is an author's, and removing it undoes work: a family an earlier
    batch already converted is full of `unsafe {` / `}` pairs that match
    `OPEN` exactly and mean the opposite thing. Batch 4 paid for that by hand
    in every family batches 1 and 2 had reached.

    The transpiler's blanket is the only statement in its function, so it
    opens on the line after the signature's `{` and closes on the line before
    the function's own `}`.
    """
    before = lines[open_at - 1].rstrip() if open_at else ""
    after = lines[close_at + 1] if close_at + 1 < len(lines) else ""
    outer = lines[open_at][: len(lines[open_at]) - len(lines[open_at].lstrip())]
    return before.endswith("{") and after.rstrip() == outer[:-4] + "}"


def unblanket(
    text: str, only: set[int] | None = None, warn: bool = True
) -> tuple[str, int]:
    """The text with blanket blocks removed, and how many were removed."""
    lines = text.split("\n")
    masked = ratchet.mask(text).split("\n")
    out: list[str] = []
    i, removed, n = 0, 0, len(lines)
    while i < n:
        match = OPEN.match(lines[i])
        if match and (only is None or (i + 1) in only):
            indent = match.group(1)
            depth, close = 0, None
            for j in range(i, n):
                depth += masked[j].count("{") - masked[j].count("}")
                if depth == 0 and j > i:
                    close = j
                    break
            if close is not None and lines[close] == indent + "}":
                if warn and not whole_body(lines, i, close):
                    print(
                        f"  line {i + 1}: NOT a whole function body -- this "
                        "looks like a region an author wrote, not a blanket "
                        "the transpiler did. Removing it anyway; check the "
                        "diff.",
                        file=sys.stderr,
                    )
                body = lines[i + 1 : close]
                out.extend(
                    line[4:] if line.startswith(indent + "    ") else line
                    for line in body
                )
                removed += 1
                i = close + 1
                continue
        out.append(lines[i])
        i += 1
    return "\n".join(out), removed


def main() -> int:
    args = sys.argv[1:]
    only = None
    if args and args[0].startswith("--lines="):
        only = {int(x) for x in args[0].split("=", 1)[1].split(",")}
        args = args[1:]
    if not args:
        print(__doc__)
        return 2
    for name in args:
        path = pathlib.Path(name)
        text, count = unblanket(path.read_text(), only)
        path.write_text(text)
        print(f"{path}: {count} blanket block(s) removed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
