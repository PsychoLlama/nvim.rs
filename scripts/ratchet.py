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
  extern_abi  definitions of functions carrying a C ABI — `extern "C" fn
              name` / `extern "C-unwind" fn name`, i.e. the named form, so an
              `extern` *type* (a function pointer) and a declaration inside an
              `unsafe extern` block are not counted. Each one is a signature
              the compiler cannot check across and that clippy skips wholesale,
              and phase 18's census classed every survivor: exported to C
              (`#[unsafe(no_mangle)]`, an abi-ledger entry), address-taken by a
              declared C caller (lua_CFunction, libvterm, libuv, qsort), a
              variadic (where `...` requires the ABI), or an api entry point
              whose ABI is only apigen's recognition convention. The last class
              retires as `api/` converts to `Result<T, Error>`; the rest retire
              as their C callers do.
  missing_safety_doc
              `unsafe fn`s whose doc comment has no `# Safety` section — the
              obligation the signature announces but nobody wrote down. It
              stands in for `clippy::missing_safety_doc`, which is *allowed*
              in Cargo.toml (2 380 findings when phase 19 switched the style
              tier on; that is a phase of work, not a slice, and it retires as
              modules are rewritten rather than by a sweep of stub sections).
              The lint could not be ratcheted where it was: clippy reports it
              at the crate lint level, so a site-level or module-level
              `#[allow]` cannot scope it, and leaving it warning would bury
              `just lint`'s ~20 real findings under 2 380.

              Counted here instead, from the source, which also makes it
              *broader* than the lint in the one direction that helps:
              clippy only asks exported functions, this asks every `unsafe fn`
              the tree defines or declares, because a private one's caller has
              the same obligation to discharge. Declarations inside an
              `unsafe extern` block are not counted (their obligation is the C
              library's) and neither is an `unsafe fn` *type*. The section is
              recognised as a `# Safety` heading in the run of `///` lines
              immediately above the item, attributes skipped — which is the
              shape rustfmt keeps and the shape every rewritten module already
              uses.

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

  files_without_deny_casts  the number of source files that have not adopted
                    the cast lints. `as` is the transpile's universal
                    conversion — ~21k of them, ~5k infallible widenings that
                    `From` answers and a long tail of narrowings that need
                    `TryFrom` and an error — and clippy's cast family
                    (`cast_lossless`, `cast_possible_truncation`,
                    `cast_possible_wrap`, `cast_sign_loss`, `ptr_as_ptr`) is
                    pedantic, i.e. allowed by default. Turning it on tree-wide
                    is a big-bang sweep nobody can review; the migration adopts
                    it per module, as the roadmap's phase 19 item 6 asks.

                    So the same inverted trick as the two attributes above: a
                    module that has finished its casts writes

                        #![deny(
                            clippy::cast_lossless,
                            clippy::cast_possible_truncation,
                            clippy::cast_possible_wrap,
                            clippy::cast_sign_loss,
                            clippy::ptr_as_ptr
                        )]

                    and the count of files *not* carrying it may only fall.
                    The needle is `clippy::cast_lossless` inside a `deny(...)`,
                    spanning newlines so rustfmt may wrap the list: that lint
                    is the one the `From` vocabulary answers, so it is the
                    family's marker. Naming the rest is the house convention,
                    not something the ratchet can check — as with
                    `forbid(unsafe_code)`, the attribute is a claim the
                    compiler then enforces, and the ratchet only counts who
                    has made it.

  files_without_deny_unsafe_op  the number of source files carrying neither
                    #![forbid(unsafe_code)] nor
                    #![deny(unsafe_op_in_unsafe_fn)]. Same trick for edition
                    2024's honest-unsafe lint: Cargo.toml allows it (blanket
                    body-wrapping would double the textual unsafe count), each
                    module denies it once its unsafe fns use explicit unsafe
                    blocks, and the count of files doing neither may only fall.
                    Phase 20 drove it to **0**: the crate root's inner
                    attribute is crate-level rather than per-module, so lib.rs
                    could only take it once every file beneath it already had
                    a marker — which is exactly the state the criterion
                    describes, and why lib.rs went last.

