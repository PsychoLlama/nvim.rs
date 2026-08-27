#!/usr/bin/env python3
"""Prove that a narrowing-`unsafe` refactor changed nothing but the `unsafe`.

Phase 23's slices move `unsafe` around in bulk: a whole-body `unsafe { .. }`
becomes a dozen tight regions, an `unsafe fn` becomes a safe `fn`, a call site
loses a now-redundant wrapper. Those edits are supposed to be behaviour-
preserving *by construction*, and they touch thousands of lines, so reading
every hunk is not a real review. This script is the mechanical half of that
review: it erases exactly what such a refactor is allowed to change and diffs
what is left.

Per file it normalises both revisions by

  * dropping comments -- a SAFETY note is free, and adding one is the whole
    point of the exercise;
  * dropping the `unsafe` keyword wherever it appears, and the braces of an
    `unsafe { .. }` block along with it, so one wide region and ten narrow
    ones over the same statements collapse to the same text;
  * collapsing whitespace, so re-indentation and rustfmt's line breaking do
    not register.

String and character literals are kept verbatim: a lost trailing space in a C
format string is a real shipped bug, and this is the cheapest place to catch
one.

Whatever still differs is a change to the *code*, which a refactor claiming to
be `unsafe`-only has to justify. The output is a unified diff of the
normalised token streams, one token per line, so the offending token is easy
to see.

The class of regression this exists to catch, in the order they have actually
shipped here:

  1. `unsafe { *p }.a = 42;` -- a block is a value expression, so wrapping a
     dereference without its field projection writes to a discarded temporary
     when the pointee is `Copy`. The store is silently dropped; rustc, clippy
     and Miri all pass it. Normalised, the correct
     `unsafe { (*p).a = 42 };` keeps its parentheses and the broken form does
     not, so the two do not match.
  2. A dereference hoisted out of `&&`/`||`, where the short circuit was
     guarding a null pointer on the right-hand side. The moved binding lands
     in a different place in the token stream.
  3. `let save = x.get(); ..; x.set(save)` collapsed to `x.set(x.get())`,
     which restores what the intervening call left rather than what was
     saved. The vanished binding shows up as a removed token run.
  4. A C format string quietly reflowed while its `unsafe` region was being
     rewritten around it.

It reports candidates, not verdicts: a hunk here means "a human reads this
one", and a legitimate rewrite (adding a `let` to shorten a line, say) will
show up too. Silence is the useful answer, and it is the common one.

    scripts/unsafe-diff.py                    # working tree against HEAD
    scripts/unsafe-diff.py --base v0.12.4     # against a tag
    scripts/unsafe-diff.py crates/nvim/src/undo    # only these paths
"""

import argparse
import difflib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The keywords `unsafe` may qualify. `unsafe {` is handled separately: only
# there does a brace pair disappear with the keyword.
QUALIFIED = ("fn", "impl", "trait", "extern")


def raw_string_end(src, i):
    """The index just past the raw string starting at `i`, or None.

    `i` is the `r`; a raw string is `r`, some `#`s, then a quoted body that
    ends at the same number of `#`s. Nothing inside it is escapable, so an
    ordinary string scan would stop at the first quote it holds.
    """
    n = len(src)
    j, hashes = i + 1, 0
    while j < n and src[j] == "#":
        hashes, j = hashes + 1, j + 1
    if j >= n or src[j] != '"':
        return None
    close = '"' + "#" * hashes
    end = src.find(close, j + 1)
    return n if end < 0 else end + len(close)


def literal_end(src, i):
    """The index just past the string or char literal at `i`, or None.

    None means the quote at `i` opens no literal: in Rust a lone `'` is far
    more often a lifetime (`&'a T`, `for<'de>`) than a character, and the two
    are only told apart by looking for the closer.
    """
    n = len(src)
    quote = src[i]
    if quote == '"':
        j = i + 1
        while j < n:
            if src[j] == "\\":
                j += 2
                continue
            if src[j] == '"':
                return j + 1
            j += 1
        return n
    # A character literal is short and holds no whitespace; a lifetime runs
    # into an identifier and stops.
    j = i + 1
    while j < n and j - i <= 8:
        if src[j] == "\\":
            j += 2
            continue
        if src[j] == "'":
            return j + 1
        if src[j].isspace():
            break
        j += 1
    return None


