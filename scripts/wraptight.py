#!/usr/bin/env python3
"""Wrap the residue `scripts/unblanket.py` leaves in tight `unsafe {}` regions.

Phase 23's S6-S9 dissolve the blanket `unsafe {}` c2rust wrapped each
function body in, and then wrap what is left one operation at a time.
`unblanket.py` does the first half; this does the mechanical part of the
second. It drives `cargo check --message-format=json` in a loop and rewrites
the exact byte spans rustc reports, so the region it produces is the
compiler's own idea of the operation's extent -- which is as tight as one
can be without judging the code.

    scripts/wraptight.py crates/nvim/src/ex_cmds/

Five passes run to a fixed point, in this order, because each one's output
is the next one's input:

  wrap      E0133 -> `unsafe { <span> }`.
  rechain   E0507 -> push the `}` past the field chain hanging off it.
            `(*eap).forceit`'s E0133 span is the *deref*, so a naive wrap
            gives `unsafe { *eap }.forceit`, which moves the struct out of
            the raw pointer.
  reparen   E0609 with a `*mut`/`*const` base -> put back the parentheses a
            deref needs (`unsafe { *eap.f }` parses as `*(eap.f)`).
  fixtail   "expected expression, found `=`/`as`/`+=`" -> grow a block that
            ended too early. A block is not a place expression and cannot
            lead a binary operand, so `unsafe { (*p).f } = v` is a syntax
            error; the region has to cover the whole statement.
  unnest    "unnecessary `unsafe` block" -> delete blocks that the wrapping
            made redundant, and the parentheses rustc calls unnecessary.

What it does NOT do is the part that pays best -- reading a byte once per
loop iteration, lifting a callee to a safe fn, taking one `&mut *ptr` at the
head of a run of assignments (p23-6 §2's shapes 1-4). Those are judgement
and stay the author's. Nor does it write SAFETY notes: after a run, every
region it made still needs one, and `just lint` will not tell you that.

Anything the five passes cannot express is left as a compile error for a
human -- an assignment through a `let ... else`, a borrow of the span, a
tail expression leading a `||` chain. Expect a handful per family.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys

CHECK = ["cargo", "check", "-p", "neovim", "--message-format=json"]


# ---------------------------------------------------------------------------
# Talking to rustc


def diagnostics(paths: list[str]) -> list[dict]:
    """Every compiler message, in order."""
    out = subprocess.run(CHECK, capture_output=True, text=True).stdout
    found = []
    for line in out.split("\n"):
        if not line.startswith("{"):
            continue
        msg = json.loads(line)
        if msg.get("reason") == "compiler-message":
            found.append(msg["message"])
    return found


def code_of(diag: dict) -> str:
    return (diag.get("code") or {}).get("code") or ""


def primary_spans(diags: list[dict], paths: list[str], want) -> dict[str, list[dict]]:
    """`{file: [span, ...]}` for the diagnostics `want` accepts."""
    found: dict[str, dict[int, dict]] = {}
    for diag in diags:
        if not want(diag):
            continue
        for span in diag["spans"]:
            if not span["is_primary"]:
                continue
            name = span["file_name"]
            if any(name.startswith(p) for p in paths):
                found.setdefault(name, {})[span["byte_start"]] = span
    return {f: [v[k] for k in sorted(v)] for f, v in found.items()}


# ---------------------------------------------------------------------------
# The five passes


def outermost(spans: list[tuple[int, int]]) -> list[tuple[int, int]]:
    """Keep only the outermost of any nested or overlapping group."""
    kept: list[tuple[int, int]] = []
    for start, end in sorted(spans, key=lambda s: (s[0], -s[1])):
        if kept and start < kept[-1][1]:
            kept[-1] = (kept[-1][0], max(kept[-1][1], end))
            continue
        kept.append((start, end))
    return kept


def statement_end(raw: bytes, at: int) -> int | None:
    """The `;` or closing bracket that ends the expression starting at `at`."""
    depth = 0
    for i in range(at, len(raw)):
        c = raw[i : i + 1]
        if c in b"([{":
            depth += 1
        elif c in b")]}":
            if depth == 0:
                return i
            depth -= 1
        elif c in b";," and depth == 0:
            return i
    return None


def wrap(raw: bytes, spans: list[dict]) -> tuple[bytes, int]:
    """E0133: `<span>` -> `unsafe { <span> }`."""
    done = 0
    pairs = [(s["byte_start"], s["byte_end"]) for s in spans]
    for start, end in reversed(outermost(pairs)):
        body = raw[start:end]
        if body.startswith(b"unsafe"):
            continue
        raw = raw[:start] + b"unsafe { " + body + b" }" + raw[end:]
        done += 1
    return raw, done


def closing_brace(raw: bytes, open_at: int) -> int | None:
    """The index of the `}` matching the `{` at `open_at`."""
    depth = 0
    for j in range(open_at, len(raw)):
        c = raw[j : j + 1]
        if c == b"{":
            depth += 1
        elif c == b"}":
            depth -= 1
            if depth == 0:
                return j
    return None


def enclosing_unsafe(raw: bytes, at: int) -> int | None:
    """The index of the `}` closing the `unsafe {}` around `at`."""
    i = raw.rfind(b"unsafe {", 0, at)
    if i < 0:
        return None
    depth = 0
    for j in range(raw.index(b"{", i), len(raw)):
        c = raw[j : j + 1]
        if c == b"{":
            depth += 1
        elif c == b"}":
            depth -= 1
            if depth == 0:
                return j if j > at else None
    return None


def chain_end(raw: bytes, at: int) -> int:
    """The end of the `.field` / `[i]` chain starting at `at`.

    **A method call ends the chain, before its `.`.** A projection is part of
    the place the deref names, so the region has to cover it; a method call
    takes the place *by value* and moving the brace past it changes what the
    `*` binds to -- `unsafe { *var }.clamp(0, 1)` rewritten as
    `unsafe { *var .clamp(0, 1) }` is `*(var.clamp(0, 1))`, which is a
    different program and sometimes still compiles. Four of those shipped in
    one S8 run over the whole crate before this stopped.
    """
    i = at
    while i < len(raw) and raw[i : i + 1] in (b".", b"["):
        if raw[i : i + 1] == b".":
            j = i + 1
            while j < len(raw) and (raw[j : j + 1].isalnum() or raw[j : j + 1] == b"_"):
                j += 1
            # `.name(` is a method call, and `.0`/`.1` on a tuple is not.
            if raw[j : j + 1] == b"(" or raw[j : j + 1] == b":":
                return i
            i = j
            continue
        depth = 0
        while i < len(raw):
            c = raw[i : i + 1]
            if c == b"[":
                depth += 1
            elif c == b"]":
                depth -= 1
                if depth == 0:
                    i += 1
                    break
            i += 1
    return i


DEREF_WRAP = re.compile(rb"unsafe \{ (\(*\*[^{}]*?) \}(?=[.\[])")


def ends_in_call(inner: bytes) -> bool:
    """Whether `inner` ends in a *call*, so it answers with a value.

    `(*p)` also ends in `)` and is very much a place; the two are told apart
    by what sits before the matching `(` -- an identifier means a call,
    anything else means a grouping paren.
    """
    inner = inner.rstrip()
    if not inner.endswith(b")"):
        return False
    depth = 0
    for i in range(len(inner) - 1, -1, -1):
        c = inner[i : i + 1]
        if c == b")":
            depth += 1
        elif c == b"(":
            depth -= 1
            if depth == 0:
                before = inner[i - 1 : i]
                return bool(before) and (before.isalnum() or before in (b"_", b"]"))
    return False


def open_of(raw: bytes, close: int) -> int:
    """The first byte of the block whose `}` is at `close`."""
    depth = 0
    for i in range(close, -1, -1):
        if raw[i : i + 1] == b"}":
            depth += 1
        elif raw[i : i + 1] == b"{":
            depth -= 1
            if depth == 0:
                return i + 2 if raw[i + 1 : i + 2] == b" " else i + 1
    return close


def rechain(raw: bytes, spans: list[dict] | None = None) -> tuple[bytes, int]:
    """Move the `}` past the projection hanging off a wrapped deref.

    **This pass is structural, not diagnostic-driven, and it must stay that
    way.** `(*p).f`'s E0133 span is the deref alone, so a naive wrap leaves
    `unsafe { *p }.f`. When `*p`'s type is not `Copy` that is E0507 and the
    compiler catches it -- but when it *is* `Copy` the code compiles, reads
    fine, and **silently writes to a discarded temporary**:

        unsafe { *p }.a = 42;   // compiles, warns nothing, does nothing

    That shape cost this slice two behaviour regressions (`nv_gv_cmd`'s
    `b_visual` swap and `unadjust_for_sel_inner`, both `Copy` structs) which
    only the functional suite caught. So every wrapped deref followed by a
    `.field` or `[i]` gets the brace pushed past the chain, whether or not
    rustc complained -- but only past a *projection*: see `chain_end`, and
    the `as` guard below.
    """
    edits = set()
    for match in DEREF_WRAP.finditer(raw):
        if ends_in_call(match.group(1)):
            # The wrapped expression ends in a *call*, so it is already a
            # value and the `.field` after it is a read of a temporary --
            # which is what the source said. Nothing to repair.
            continue
        if b" as " in match.group(1):
            # `unsafe { *p as u8 }.is_ascii_uppercase()` is already right: the
            # cast made a value, and the method on it is not a place. Pushing
            # the brace past it is a syntax error at best.
            continue
        close = match.end() - 1
        edits.add((close, chain_end(raw, close + 1), match.start(1)))
    for span in spans or []:
        close = enclosing_unsafe(raw, span["byte_start"])
        if close is not None and raw[close + 1 : close + 2] in (b".", b"["):
            edits.add((close, chain_end(raw, close + 1), open_of(raw, close)))
    for close, end, start in sorted(edits, reverse=True):
        if end <= close + 1:
            continue  # nothing to move the brace past
        inner = raw[start:close].rstrip()
        # `unsafe { *p }.f` must become `unsafe { (*p).f }`, never
        # `unsafe { *p .f }` -- the latter is `*(p.f)`, a different program
        # that compiles whenever `p.f` is itself a pointer. c2rust writes
        # `(*p).f` so the parentheses are usually already there, which is
        # exactly why this hole stayed open.
        if not balanced_group(inner):
            raw = (
                raw[:start]
                + b"("
                + inner
                + b")"
                + raw[close + 1 : end]
                + b" }"
                + raw[end:]
            )
        else:
            raw = raw[:close] + raw[close + 1 : end] + b" }" + raw[end:]
    return raw, sum(1 for close, end, _ in edits if end > close + 1)


def balanced_group(text: bytes) -> bool:
    """Whether `text` is one parenthesised group, so `text.f` binds to it."""
    if not (text.startswith(b"(") and text.endswith(b")")):
        return False
    depth = 0
    for i in range(len(text)):
        if text[i : i + 1] == b"(":
            depth += 1
        elif text[i : i + 1] == b")":
            depth -= 1
            if depth == 0:
                return i == len(text) - 1
    return False


def reparen(raw: bytes, spans: list[dict]) -> tuple[bytes, int]:
    """E0609 on a pointer base: `unsafe { *p.f }` -> `unsafe { (*p).f }`."""
    edits = set()
    for span in spans:
        at = span["byte_start"]
        i = raw.rfind(b"unsafe {", 0, at)
        if i < 0:
            continue
        star = raw.index(b"{", i) + 1
        while raw[star : star + 1].isspace():
            star += 1
        if raw[star : star + 1] != b"*" or raw[at - 1 : at] != b".":
            continue
        edits.add((star, at - 1))
    for star, dot in sorted(edits, reverse=True):
        raw = raw[:star] + b"(" + raw[star:dot] + b")" + raw[dot:]
    return raw, len(edits)


def line_offsets(raw: bytes) -> list[int]:
    offs, at = [0], 0
    for line in raw.split(b"\n"):
        at += len(line) + 1
        offs.append(at)
    return offs


def fixtail(raw: bytes, spans: list[dict]) -> tuple[bytes, int]:
    """A block that stopped at an operator: grow it to the statement's end."""
    offs = line_offsets(raw)
    edits = set()
    for span in spans:
        at = offs[span["line_start"] - 1] + span["column_start"] - 1
        close = raw.rfind(b"}", 0, at)
        if close < 0 or raw[close + 1 : at].strip():
            continue
        end = statement_end(raw, at)
        if end is not None:
            edits.add((close, end))
    for close, end in sorted(edits, reverse=True):
        raw = raw[:close] + raw[close + 1 : end].rstrip() + b" }" + raw[end:]
    return raw, len(edits)


