#!/usr/bin/env python3
"""Fail if a state record's cell is exported from the binary.

A state record is one `GlobalCell` holding a whole namespace of editor state
(`global_cell::state_record!`, and the option table apigen writes). Every
read of one is a load from that static, so where the static lives matters:
a static only this crate names is addressed directly, but one the compiler
thinks another crate may name is *exported* and read through the GOT -- an
extra load on every access. Nothing in the source says which one you get.
A static is exported when a body that names it is shippable cross-crate: a
`pub` generic or `#[inline]` item, a small `pub` leaf fn (rustc inlines
those across crates without being asked), or an `#[inline]` method of a
trait impl for a `pub` type. Each of those has happened once already; the
option table's cost 0.7 % of spellbench.

`nm` tells: an exported data symbol is `D`/`B`, a local one `d`/`b`. This
reads the release binary and fails on any record whose cell is upper-case,
and on any record it cannot find at all -- a renamed cell would otherwise
leave the guard watching nothing.

    just state-exports                      # over target/release/nvim
    scripts/state-exports.py path/to/nvim   # over a binary you built

The records are found in the source: every `state_record!` invocation names
its cell (`struct R in CELL as F;`), and the module path comes from the
file. `EXTRA` lists the ones a macro other than `state_record!` declares.
"""

import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
SRC = REPO / "crates/nvim/src"
DEFAULT_BINARY = REPO / "target/release/nvim"
CRATE = "neovim"

# Records declared some other way, as `(label, path as nm -C spells it)`.
EXTRA = [
    ("OPTIONS", f"{CRATE}::options::vars::OPTIONS"),
]

RECORD = re.compile(
    r"state_record!\s*\{.*?\bstruct\s+(\w+)\s+in\s+(\w+)\s+as\s+\w+\s*;", re.S
)


def module_path(file: pathlib.Path) -> str:
    parts = list(file.relative_to(SRC).with_suffix("").parts)
    if parts[-1] in ("mod", "lib"):
        parts.pop()
    return "::".join([CRATE, *parts])


def records():
    """`(label, path)` for every record the source declares."""
    found = list(EXTRA)
    for file in sorted(SRC.rglob("*.rs")):
        if file.name == "global_cell.rs":
            # The macro's own definition and its doc example.
            continue
        for match in RECORD.finditer(file.read_text()):
            record, cell = match.groups()
            found.append((record, f"{module_path(file)}::{cell}"))
    return found


def data_symbols(binary: pathlib.Path):
    """Demangled name -> `nm` kind, for every data symbol."""
    out = subprocess.run(
        ["nm", "-C", "--defined-only", str(binary)],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    kinds = {}
    for line in out.splitlines():
        parts = line.split(maxsplit=2)
        if len(parts) == 3 and parts[1] in "dDbB":
            kinds[parts[2]] = parts[1]
    return kinds


def main() -> int:
    binary = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_BINARY
    if not binary.exists():
        print(
            f"state-exports: {binary} does not exist; build it first", file=sys.stderr
        )
        return 2

    kinds = data_symbols(binary)
    failed = False
    for label, path in records():
        kind = kinds.get(path)
        if kind is None:
            print(f"state-exports: no data symbol {path} ({label})", file=sys.stderr)
            failed = True
        elif kind.isupper():
            print(
                f"state-exports: {path} ({label}) is exported ({kind}).\n"
                "  Some body that names it is shippable cross-crate: a `pub`\n"
                "  generic or `#[inline]` item, a small `pub` leaf fn, or an\n"
                "  `#[inline]` trait method on a `pub` type. Make it `pub(crate)`\n"
                "  or move the access out of line.",
                file=sys.stderr,
            )
            failed = True
        else:
            print(f"state-exports: {label} is local ({kind})")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
