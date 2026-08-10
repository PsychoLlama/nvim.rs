#!/usr/bin/env python3
"""Ratchet the migration metrics: counts may hold or shrink, never grow.

The migration's promise is monotonic progress — every change leaves the tree
no less safe than it found it. This script is the mechanism. It measures, per
Rust source file (crates/*/src/**/*.rs plus the crate-root .rs files;
integration tests under crates/*/tests are not migration surface and stay
unmeasured, as they were when they lived at the repo root):

  unsafe_lines  lines of *code* the compiler is not checking: the lines
              spanned by an `unsafe {}` block, by an `unsafe extern` block,
              or by the body of an `unsafe fn` in a file that has not
              adopted #![deny(unsafe_op_in_unsafe_fn)] (there the body is
              implicitly unsafe throughout, which is exactly what the metric
              is about). Spans are unioned, so a nested block never counts
              twice, and blank/comment-only lines are excluded, so a SAFETY
              comment is free and a rewrite is never penalised for
              explaining itself. `unsafe impl`/`unsafe trait` cost their
              header line — an unchecked promise with nowhere else to book
              it. An `unsafe fn` *type* (a function pointer) costs nothing:
              the obligation is paid where it is called.

              This metric used to count `unsafe {}` *blocks*, and blocks and
              the goal diverge on exactly the change the migration wants
              most. Splitting a 700-line transpiled body into fifteen
              functions with narrow blocks books fifteen units where there
              was one: edit.rs went 89 -> 104 blocks across phase 15 while
              its clippy count went 33 -> 0. Lines-of-unchecked-code moves
              the right way for every shape: narrowing a block lowers it,
              deleting unsafe code lowers it, adopting the deny and wrapping
              a body is neutral, and adding unchecked code raises it. It is
              also the number that states the goal — phase by phase, how
              much of the editor the compiler still cannot vouch for.

  static_mut  occurrences of "static mut "
  no_mangle   occurrences of "#[unsafe(no_mangle)]"
  variadic    occurrences of ": ..." — C-variadic parameters, whose calls
              are format-string-unchecked. They retire as their callers
              migrate to the format_args!-based macros (semsg! and friends)
              or their modules are rewritten; vim_snprintf/vim_vsnprintf
              (vim's own user-visible format language) are expected to be
              the long-lived remainder.
  cell_ptr    raw escape-hatch accesses to global editor state:
              GlobalCell/SharedCell `.ptr()` and `.as_raw()`. The cells
              themselves are the safe replacement for c2rust's mutable
              statics, but `ptr()` hands back a bare `*mut T` and reinstates
              every obligation the cell exists to check — it was the
              landing zone for the mechanical conversion, not a destination.
              Call sites narrow to get/set/with/with_mut as their modules
              are rewritten, and cells whose state can simply be owned are
              deleted outright; both show up here. Counting accesses rather
              than cells is deliberate: converting a surviving `extern`
              global into a real cell is progress, and a metric over cell
              *declarations* would book it as regression. The needles are
              receiver-blind, as everything else here is; today no other
              type in the tree has a nullary `ptr()`/`as_raw()`, and one
              that did would only over-count, which is the safe direction.
  lines       line count. No file may exceed 1,000 lines; files already over
              the cap are grandfathered at their committed size and may
              shrink or hold, never grow. New files start at the cap.

plus two whole-tree metrics:

  internal_exports  the number of internal-only exports in the committed ABI
                    ledger (metrics/abi-ledger.jsonl — `just abi-ledger
                    --check` separately guarantees that file matches the
                    tree).

  files_without_forbid_unsafe  the number of source files not carrying
                    #![forbid(unsafe_code)]. The shrink-only trick inverted:
                    fully safe modules take the attribute — which makes
                    "safe module" a compiler-enforced status instead of a
                    grep result — and the count of files still lacking it
                    may only fall. New files are expected to be born safe.

  files_without_deny_unsafe_op  the number of source files carrying neither
                    #![forbid(unsafe_code)] nor
                    #![deny(unsafe_op_in_unsafe_fn)]. Same trick for edition
                    2024's honest-unsafe lint: the crate allows it (see
                    Cargo.toml — blanket body-wrapping would double the
                    textual unsafe count), each module denies it once its
                    unsafe fns use explicit unsafe blocks, and the count of
                    files doing neither may only fall. The crate root
                    (lib.rs) can't deny per-module — its inner attributes
                    are crate-level — so it holds the floor at 1 until the
                    allow itself retires.

A `warnings` metric used to sit alongside it; phase 5 drove the count to
zero and the dev shell (flake.nix) now sets `RUSTFLAGS="-D warnings"` for
every local and CI build instead, so the counter is retired.

Everything is measured over a *masked* copy of the source, in which comments,
string literals and character literals are blanked out (offsets and newlines
preserved) so that only code is scanned. That is what makes the counts mean
what they say: prose about `unsafe` costs nothing, a doc comment quoting
`#![deny(unsafe_op_in_unsafe_fn)]` does not switch on the deny, and a string
containing a brace cannot desynchronise the block scanner. Everything else is
plain substring matching, which still over-counts a little (a macro naming
`static mut ` in its expansion counts), but is deterministic, cheap enough for
a pre-commit hook, and kept canonical by rustfmt (enforced by fmt-check). The
point is monotonic pressure, not precision.

The baseline is committed at metrics/ratchet.json (one file per line, so diffs
review like the ledger's). A metric above its baseline is a violation; a
metric below it means progress that must be locked in by regenerating the
baseline and committing it alongside the change.

Regenerate through `just refresh`, not this script directly: the measurement
is only valid on a formatted tree with a current ledger, and refresh sequences
those. Calling ratchet.py first and formatting after bakes in line counts the
formatter is about to change.

Usage: ratchet.py [--check] [--allow-growth]
  --check         compare the tree against the committed baseline instead of
                  writing: exit 1 if any metric grew, or if the baseline is
                  stale (a metric shrank but metrics/ratchet.json wasn't
                  regenerated).
  --allow-growth  write a baseline even though a metric grew. The override
                  for justified cases — the growth shows up in the
                  metrics/ratchet.json diff; explain it in the commit message.
"""