def unnest(raw: bytes, spans: list[dict]) -> tuple[bytes, int]:
    """Delete an `unsafe {}` rustc now calls unnecessary, body kept.

    `unused_unsafe`'s primary span covers **only the `unsafe` keyword**, not
    the block, so the brace has to be matched forward from there.
    """
    offs = line_offsets(raw)
    edits = []
    for span in spans:
        at = offs[span["line_start"] - 1] + span["column_start"] - 1
        if raw[at : at + 6] != b"unsafe":
            continue
        open_at = raw.find(b"{", at)
        close = closing_brace(raw, open_at) if open_at >= 0 else None
        if close is None:
            continue
        edits.append((at, open_at, close))
    for start, open_at, close in sorted(set(edits), reverse=True):
        raw = raw[:start] + raw[open_at + 1 : close].strip() + raw[close + 1 :]
    return raw, len(set(edits))


def deparen(raw: bytes, spans: list[dict]) -> tuple[bytes, int]:
    """Drop parentheses rustc calls unnecessary around a block's value."""
    offs = line_offsets(raw)
    edits = set()
    for span in spans:
        at = offs[span["line_start"] - 1] + span["column_start"] - 1
        if raw[at : at + 1] != b"(":
            continue
        depth = 0
        for k in range(at, len(raw)):
            c = raw[k : k + 1]
            if c == b"(":
                depth += 1
            elif c == b")":
                depth -= 1
                if depth == 0:
                    edits.add((at, k))
                    break
    for open_at, close in sorted(edits, reverse=True):
        raw = raw[:open_at] + raw[open_at + 1 : close] + raw[close + 1 :]
    return raw, len(edits)