def strip_comments(src):
    """`src` with every comment replaced by a space, literals untouched.

    Rust's block comments nest and its raw strings swallow quotes, so this
    walks the text rather than reaching for a regular expression.
    """
    out = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            while i < n and src[i] != "\n":
                i += 1
            out.append(" ")
        elif c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, i = 1, i + 2
            while i < n and depth:
                if src.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif src.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            out.append(" ")
        elif c == "r" and (end := raw_string_end(src, i)) is not None:
            out.append(src[i:end])
            i = end
        elif c in "\"'":
            end = literal_end(src, i)
            if end is None:
                out.append(c)
                i += 1
            else:
                out.append(src[i:end])
                i = end
        else:
            out.append(c)
            i += 1
    return "".join(out)


def matching_brace(src, at):
    """The index just past the `}` closing the `{` at `at`."""
    depth, i, n = 0, at, len(src)
    while i < n:
        if src[i] == "{":
            depth += 1
        elif src[i] == "}":
            depth -= 1
            if not depth:
                return i + 1
        elif src[i] in "\"'":
            end = literal_end(src, i)
            if end is not None:
                i = end
                continue
        i += 1
    return n


def is_word_at(src, i, word):
    """Whether `word` sits at `i` as a whole token."""
    if not src.startswith(word, i):
        return False
    before = src[i - 1] if i else " "
    after = src[i + len(word)] if i + len(word) < len(src) else " "
    return not (before.isalnum() or before == "_") and not (
        after.isalnum() or after == "_"
    )


def strip_unsafe(src):
    """`src` with every `unsafe` gone, and `unsafe {}`'s braces with it."""
    out = []
    i, n = 0, len(src)
    while i < n:
        if src[i] == "r" and (end := raw_string_end(src, i)) is not None:
            out.append(src[i:end])
            i = end
            continue
        if src[i] in "\"'":
            end = literal_end(src, i)
            if end is not None:
                out.append(src[i:end])
                i = end
                continue
        if not is_word_at(src, i, "unsafe"):
            out.append(src[i])
            i += 1
            continue
        after = i + len("unsafe")
        while after < n and src[after].isspace():
            after += 1
        if after < n and src[after] == "{":
            end = matching_brace(src, after)
            # The block's contents stay; its own braces go, so a wide region
            # and a narrow one over the same statements read alike.
            out.append(" ")
            out.append(strip_unsafe(src[after + 1 : end - 1]))
            out.append(" ")
            i = end
            continue
        if any(is_word_at(src, after, kw) for kw in QUALIFIED):
            out.append(" ")
            i = after
            continue
        # `unsafe(no_mangle)` and anything else: drop the keyword only.
        out.append(" ")
        i = after
    return "".join(out)


def normalise(src):
    """The comparable form: one statement per line, no comments, no `unsafe`."""
    toks = drop_trailing_commas(tokens(strip_unsafe(strip_comments(src))))
    return statements(drop_use_items(toks))


def drop_use_items(toks):
    """`toks` without its `use ..;` items.

    Narrowing a region routinely adds or removes an import, and rustfmt
    reorders the list when it does. None of that is behaviour, and left in it
    swamps the diff.
    """
    out = []
    i, n = 0, len(toks)
    while i < n:
        starts_item = not out or out[-1] in (";", "{", "}")
        if toks[i] == "use" and starts_item:
            while i < n and toks[i] != ";":
                i += 1
            i += 1
            continue
        out.append(toks[i])
        i += 1
    return out


def statements(toks):
    """`toks` regrouped one statement per line, so a hunk is readable."""
    lines, cur = [], []
    for t in toks:
        cur.append(t)
        if t in ";{}":
            lines.append(" ".join(cur))
            cur = []
    if cur:
        lines.append(" ".join(cur))
    return lines


def drop_trailing_commas(toks):
    """`toks` without the comma rustfmt adds when it breaks a call up.

    Narrowing a region moves calls across the 60-column threshold in both
    directions, and the comma that appears or vanishes with the reflow says
    nothing about behaviour.
    """
    return [
        t
        for i, t in enumerate(toks)
        if not (t == "," and i + 1 < len(toks) and toks[i + 1] in ")]}")
    ]


def tokens(src):
    """`src` split into tokens, whitespace collapsed away."""
    out = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c.isspace():
            i += 1
        elif c == "r" and (end := raw_string_end(src, i)) is not None:
            out.append(src[i:end])
            i = end
        elif c in "\"'" and (end := literal_end(src, i)) is not None:
            out.append(src[i:end])
            i = end
        elif c.isalnum() or c == "_":
            j = i
            while j < n and (src[j].isalnum() or src[j] == "_"):
                j += 1
            out.append(src[i:j])
            i = j
        else:
            out.append(c)
            i += 1
    return out