import json
import re
import sys
from bisect import bisect_right
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "metrics" / "ratchet.json"
LEDGER = ROOT / "metrics" / "abi-ledger.jsonl"

LINE_CAP = 1000
# name -> needles counted in the masked source, summed.
COUNTED = {
    "static_mut": ("static mut ",),
    "no_mangle": ("#[unsafe(no_mangle)]",),
    "variadic": (": ...",),
    "cell_ptr": (".ptr()", ".as_raw()"),
}
FORBID = "#![forbid(unsafe_code)]"
DENY_UNSAFE_OP = "#![deny(unsafe_op_in_unsafe_fn)]"

IDENT = re.compile(r"[A-Za-z0-9_]")
IDENT_AT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
WHITESPACE = re.compile(r"\s*")
UNSAFE_WORD = re.compile(r"\bunsafe\b")
# Literal prefixes that make a following `#`/`"` open a raw string.
RAW_PREFIXES = {"r", "br", "cr", "rb", "rc"}


def mask(text):
    """The source with comments, strings and char literals blanked to spaces.

    Offsets and newlines are preserved, so the result can be scanned
    structurally (brace matching, keyword search) and mapped back to line
    numbers, while nothing inside a literal or a comment can be mistaken for
    code.
    """
    out = list(text)
    n = len(text)
    i = 0

    def blank(start, stop):
        for k in range(start, stop):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = text[i]
        if IDENT.match(c):
            # Consume whole identifiers so `r` inside one can't open a raw
            # string, and so `b'x'` reaches the char-literal branch below.
            j = i
            while j < n and IDENT.match(text[j]):
                j += 1
            if text[i:j] in RAW_PREFIXES and j < n and text[j] in '#"':
                k = j
                while k < n and text[k] == "#":
                    k += 1
                if k < n and text[k] == '"':
                    close = text.find('"' + "#" * (k - j), k + 1)
                    j = n if close < 0 else close + 1 + (k - j)
                    blank(i, j)
            i = j
        elif c == "/" and text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif c == "/" and text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            j = min(j + 1, n)
            blank(i, j)
            i = j
        elif c == "'":
            # A char literal, or a lifetime — `'a` looks like the start of
            # one until the closing quote fails to show up.
            if i + 1 < n and text[i + 1] == "\\":
                # The escape consumes exactly one character, so the earliest
                # the closing quote can sit is i+3 -- and starting the scan
                # there is what makes `'\\'` terminate. Treating the second
                # backslash as an escape instead (`j += 2`) walked straight
                # past the closing quote and blanked source as far as the
                # next quote in the file, taking its braces with it.
                j = i + 3
                while j < n and text[j] != "'":
                    j += 1
                j = min(j + 1, n)
            elif i + 2 < n and text[i + 2] == "'":
                j = i + 3
            else:
                i += 1
                continue
            blank(i, j)
            i = j
        else:
            i += 1
    return "".join(out)