# ---------------------------------------------------------------------------

PARSE_TAIL = re.compile(r"expected expression, found `(=|as|[-+*/%|&<>!]?=|\|\||&&)")

PASSES = [
    ("wrap", wrap, lambda d: code_of(d) == "E0133"),
    # Runs on every round, diagnostics or not -- see `rechain`'s docstring.
    ("rechain", rechain, lambda d: code_of(d) == "E0507"),
    (
        "reparen",
        reparen,
        lambda d: (
            code_of(d) == "E0609"
            and ("on type `*mut" in d["message"] or "on type `*const" in d["message"])
        ),
    ),
    ("fixtail", fixtail, lambda d: bool(PARSE_TAIL.search(d["message"]))),
    ("unnest", unnest, lambda d: "unnecessary `unsafe` block" in d["message"]),
    (
        "deparen",
        deparen,
        lambda d: "unnecessary parentheses around block" in d["message"],
    ),
]


def all_files(paths: list[str]) -> list[str]:
    """Every `.rs` file under the paths given."""
    found: list[str] = []
    for name in paths:
        path = pathlib.Path(name)
        found.extend(
            str(p) for p in (sorted(path.rglob("*.rs")) if path.is_dir() else [path])
        )
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="+")
    parser.add_argument("--rounds", type=int, default=12)
    args = parser.parse_args()

    for round_no in range(1, args.rounds + 1):
        diags = diagnostics(args.paths)
        moved = 0
        for name, apply, want in PASSES:
            per_file = primary_spans(diags, args.paths, want)
            if name == "rechain":
                # Structural: every file in scope, not only the ones rustc
                # named. A `Copy` deref leaves no diagnostic behind.
                for path in sorted(all_files(args.paths)):
                    per_file.setdefault(path, [])
            if not per_file:
                continue
            count = 0
            for path, spans in per_file.items():
                p = pathlib.Path(path)
                raw, n = apply(p.read_bytes(), spans)
                p.write_bytes(raw)
                count += n
            print(f"round {round_no}: {name} {count} site(s)")
            moved += count
            if count:
                # One pass per round: a later pass would read stale offsets.
                # A pass that applied nothing must not starve the ones after
                # it -- `rechain` holding spans it cannot express would stop
                # the run with parse errors still in the tree.
                break
        if moved == 0:
            print(f"round {round_no}: nothing left to do")
            return 0
    print("wraptight: did not reach a fixed point; look at the errors by hand")
    return 1


if __name__ == "__main__":
    sys.exit(main())
