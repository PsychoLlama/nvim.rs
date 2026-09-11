#!/usr/bin/env python3
"""Fail if a value type's drop shim grew a cleanup path.

`TypVal` and `Object` release themselves: each walks its tree iteratively and
takes every owning payload out of its slot by hand, so the payloads sit in a
`ManuallyDrop` and the compiler appends nothing after `Drop::drop` returns.

Give one of them a field the compiler *does* drop and `core::ptr::drop_glue`
grows a landing pad -- the fields have to be released if `Drop::drop` unwinds
-- and a shim with one is neither a tail call nor small enough to inline at
the hundreds of sites that drop a value. That was worth ~0.5 % on four of the
five benches once (see the phase-30 notes on the drop policy); it is
invisible in the source and invisible to every suite, so this reads the shim.

    just drop-glue                      # over target/release/nvim
    scripts/drop-glue.py path/to/nvim   # over a binary you built

The signal is a call to `_Unwind_Resume`, which is how a cleanup path ends. A
shim the compiler inlined everywhere emits no symbol at all, which is the
*best* outcome and is reported as such -- but the type's `Drop` impl must
still be there, or the guard is watching a type that no longer has one.
"""

import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
DEFAULT_BINARY = REPO / "target/release/nvim"

# Each row is (label, the type's path as `nm -C` spells it).
WATCHED = [
    ("TypVal", "neovim::types::typval::TypVal"),
    ("Object", "neovim::types::api::Object"),
]

UNWIND = re.compile(rb"_Unwind_Resume")


def symbols(binary: pathlib.Path):
    """`(address, size, demangled name)` for every defined symbol."""
    out = subprocess.run(
        ["nm", "-C", "-S", "--defined-only", str(binary)],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    for line in out.splitlines():
        parts = line.split(maxsplit=3)
        if len(parts) == 4:
            addr, size, _kind, name = parts
            try:
                yield int(addr, 16), int(size, 16), name
            except ValueError:
                continue
        elif len(parts) == 3:
            # A symbol `nm` knows no size for; the disassembly below needs
            # one, so it is not a shim this can read.
            addr, _kind, name = parts
            try:
                yield int(addr, 16), 0, name
            except ValueError:
                continue


def disassemble(binary: pathlib.Path, addr: int, size: int) -> bytes:
    return subprocess.run(
        [
            "objdump",
            "-d",
            "--no-show-raw-insn",
            f"--start-address=0x{addr:x}",
            f"--stop-address=0x{addr + size:x}",
            str(binary),
        ],
        capture_output=True,
        check=True,
    ).stdout


def main() -> int:
    binary = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_BINARY
    if not binary.exists():
        print(f"drop-glue: {binary} does not exist; build it first", file=sys.stderr)
        return 2

    table = list(symbols(binary))
    failed = False
    for label, path in WATCHED:
        glue = f"core::ptr::drop_glue::<{path}>"
        impl = f"<{path} as core::ops::drop::Drop>::drop"
        shims = [row for row in table if row[2] == glue]
        has_impl = any(row[2] == impl for row in table)

        if not has_impl:
            # Either the type stopped having a destructor -- in which case
            # this guard is watching the wrong thing and needs rewriting --
            # or the name moved.
            print(f"drop-glue: no Drop impl for {label} ({impl})", file=sys.stderr)
            failed = True
            continue

        if not shims:
            print(f"drop-glue: {label}'s shim is inlined everywhere")
            continue

        for addr, size, name in shims:
            body = disassemble(binary, addr, size)
            if UNWIND.search(body):
                print(
                    f"drop-glue: {name} has a cleanup path (it calls _Unwind_Resume).\n"
                    "  A payload the compiler drops after Drop::drop gives the shim a\n"
                    "  landing pad, which stops it being a tail call and stops it\n"
                    "  inlining. Hold the payload in a ManuallyDrop and release it from\n"
                    "  the type's own Drop.",
                    file=sys.stderr,
                )
                failed = True
            else:
                print(f"drop-glue: {label}'s shim is clean ({size} bytes)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
