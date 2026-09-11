#!/usr/bin/env python3
"""Fail if a value type's drop shim grew a cleanup path.

`TypVal` and `Object` release themselves: both walk their tree iteratively
and take each owning payload out of its slot by hand, so every payload they
hold sits in a `ManuallyDrop` and the compiler appends nothing after
`Drop::drop` returns.

Give one of them a field the compiler *does* drop and `drop_in_place` grows
a landing pad -- the fields have to be released if `Drop::drop` unwinds --
and a shim with one is neither a tail call nor small enough to inline. That
cost 0.5 % on four of the five benches once (see
`~/agents/context/.../p30-11-drop-policy.md`); it is invisible in the source
and invisible to every test, so this reads the shim instead.

    just drop-glue                      # over target/release/nvim
    scripts/drop-glue.py path/to/nvim   # over a binary you built

The signal is a call to `_Unwind_Resume`, which is what a cleanup path ends
with, plus `__gxx_personality`/`rust_eh_personality` in the same body. A
shim that is a bare jump table and a tail call has neither.
"""

import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
DEFAULT_BINARY = REPO / "target/release/nvim"

# The shims that must stay clean. Each entry is (label, regex over the
# demangled symbol name).
WATCHED = [
    ("TypVal", re.compile(r"drop_in_place<.*\bTypVal>$")),
    ("Object", re.compile(r"drop_in_place<.*\bObject>$")),
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
        if len(parts) != 4:
            continue
        addr, size, _kind, name = parts
        try:
            yield int(addr, 16), int(size, 16), name
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

    found = {label: [] for label, _ in WATCHED}
    for addr, size, name in symbols(binary):
        for label, pattern in WATCHED:
            if pattern.search(name):
                found[label].append((addr, size, name))

    failed = False
    for label, _ in WATCHED:
        shims = found[label]
        if not shims:
            # A missing shim is a *failure*: either the type stopped having a
            # destructor (in which case this guard needs rewriting) or the
            # symbol was inlined away everywhere and there is nothing to
            # check, which the release build has never done.
            print(f"drop-glue: no drop_in_place shim for {label}", file=sys.stderr)
            failed = True
            continue
        for addr, size, name in shims:
            body = disassemble(binary, addr, size)
            if UNWIND.search(body):
                print(
                    f"drop-glue: {name} has a cleanup path "
                    f"(it calls _Unwind_Resume).\n"
                    f"  A payload the compiler drops after Drop::drop gives the shim\n"
                    f"  a landing pad, which stops it being a tail call and stops it\n"
                    f"  inlining. Hold the payload in a ManuallyDrop and release it\n"
                    f"  from the type's own Drop.",
                    file=sys.stderr,
                )
                failed = True
            else:
                print(f"drop-glue: {label} shim clean ({size} bytes)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
