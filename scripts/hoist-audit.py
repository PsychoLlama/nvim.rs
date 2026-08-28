#!/usr/bin/env python3
"""Find calls that left a `||`/`&&` chain — the audit's one blind spot.

Phase 23's batches move `unsafe` around in bulk, and after the blanket is
gone the biggest remaining lever is **argument hoisting**: rustfmt's
`fn_call_width` of 60 makes a seven-argument call eight to seventeen lines
inside its region, and the same call with its arguments bound to `let`s
*outside* it is one. Every worker of batch 4 reported it as the largest win
after the blanket itself.

It is only sound out of an **unconditional** call:

    // upstream
    if a() || copy_option_part(&mut p, buf, len, sep) || b() { .. }
    // hoisted -- WRONG
    let part = unsafe { copy_option_part(&mut p, buf, len, sep) };
    if a() || part || b() { .. }

`copy_option_part` advances `p`, so run unconditionally it writes and steps
where upstream short-circuits past it.

**Neither existing audit sees this.** `scripts/ratchet.py` is about `unsafe`,
not evaluation order. The token-multiset audit (p23-8 §5) counts `||`, `&&`,
`!` and callee names, and this edit changes none of them. `unsafe-diff.py`
does show it, as one hunk among the thousands a batch produces.

So this narrows the field instead of judging it: a callee whose count in
boolean context **fell** while a `let` binding it **appeared** is a
candidate. It is deliberately a triage aid, not a gate — over batch 4's 450
changed files it answered 41 candidates, of which 4 were calls with side
effects worth reading and 0 were bugs. The other 37 are pure reads (`get`,
`as_ptr`, `is_null`), closures whose body holds the call, and calls that
were already the unconditional left operand.

    scripts/hoist-audit.py <base-rev> [paths...]
"""

from __future__ import annotations

import collections
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import ratchet  # noqa: E402

CALL = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(")
LET_CALL = re.compile(
    r"\blet\s+(?:mut\s+)?[^=;]+=\s*[^;]*?\b([A-Za-z_][A-Za-z0-9_]*)\s*\("
)
# Names that are never the interesting callee: control flow, and the two
# constructors that wrap almost every expression in this tree.
IGNORE = frozenset(
    {
        "if",
        "while",
        "for",
        "match",
        "return",
        "fn",
        "unsafe",
        "let",
        "as",
        "impl",
        "Some",
        "Ok",
        "Err",
    }
)


def boolean_calls(text: str) -> collections.Counter[str]:
    """Callee names within one line of a `||` or `&&`."""
    lines = ratchet.mask(text).split("\n")
    found: collections.Counter[str] = collections.Counter()
    for i, line in enumerate(lines):
        if "||" not in line and "&&" not in line:
            continue
        window = "\n".join(lines[max(0, i - 1) : i + 2])
        for match in CALL.finditer(window):
            if match.group(1) not in IGNORE:
                found[match.group(1)] += 1
    return found


def let_calls(text: str) -> collections.Counter[str]:
    """Callee names appearing in the initialiser of a `let`."""
    found: collections.Counter[str] = collections.Counter()
    for match in LET_CALL.finditer(ratchet.mask(text)):
        if match.group(1) not in IGNORE:
            found[match.group(1)] += 1
    return found


def main() -> int:
    if len(sys.argv) < 2:
        sys.exit("usage: hoist-audit.py <base-rev> [paths...]")
    base = sys.argv[1]
    paths = sys.argv[2:] or ["crates/nvim/src"]
    changed = subprocess.run(
        ["git", "diff", "--name-only", base, "--", *paths],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()

    flagged = 0
    for name in changed:
        if not name.endswith(".rs"):
            continue
        file = pathlib.Path(name)
        if not file.exists():
            continue
        old = subprocess.run(
            ["git", "show", f"{base}:{name}"], capture_output=True, text=True
        )
        if old.returncode:
            continue  # added by this range
        new_text = file.read_text()
        was_bool, now_bool = boolean_calls(old.stdout), boolean_calls(new_text)
        was_let, now_let = let_calls(old.stdout), let_calls(new_text)
        moved = [
            callee
            for callee in was_bool
            if now_bool[callee] < was_bool[callee] and now_let[callee] > was_let[callee]
        ]
        if moved:
            flagged += 1
            detail = ", ".join(
                f"{c} (in a chain {was_bool[c]}->{now_bool[c]}, "
                f"in a `let` {was_let[c]}->{now_let[c]})"
                for c in sorted(moved)[:5]
            )
            print(f"!! {name}: {detail}")
    print(
        f"\n{flagged} file(s) to read: does the callee have a side effect, "
        "and was it already unconditional?"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