def matching_brace(masked, open_at):
    """Offset of the `}` closing the `{` at open_at (end of text if unpaired)."""
    depth = 0
    for i in range(open_at, len(masked)):
        if masked[i] == "{":
            depth += 1
        elif masked[i] == "}":
            depth -= 1
            if not depth:
                return i
    return len(masked) - 1


def unsafe_lines(masked, deny):
    """Lines of code the compiler is not checking. See the module docs."""
    starts = [0, *(m.end() for m in re.finditer("\n", masked))]

    def lineno(offset):
        """0-based line holding `offset`."""
        return bisect_right(starts, offset) - 1

    # Lines whose masked content is blank hold no code: comments (SAFETY
    # notes included) and empty lines inside a block are free.
    code_lines = {i for i, line in enumerate(masked.splitlines()) if line.strip()}

    covered = set()
    for match in UNSAFE_WORD.finditer(masked):
        at = WHITESPACE.match(masked, match.end()).end()
        word = IDENT_AT.match(masked, at)
        keyword = word.group(0) if word else ""
        body_at = None
        if at < len(masked) and masked[at] == "{":
            body_at = at  # `unsafe { ... }`
        elif keyword == "extern":
            after = WHITESPACE.match(masked, word.end()).end()
            follows = IDENT_AT.match(masked, after)
            if after < len(masked) and masked[after] == "{":
                body_at = after  # `unsafe extern "C" { ... }`
            elif follows and follows.group(0) == "fn":
                word = follows  # `unsafe extern "C" fn ...`
                keyword = "fn"
            else:
                continue
        elif keyword in ("impl", "trait"):
            covered.add(lineno(match.start()))
            continue
        elif keyword != "fn":
            continue  # `unsafe(no_mangle)` and friends: not a region

        if body_at is None:
            named = WHITESPACE.match(masked, word.end()).end()
            if named < len(masked) and masked[named] == "(":
                continue  # an `unsafe fn(..)` *type*; paid at the call site
            if deny:
                continue  # the body's own blocks state its unsafe surface
            brace = masked.find("{", named)
            semi = masked.find(";", named)
            if brace < 0 or 0 <= semi < brace:
                covered.add(lineno(match.start()))  # a bodyless declaration
                continue
            body_at = brace

        span = range(lineno(match.start()), lineno(matching_brace(masked, body_at)) + 1)
        covered.update(span)
    return len(covered & code_lines)


def measure():
    """(repo-relative file -> {metric: count} with zeros included,
    number of files not carrying the forbid attribute,
    number of files carrying neither forbid nor the unsafe-op deny)."""
    stats = {}
    without_forbid = 0
    without_deny = 0
    for path in sorted(
        [*ROOT.glob("crates/*/src/**/*.rs"), *ROOT.glob("crates/*/*.rs")]
    ):
        text = path.read_text()
        masked = mask(text)
        counts = {
            name: sum(masked.count(needle) for needle in needles)
            for name, needles in COUNTED.items()
        }
        counts["unsafe_lines"] = unsafe_lines(masked, DENY_UNSAFE_OP in masked)
        counts["lines"] = len(text.splitlines())
        stats[str(path.relative_to(ROOT))] = counts
        without_forbid += FORBID not in masked
        without_deny += FORBID not in masked and DENY_UNSAFE_OP not in masked
    return stats, without_forbid, without_deny