# (before, after, whether the two should normalise alike). Every "differ"
# case is a regression this has actually shipped; every "match" case is an
# edit a narrowing slice is expected to make freely.
SELF_TEST = [
    # A whole-body region split into tight ones over the same statements.
    (
        "unsafe fn f(p: *mut T) {\n // SAFETY: c\n unsafe { (*p).a = 1; (*p).b = 2; }\n}",
        "fn f(p: *mut T) {\n // SAFETY: a\n unsafe { (*p).a = 1 };\n"
        " // SAFETY: b\n unsafe { (*p).b = 2 };\n}",
        True,
    ),
    # The dropped store: `unsafe { *p }.a` writes to a temporary.
    ("unsafe { (*p).a = 42 };", "unsafe { *p }.a = 42;", False),
    # A dereference hoisted out of the `&&` that was guarding it.
    (
        "if !p.is_null() && unsafe { (*p).x } { }",
        "let x = unsafe { (*p).x };\nif !p.is_null() && x { }",
        False,
    ),
    # A saved global inlined into its own restore.
    ("let s = x.get();\nf();\nx.set(s);", "f();\nx.set(x.get());", False),
    # A C format string that lost its trailing space.
    ('c"E96: two %d "', 'c"E96: two %d"', False),
    # rustfmt breaking a call up, trailing comma and all.
    (
        "let x = unsafe { g(a, b, c) };",
        "let x = unsafe {\n g(\n a, b,\n c,\n )\n};",
        True,
    ),
    ("pub unsafe fn f() -> i32 { 1 }", "pub fn f() -> i32 { 1 }", True),
    ("unsafe impl Send for T {}", "impl Send for T {}", True),
    ("unsafe { unsafe { g() } }", "g()", True),
    # A raw string holds quotes; a lifetime is not a character literal.
    (
        'let s = r#"a"b"#; fn f<\'a>(x: &\'a T) {}',
        'let s = r#"a"b"#;\nfn f<\'a>(x: &\'a T) {}',
        True,
    ),
    ('let s = r#"a"b"#;', 'let s = r#"a"c"#;', False),
    ("let c = 'x';", "let c = 'y';", False),
    ("let a = 1; /* x /* y */ z */ let b = 2;", "let a = 1;\nlet b = 2;", True),
    # An import added beside a narrowed region.
    ("use core::ptr;\nf();", "use core::ptr;\nuse crate::winlayer::Buf;\nf();", True),
    # A newtype threaded through a call is a real change, and shows.
    ("f(buf)", "f(Buf::new(buf))", False),
]


def self_test():
    """The normaliser's own cases; `main` runs them before it reports."""
    for before, after, want_same in SELF_TEST:
        got = normalise(before) == normalise(after)
        assert got == want_same, (
            f"normalise: {'matched' if got else 'differed'}, want "
            f"{'match' if want_same else 'differ'}, for {before!r} -> {after!r}"
        )


def changed_files(base, paths):
    """The Rust files that differ from `base`, restricted to `paths`."""
    cmd = ["git", "diff", "--name-only", base]
    if paths:
        cmd += ["--", *paths]
    out = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT).stdout
    return [f for f in out.split() if f.endswith(".rs")]


def at(base, path):
    """`path`'s contents at `base`, or None when it did not exist there."""
    got = subprocess.run(
        ["git", "show", f"{base}:{path}"], capture_output=True, text=True, cwd=ROOT
    )
    return got.stdout if got.returncode == 0 else None


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("paths", nargs="*", help="limit to these paths")
    ap.add_argument("--base", default="HEAD", help="revision to compare against")
    args = ap.parse_args()

    self_test()
    flagged = 0
    for path in changed_files(args.base, args.paths):
        old = at(args.base, path)
        if old is None:
            continue  # a new file has nothing to have changed
        try:
            new = (ROOT / path).read_text()
        except OSError:
            continue  # deleted
        before, after = normalise(old), normalise(new)
        if before == after:
            continue
        flagged += 1
        print(f"=== {path}")
        diff = difflib.unified_diff(before, after, lineterm="", n=3)
        for line in list(diff)[2:]:
            print(f"  {line}")
        print()

    if flagged:
        print(f"{flagged} file(s) changed more than their `unsafe`; read each hunk.")
    else:
        print("no file changed anything but its `unsafe`.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