A `warnings` metric used to sit alongside it; phase 5 drove the count to
zero and the dev shell (flake.nix) now sets `RUSTFLAGS="-D warnings"` for
every local and CI build instead, so the counter is retired.

Everything is measured over a *masked* copy of the source, in which comments,
string literals and character literals are blanked out (offsets and newlines
preserved) so that only code is scanned. That is what makes the counts mean
what they say: prose about `unsafe` costs nothing, a doc comment quoting
`#![deny(unsafe_op_in_unsafe_fn)]` does not switch on the deny, and a string
containing a brace cannot desynchronise the block scanner. Everything else is
plain substring matching — bar `extern_abi`, whose needle is a regex because
masking erases the very ABI string a substring would key on — which still
over-counts a little (a macro naming
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
# name -> regex counted in the masked source, for what a substring cannot
# separate. Masking blanks the ABI string itself (`extern "C" fn` reads
# `extern     fn`), which is why this is a regex and why it is ABI-blind —
# every `extern` the tree writes names a C ABI. `fn <name>` is the definition
# form: a function-pointer type writes `fn(` with no name, and a declaration
# inside an `unsafe extern` block has the block's `{` between the two words.
COUNTED_RE = {
    "extern_abi": re.compile(r"\bextern\s+fn [A-Za-z_]"),
}
FORBID = "#![forbid(unsafe_code)]"
DENY_UNSAFE_OP = "#![deny(unsafe_op_in_unsafe_fn)]"
# A module's claim to have finished its casts. `.` spans newlines so the list
# may be wrapped; `clippy::cast_lossless` is the family's marker (see above).
DENY_CASTS = re.compile(r"#!\[deny\([^\]]*\bclippy::cast_lossless\b", re.DOTALL)
# A `# Safety` heading in a doc comment. Any heading level, any case, because
# what is being counted is whether the obligation is written down.
SAFETY_HEADING = re.compile(r"^\s*///\s*#+\s*safety\b", re.IGNORECASE)
DOC_LINE = re.compile(r"^\s*///")

# Metrics computed from the source rather than counted with a needle.
DERIVED = ("unsafe_lines", "missing_safety_doc")

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


def has_safety_doc(lines, at):
    """Whether a `# Safety` heading sits in the doc comment above line `at`.

    Attribute lines are skipped: they sit between the doc comment and the item.
    A doc comment hidden behind an attribute rustfmt wrapped over several lines
    would read as absent, which over-counts — the direction that keeps the
    ratchet honest.
    """
    i = at - 1
    while i >= 0 and lines[i].lstrip().startswith("#"):
        i -= 1
    while i >= 0 and DOC_LINE.match(lines[i]):
        if SAFETY_HEADING.match(lines[i]):
            return True
        i -= 1
    return False


def missing_safety_doc(text, masked):
    """`unsafe fn`s whose doc comment has no `# Safety` section.

    Walks the same `unsafe` keyword occurrences `unsafe_lines` does and keeps
    the ones that introduce a *function* — a definition or a bodyless
    declaration, never a function-pointer type and never a declaration inside
    an `unsafe extern` block, whose obligation is the C library's.
    """
    lines = text.splitlines()
    starts = [0, *(m.end() for m in re.finditer("\n", masked))]
    missing = 0
    skip_until = 0
    for match in UNSAFE_WORD.finditer(masked):
        if match.start() < skip_until:
            continue
        at = WHITESPACE.match(masked, match.end()).end()
        word = IDENT_AT.match(masked, at)
        keyword = word.group(0) if word else ""
        if keyword == "extern":
            after = WHITESPACE.match(masked, word.end()).end()
            follows = IDENT_AT.match(masked, after)
            if after < len(masked) and masked[after] == "{":
                skip_until = matching_brace(masked, after)
                continue
            if not (follows and follows.group(0) == "fn"):
                continue
            word = follows  # `unsafe extern "C" fn ...`
        elif keyword != "fn":
            continue  # `unsafe {`, `unsafe impl`, `unsafe(no_mangle)`, ...
        named = WHITESPACE.match(masked, word.end()).end()
        if named < len(masked) and masked[named] == "(":
            continue  # an `unsafe fn(..)` *type*; it has no docs to carry
        missing += not has_safety_doc(lines, bisect_right(starts, match.start()) - 1)
    return missing


def measure():
    """(repo-relative file -> {metric: count} with zeros included,
    number of files not carrying the forbid attribute,
    number of files carrying neither forbid nor the unsafe-op deny,
    number of files not carrying the cast deny)."""
    stats = {}
    without_forbid = 0
    without_deny = 0
    without_casts = 0
    for path in sorted(
        [*ROOT.glob("crates/*/src/**/*.rs"), *ROOT.glob("crates/*/*.rs")]
    ):
        text = path.read_text()
        masked = mask(text)
        counts = {
            name: sum(masked.count(needle) for needle in needles)
            for name, needles in COUNTED.items()
        }
        counts.update(
            (name, len(rx.findall(masked))) for name, rx in COUNTED_RE.items()
        )
        counts["unsafe_lines"] = unsafe_lines(masked, DENY_UNSAFE_OP in masked)
        counts["missing_safety_doc"] = missing_safety_doc(text, masked)
        counts["lines"] = len(text.splitlines())
        stats[str(path.relative_to(ROOT))] = counts
        without_forbid += FORBID not in masked
        without_deny += FORBID not in masked and DENY_UNSAFE_OP not in masked
        without_casts += DENY_CASTS.search(masked) is None
    return stats, without_forbid, without_deny, without_casts


def internal_exports():
    if not LEDGER.exists():
        sys.exit(f"ratchet: {LEDGER.relative_to(ROOT)} is missing; run `just refresh`")
    return sum(
        json.loads(line)["class"] == "internal"
        for line in LEDGER.read_text().splitlines()
    )


def render(stats, internal, without_forbid, without_deny, without_casts):
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
        f'  "files_without_deny_casts": {without_casts},\n'
        f'  "files": {{\n{body}\n  }}\n'
        "}\n"
    )


