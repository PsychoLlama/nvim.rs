#!/usr/bin/env python3
"""Ledger every `pub` item that exists only because an integration test names it.

The ABI ledger answers "who resolves this symbol *by name*". With the C
exports retired that question is nearly closed (49 left), but the pressure it
applied did not disappear -- it changed form. The tree's other public surface
is Rust visibility, and the same tension is there: an entry point stays `pub`
because `crates/nvim/tests/` links the library from *outside* and drives it,
which is invisible to every reader of the module it lives in.

This is that ledger. One record per `pub` item an integration test reaches,
naming the test, sorted by path, e.g.

    jq -r 'select(.tests | length == 1) | .path' metrics/visibility-ledger.jsonl

Shrink-only, through `metrics/ratchet.json`'s `test_reached_pub`: an item may
leave the ledger (its test moved in-crate, or stopped needing the entry point)
but a new one is growth that has to be justified like any other.

It reads the **consumer** side first, and that is the whole design.
------------------------------------------------------------------------

The ABI ledger's structural flaw is that it enumerates the names that *are*
exported and asks who wants them. A name that should be exported and is not
falls out of its world entirely -- it has no record, so nothing can be stale,
and the failure surfaces as a runtime "undefined symbol" from one functional
spec a slice later (`build_stl_str_hl` sat broken that way). Half of that hole
was closed by reading the specs' own `ffi.cdef` declarations and failing when
one names a symbol that is gone.

The same hole would be here if this script enumerated `pub` items and asked
which tests use them. So it does the opposite: it parses the `use
neovim::...` trees out of `crates/nvim/tests/`, and every path it finds
must resolve to something the crate actually declares or re-exports. A path it
cannot account for is an **error**, not a dropped row -- because the one way
this ledger could go quietly wrong is a scanner blind spot (a re-export chain,
an inline module) silently shrinking the count and reading as progress.

Two things make the Rust side genuinely safer than the C side, and both are
worth saying out loud rather than being relied on silently:

  * Narrowing an item a test names is a **compile** error, in the same commit,
    with the path in the message. The linker's silence has no analogue here.
    This ledger is therefore a classifier, not a guard -- as the ABI ledger
    always was -- and its job is to tell a future reader *why* an item is
    `pub` before they try to narrow it.
  * `unreachable_pub` is `deny` in the package (see crates/nvim/Cargo.toml),
    so `pub` in this crate means *externally reachable*. Without that, "the
    crate's public surface" would not be a well-defined set and neither the
    ledger nor the `pub_items` total below would mean anything.

What the ledger still cannot see is the other direction -- an item that is
`pub` for no reason at all. That is not a per-item question (there are ~15k of
them), so it is a whole-tree count instead: `pub_items` in metrics/ratchet.json,
the size of the crate's externally reachable surface, measured there because it
is a source needle like every other thing that file counts. The two numbers
together are the boundary: `test_reached_pub` is the part of it that is
*earned*, and today that is 672 of 15,309.

Usage: visibility-ledger.py [--check]
  --check   regenerate and diff against the committed ledger; exit 1 on drift
            instead of writing.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "metrics" / "visibility-ledger.jsonl"
CRATE = "neovim"
SRC = ROOT / "crates" / "nvim" / "src"
TESTS = ROOT / "crates" / "nvim" / "tests"

# A top-level item declaration: `pub` at column 0, which is where rustfmt puts
# every module-level item. An indented `pub fn` is an associated item or lives
# in an inline module, and neither is nameable through a `use` path.
DECL = re.compile(
    r'^pub (?:unsafe )?(?:extern "C(?:-unwind)?" )?'
    r"(fn|static mut|static|const|struct|enum|union|trait|type|mod) ([A-Za-z0-9_]+)",
    re.M,
)
# A `pub use` at column 0, up to its `;`. Multi-line trees included.
REEXPORT = re.compile(r"^pub use ([^;]*);", re.M | re.S)
# Comment lines, which are stripped before any of the above run.
COMMENT = re.compile(r"^\s*//.*$", re.M)

# Names cargo makes available under the crate root that are not items of it.
NOT_ITEMS = {"self", "super", "crate"}


def strip_comments(text):
    return COMMENT.sub("", text)


def module_file(parts):
    """The file holding module `parts` (a list of segments), or None."""
    rel = Path(*[p.removeprefix("r#") for p in parts]) if parts else Path()
    for candidate in (SRC / rel.with_suffix(".rs"), SRC / rel / "mod.rs"):
        if candidate.is_file():
            return candidate
    return SRC / "lib.rs" if not parts else None


def parse_use_tree(text, start):
    """Flatten one `use` tree into paths. `start` indexes just past `use `.

    Returns (paths, end) where each path is a list of segments; a glob is the
    segment `*` and `self` is dropped in favour of its prefix.
    """
    paths = []
    prefix = []
    segment = ""
    i = start
    while i < len(text):
        ch = text[i]
        if ch == ";":
            break
        if ch == "{":
            inner, i = parse_use_tree(text, i + 1)
            base = prefix + ([segment] if segment else [])
            paths.extend(base + p for p in inner)
            segment = ""
            continue
        if ch == "}":
            i += 1
            break
        if ch == ":" and text[i : i + 2] == "::":
            if segment:
                prefix.append(segment)
                segment = ""
            i += 2
            continue
        if ch == ",":
            if segment:
                paths.append(prefix + [segment])
            segment = ""
            i += 1
            continue
        if ch.isspace():
            i += 1
            continue
        segment += ch
        i += 1
    if segment:
        paths.append(prefix + [segment])
    # `use a::{self, b}` names `a` itself.
    return [p[:-1] if p and p[-1] == "self" else p for p in paths], i


def crate_paths(text):
    """Every `neovim::…` path a test source names."""
    text = strip_comments(text)
    out = []
    imported = []
    for m in re.finditer(r"\buse\s+", text):
        paths, end = parse_use_tree(text, m.end())
        imported.append((m.start(), end))
        for path in paths:
            if path and path[0] == CRATE:
                # `as` renames leave the alias behind; the item is the segment
                # before it.
                if "as" in path:
                    path = path[: path.index("as")]
                out.append(path[1:])
    # Paths written inline rather than imported. The `use` spans are skipped:
    # a match inside one is the prefix of a tree already flattened above, and
    # counting it would book the module as reached in its own right.
    for m in re.finditer(rf"\b{CRATE}((?:::[A-Za-z0-9_#]+)+)", text):
        if any(a <= m.start() < b for a, b in imported):
            continue
        out.append([s for s in m.group(1).split("::") if s])
    return [p for p in out if p and p[-1] not in NOT_ITEMS]


def declarations(path: Path):
    """(name -> kind for what this file declares, name -> origin module for
    what it re-exports, whether it re-exports a glob)."""
    text = strip_comments(path.read_text())
    declared = {name: kind.replace(" ", "-") for kind, name in DECL.findall(text)}
    through = {}
    globbed = False
    for tree in REEXPORT.findall(text):
        paths, _ = parse_use_tree(tree + ";", 0)
        for p in paths:
            if not p:
                continue
            if "as" in p:
                p = p[: p.index("as")]
            if p[-1] == "*":
                globbed = True
            else:
                through.setdefault(p[-1], p[:-1])
    return declared, through, globbed


def origin(segments, module):
    """`pub use <segments>::name` seen in `module` -> the module it comes from.

    None for a path this script does not follow, which in practice means an
    external crate: edition 2018 onwards every in-crate `use` starts with one
    of these three.
    """
    if not segments:
        return None
    if segments[0] == "crate":
        return segments[1:]
    if segments[0] == "self":
        return [*module, *segments[1:]]
    if segments[0] == "super":
        return [*module[:-1], *segments[1:]]
    holder = module_file(module)
    if holder is not None and segments[0] in children(holder):
        # Edition 2018 uniform paths: `pub use file::{..}` names a child
        # module without spelling `self::`, and half the tree writes it that
        # way.
        return [*module, *segments]
    return None


def declaring(module, item, seen=()):
    """Chase re-exports to the file that really declares `item`.

    A row whose `file` is the module the test imports from says nothing about
    where the item lives; the header modules are re-exported through half the
    tree. Answers (file, kind), falling back to the importing module when the
    chain leaves what this script can follow.
    """
    holder = module_file(module)
    if holder is None or holder in seen:
        return None
    declared, through, globbed = declarations(holder)
    if item in declared:
        return holder, declared[item]
    if item in through:
        nxt = origin(through[item], module)
        if nxt is not None:
            found = declaring(nxt, item, (*seen, holder))
            if found is not None:
                return found
        return holder, "use"
    if globbed:
        # A glob re-export -- `pub use self::<part>::*`, which is how every
        # split-up transpiled module puts itself back together. The children
        # come from the filesystem rather than the `mod` declarations: those
        # are private (the glob is what makes their contents visible), and a
        # scan keyed on `pub mod` would find none of them.
        for child in sorted(children(holder)):
            found = declaring([*module, child], item, (*seen, holder))
            if found is not None:
                return found
    return None


def children(holder: Path):
    """The child modules of the module `holder` defines."""
    directory = holder.parent if holder.name == "mod.rs" else holder.with_suffix("")
    if not directory.is_dir():
        return []
    return [
        entry.stem if entry.is_file() else entry.name
        for entry in directory.iterdir()
        if (entry.is_file() and entry.suffix == ".rs" and entry.stem != "mod")
        or (entry.is_dir() and (entry / "mod.rs").is_file())
    ]


def resolve(path):
    """(module segments, item, declaring file, kind) for one crate path."""
    if path and module_file(path) is not None:
        # The path names a module outright.
        return "::".join(path), None, module_file(path), "mod"
    for cut in range(len(path) - 1, -1, -1):
        if module_file(path[:cut]) is None:
            continue
        item = path[cut]
        if item == "*":
            return "::".join(path[:cut]), item, module_file(path[:cut]), "glob"
        found = declaring(path[:cut], item)
        if found is None:
            # A glob re-export the chase could not localise -- an inline
            # module, a macro-declared item, a name that came from another
            # crate. The evidence that the item is there is the glob itself,
            # so the row names the module the test imports from.
            holder = module_file(path[:cut])
            if declarations(holder)[2]:
                found = (holder, "use")
        if found is not None:
            return "::".join(path[:cut]), item, found[0], found[1]
        break
    return None


def build():
    reached = {}
    for spec in sorted(TESTS.rglob("*.rs")):
        rel = str(spec.relative_to(ROOT))
        for path in crate_paths(spec.read_text()):
            found = resolve(path)
            if found is None:
                sys.exit(
                    f"visibility-ledger: {rel} names "
                    f"`{CRATE}::{'::'.join(path)}`, which no module under "
                    f"{SRC.relative_to(ROOT)} declares or re-exports.\n"
                    "Either the path moved, or this script cannot follow the "
                    "re-export that reaches it — teach it, do not drop the row."
                )
            module, item, holder, kind = found
            key = (module, item)
            reached.setdefault(key, {"file": holder, "kind": kind, "tests": set()})
            reached[key]["tests"].add(rel)

    lines = []
    for (module, item), info in sorted(
        reached.items(), key=lambda kv: (kv[0][0], kv[0][1] or "")
    ):
        full = f"{CRATE}::{module}" + (f"::{item}" if item else "")
        record = {
            "path": full,
            "kind": info["kind"],
            "file": str(info["file"].relative_to(ROOT)),
            "tests": sorted(info["tests"]),
        }
        lines.append(json.dumps(record, separators=(",", ":")))
    return "\n".join(lines) + "\n", len(lines)


def main():
    content, count = build()
    if "--check" in sys.argv[1:]:
        committed = LEDGER.read_text() if LEDGER.exists() else ""
        if committed != content:
            sys.exit(
                f"visibility-ledger: {LEDGER.relative_to(ROOT)} is stale; "
                "run `just refresh` and commit the result"
            )
        return
    LEDGER.parent.mkdir(exist_ok=True)
    LEDGER.write_text(content)
    print(f"wrote {LEDGER.relative_to(ROOT)}: {count} test-reached pub items")


if __name__ == "__main__":
    main()