def internal_exports():
    if not LEDGER.exists():
        sys.exit(f"ratchet: {LEDGER.relative_to(ROOT)} is missing; run `just refresh`")
    return sum(
        json.loads(line)["class"] == "internal"
        for line in LEDGER.read_text().splitlines()
    )


def render(stats, internal, without_forbid, without_deny):
    """The baseline document: only metrics with ratchet room are recorded
    (nonzero counts, over-cap line counts), so files that are already clean
    and under the cap don't churn the file as they're edited."""
    entries = []
    for file, counts in sorted(stats.items()):
        kept = {
            name: n
            for name, n in counts.items()
            if n > (LINE_CAP if name == "lines" else 0)
        }
        if kept:
            entries.append(
                f"    {json.dumps(file)}: {json.dumps(kept, sort_keys=True)}"
            )
    body = ",\n".join(entries)
    return (
        "{\n"
        f'  "internal_exports": {internal},\n'
        f'  "files_without_forbid_unsafe": {without_forbid},\n'
        f'  "files_without_deny_unsafe_op": {without_deny},\n'
        f'  "files": {{\n{body}\n  }}\n'
        "}\n"
    )


def violations(stats, internal, without_forbid, without_deny, baseline):
    """Every metric that grew past the committed baseline."""
    found = []
    base_internal = baseline["internal_exports"]
    if internal > base_internal:
        found.append(f"abi-ledger internal exports: {base_internal} -> {internal}")
    # .get: absent from baselines committed before the metric existed.
    base_forbid = baseline.get("files_without_forbid_unsafe", without_forbid)
    if without_forbid > base_forbid:
        found.append(f"files without {FORBID}: {base_forbid} -> {without_forbid}")
    base_deny = baseline.get("files_without_deny_unsafe_op", without_deny)
    if without_deny > base_deny:
        found.append(
            f"files without {FORBID} or {DENY_UNSAFE_OP}: {base_deny} -> {without_deny}"
        )
    base_files = baseline["files"]
    counted = (*COUNTED, "unsafe_lines")
    for file in sorted(stats.keys() | base_files.keys()):
        cur = stats.get(file, {**dict.fromkeys(counted, 0), "lines": 0})
        base = base_files.get(file, {})
        for name in counted:
            if cur[name] > base.get(name, 0):
                found.append(f"{file}: {name} {base.get(name, 0)} -> {cur[name]}")
        limit = max(LINE_CAP, base.get("lines", 0))
        if cur["lines"] > limit:
            grandfathered = " (grandfathered)" if limit > LINE_CAP else ""
            found.append(f"{file}: {cur['lines']} lines > {limit}{grandfathered}")
    return found


def summary(stats, internal, without_forbid, without_deny):
    counted = (*COUNTED, "unsafe_lines")
    totals = {name: sum(c[name] for c in stats.values()) for name in counted}
    over = sum(c["lines"] > LINE_CAP for c in stats.values())
    parts = [f"{n} {name}" for name, n in totals.items()]
    parts += [
        f"{over} files over {LINE_CAP} lines",
        f"{internal} internal exports",
        f"{without_forbid} files without forbid(unsafe_code)",
        f"{without_deny} files also without deny(unsafe_op_in_unsafe_fn)",
    ]
    return ", ".join(parts)