def violations(stats, internal, without_forbid, without_deny, without_casts, baseline):
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
    base_casts = baseline.get("files_without_deny_casts", without_casts)
    if without_casts > base_casts:
        found.append(f"files without the cast deny: {base_casts} -> {without_casts}")
    base_files = baseline["files"]
    counted = (*COUNTED, *COUNTED_RE, *DERIVED)
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


def summary(stats, internal, without_forbid, without_deny, without_casts):
    counted = (*COUNTED, *COUNTED_RE, *DERIVED)
    totals = {name: sum(c[name] for c in stats.values()) for name in counted}
    over = sum(c["lines"] > LINE_CAP for c in stats.values())
    parts = [f"{n} {name}" for name, n in totals.items()]
    parts += [
        f"{over} files over {LINE_CAP} lines",
        f"{internal} internal exports",
        f"{without_forbid} files without forbid(unsafe_code)",
        f"{without_deny} files also without deny(unsafe_op_in_unsafe_fn)",
        f"{without_casts} files without the cast deny",
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
# (source, expected extern_abi) — the needle is a regex over masked source,
# so it needs its own cases: masking has already erased the ABI string by the
# time it runs.
SELF_TEST_EXTERN_ABI = [
    ('pub unsafe extern "C" fn f() {}\n', 1),
    ('extern "C-unwind" fn f() {}\n', 1),
    ("extern fn f() {}\n", 1),
    # A function-pointer type is not a definition.
    ('type F = unsafe extern "C" fn(u8);\n', 0),
    ('struct S(Option<extern "C-unwind" fn(u8)>);\n', 0),
    # Neither is a declaration inside an extern block.
    ('unsafe extern "C" {\n    fn f(x: u8);\n}\n', 0),
    # Prose about one costs nothing.
    ('/// An extern "C" fn f() would.\nfn f() {}\n', 0),
]
# (source, expected missing_safety_doc). Reads the raw text as well as the
# masked copy, since the heading it looks for lives in a comment.
SELF_TEST_SAFETY_DOC = [
    ("unsafe fn f() {}\n", 1),
    ("/// # Safety\n/// Anything.\nunsafe fn f() {}\n", 0),
    ("/// # safety\nunsafe fn f() {}\n", 0),
    ("/// ## Safety\nunsafe fn f() {}\n", 0),
    # The section has to be this item's, not the one above it.
    ("/// # Safety\nunsafe fn f() {}\nunsafe fn g() {}\n", 1),
    # A blank line between ends the doc comment, so the heading is not f's.
    ("/// # Safety\n\nunsafe fn f() {}\n", 1),
    # Attributes sit between the comment and the item.
    ('/// # Safety\n#[unsafe(no_mangle)]\npub unsafe extern "C" fn f() {}\n', 0),
    # A trait's declaration carries the obligation too.
    ("trait T {\n    unsafe fn f();\n}\n", 1),
    # ... but a C library's does not.
    ('unsafe extern "C" {\n    unsafe fn f(x: u8);\n}\n', 0),
    ('unsafe extern "C" {\n    fn f(x: u8);\n}\n', 0),
    # Neither a function-pointer type nor a promise is a function.
    ('type F = unsafe extern "C" fn(u8);\n', 0),
    ("unsafe impl Sync for X {}\n", 0),
    ("fn f() {\n    unsafe {\n        g();\n    }\n}\n", 0),
    # Prose about one costs nothing.
    ("/// An unsafe fn f would.\nfn f() {}\n", 0),
]
# (source, whether the file counts as having adopted the cast lints)
SELF_TEST_DENY_CASTS = [
    ("#![deny(clippy::cast_lossless)]\n", True),
    (
        "#![deny(\n    clippy::cast_lossless,\n    clippy::ptr_as_ptr\n)]\n",
        True,
    ),
    ("#![deny(clippy::ptr_as_ptr)]\n", False),
    # Prose about the attribute does not switch it on.
    ("//! Adopt `#![deny(clippy::cast_lossless)]` here one day.\n", False),
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
    for source, expected in SELF_TEST_SAFETY_DOC:
        got = missing_safety_doc(source, mask(source))
        assert got == expected, (
            f"missing_safety_doc={got}, want {expected}, for {source!r}"
        )
    for source, expected in SELF_TEST_DENY_CASTS:
        got = DENY_CASTS.search(mask(source)) is not None
        assert got == expected, f"cast deny={got}, want {expected}, for {source!r}"
    needle = COUNTED_RE["extern_abi"]
    for source, expected in SELF_TEST_EXTERN_ABI:
        got = len(needle.findall(mask(source)))
        assert got == expected, f"extern_abi={got}, want {expected}, for {source!r}"


def main():
    args = set(sys.argv[1:])
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"ratchet: unknown argument(s): {' '.join(sorted(unknown))}")

    self_test()
    stats, without_forbid, without_deny, without_casts = measure()
    internal = internal_exports()
    content = render(stats, internal, without_forbid, without_deny, without_casts)
    committed = BASELINE.read_text() if BASELINE.exists() else None

    if "--check" in args:
        if committed is None:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is missing; run `just refresh`"
            )
        if grew := violations(
            stats,
            internal,
            without_forbid,
            without_deny,
            without_casts,
            json.loads(committed),
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
            stats,
            internal,
            without_forbid,
            without_deny,
            without_casts,
            json.loads(committed),
        ):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "ratchet: refusing to raise the baseline. If the growth is "
                "justified, rerun with --allow-growth."
            )
    BASELINE.write_text(content)
    print(
        f"wrote {BASELINE.relative_to(ROOT)}: "
        f"{summary(stats, internal, without_forbid, without_deny, without_casts)}"
    )


if __name__ == "__main__":
    main()