# Scanner cases, checked on every run (a few hundred microseconds against a
# ~20 MB tree read). A silent regression in mask()/unsafe_lines() would
# corrupt every number the ratchet enforces, so this is not opt-in.
SELF_TEST = [
    # (source, expected unsafe_lines in a file without the deny)
    ("fn f() {\n    unsafe {\n        g();\n    }\n}\n", 3),
    ("fn f() {\n    let x = unsafe { *p };\n}\n", 1),
    # Comments and blank lines inside a block are free.
    ("fn f() {\n    unsafe {\n        // SAFETY: fine.\n\n        g();\n    }\n}\n", 3),
    # Prose and strings never count.
    ("/// An unsafe fn would need one.\n/// unsafe { }\nfn f() {}\n", 0),
    ('fn f() {\n    let s = "unsafe { g(); }";\n}\n', 0),
    ('fn f() {\n    let s = r#"unsafe {"#;\n}\n', 0),
    ("/* unsafe { /* nested */ } */\nfn f() {}\n", 0),
    # A brace in a char literal must not desynchronise the scanner.
    ("fn f() {\n    unsafe {\n        g('{');\n    }\n    h();\n}\n", 3),
    # ... and neither must an escaped one. `'\\'` ends at its own quote; a
    # scanner that reads the second backslash as an escape runs on to the
    # next quote in the file and eats every brace in between.
    ("fn f() {\n    unsafe {\n        g('\\\\');\n    }\n    h('x');\n}\n", 3),
    ("fn f() {\n    unsafe {\n        g('\\'');\n    }\n    h('x');\n}\n", 3),
    ("fn f() {\n    unsafe {\n        g('\\u{1b}');\n    }\n    h('x');\n}\n", 3),
    ("fn f<'a>(x: &'a u8) {}\n", 0),
    # An unsafe fn body is implicitly unsafe throughout without the deny.
    ("unsafe fn f() {\n    g();\n}\n", 3),
    ('unsafe extern "C" fn f() {\n    g();\n}\n', 3),
    ("trait T {\n    unsafe fn f();\n}\n", 1),
    # Nested blocks are unioned, not summed.
    ("unsafe fn f() {\n    unsafe {\n        g();\n    }\n}\n", 5),
    # Declarations, promises, types.
    ("unsafe impl Sync for X {}\n", 1),
    ("unsafe trait T {}\n", 1),
    ('type F = unsafe extern "C" fn(u8);\n', 0),
    ("struct S(Option<unsafe fn(u8)>);\n", 0),
    ('#[unsafe(no_mangle)]\npub extern "C" fn f() {}\n', 0),
    ('unsafe extern "C" {\n    static x: u8;\n}\n', 3),
]
SELF_TEST_DENY = [
    # With the deny, a body's own blocks state its unsafe surface.
    ("unsafe fn f() {\n    g();\n}\n", 0),
    ("unsafe fn f() {\n    unsafe {\n        g();\n    }\n}\n", 3),
]


def self_test():
    for source, expected in SELF_TEST:
        got = unsafe_lines(mask(source), False)
        assert got == expected, f"unsafe_lines={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_DENY:
        got = unsafe_lines(mask(source), True)
        assert got == expected, f"unsafe_lines={got}, want {expected}, for {source!r}"


def main():
    args = set(sys.argv[1:])
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"ratchet: unknown argument(s): {' '.join(sorted(unknown))}")

    self_test()
    stats, without_forbid, without_deny = measure()
    internal = internal_exports()
    content = render(stats, internal, without_forbid, without_deny)
    committed = BASELINE.read_text() if BASELINE.exists() else None

    if "--check" in args:
        if committed is None:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is missing; run `just refresh`"
            )
        if grew := violations(
            stats, internal, without_forbid, without_deny, json.loads(committed)
        ):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "ratchet: counts may only shrink. Reduce them, or if the "
                "growth is justified run `just refresh --allow-growth` and "
                "explain it in the commit message."
            )
        if committed != content:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is stale (progress "
                "to lock in); run `just refresh` and commit the result"
            )
        return

    if committed is not None and "--allow-growth" not in args:
        if grew := violations(
            stats, internal, without_forbid, without_deny, json.loads(committed)
        ):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "ratchet: refusing to raise the baseline. If the growth is "
                "justified, rerun with --allow-growth."
            )
    BASELINE.write_text(content)
    print(
        f"wrote {BASELINE.relative_to(ROOT)}: "
        f"{summary(stats, internal, without_forbid, without_deny)}"
    )


if __name__ == "__main__":
    main()
