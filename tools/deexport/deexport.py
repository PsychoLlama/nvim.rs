#!/usr/bin/env python3
"""deexport — the phase-5c rewriter.

Strips `#[no_mangle]` from internal-class exports and prunes the dead code
that de-exporting exposes.

Phase 5b moved cross-module resolution from the linker to the compiler, so
for most exports nobody resolves the symbol *name* anymore — the attribute
survives purely by inertia. Dropping it removes the symbol from the binary's
dynamic table (`--export-dynamic` exposes every `#[no_mangle]` item) and
frees the signature for later rewrites. The item itself stays
`pub extern "C" fn`: ABI and codegen are unchanged, only the linker-visible
name goes away.

Two symbol classes must keep their export, computed fresh from the tree
rather than trusted from a stale plan:

  test    class "test" in metrics/abi-ledger.jsonl — test/unit resolves
          these by name (ffi.C or unit-fixtures.so) until phase 6.
  residue any symbol still *declared* in a surviving `extern "C"` block
          (the phase-5b blacklist residue: terminal/quickfix/regexp/tui and
          friends). Those calls still resolve by name at link time; stripping
          the definition's export would turn them into undefined symbols.

`--prune` deletes de-exported items that nothing references: a top-level
`pub extern "C" fn` / `pub static` without `#[no_mangle]` whose name appears
nowhere else in the crate (word-boundary scan over src/**/*.rs + lib.rs,
`use` lines excluded) has no callers the compiler could see either —
`#[no_mangle]` was the only thing keeping it alive. Candidates come from the
tree, not the ledger: the refreshed ledger no longer lists what the strip
removed. rustc can't run this analysis itself — the crate is an rlib whose
transpiled items are all `pub`, so everything is externally reachable as far
as the dead_code lint is concerned. Deleting a function can orphan its callees and their
imports, so the scan runs to fixpoint, dropping the deleted names from
`use` lists as it goes. The scan over-counts references (a name in a
comment or string literal keeps its item), which errs toward keeping code —
never toward deleting something live. Mutual-recursion cycles with no
outside caller also survive (each member keeps the other alive); phase 8
gets those.

Dry-run by default; `--write` applies. Run `just refresh` afterwards — the
ledger shrinks by exactly the stripped symbols and the ratchet locks in the
new counts.
"""

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
LEDGER = ROOT / "metrics" / "abi-ledger.jsonl"

# Same shape abi-ledger.py recognizes: the attribute on its own line, the
# declaration on the next.
EXPORT_RE = re.compile(
    r'^pub (unsafe extern "C" fn|extern "C" fn|static mut|static) ([A-Za-z0-9_]+)'
)
EXTERN_ITEM_RE = re.compile(
    r"^\s*(?:pub\s+)?(fn|static|type)\s+(?:mut\s+)?(?:r#)?([A-Za-z_][A-Za-z0-9_]*)"
)
LINK_NAME_RE = re.compile(r'#\[link_name\s*=\s*"([^"]+)"\]')
ATTR_RE = re.compile(r"^\s*#\[")
USE_LINE_RE = re.compile(r"^\s*(?:pub\s+)?use\s")
TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def load_ledger():
    ledger = {}
    for line in LEDGER.read_text().splitlines():
        e = json.loads(line)
        ledger[e["symbol"]] = e
    return ledger


def rust_files():
    return sorted([*ROOT.glob("src/**/*.rs"), ROOT / "lib.rs"])


def extern_declared_names(texts):
    """Every name declared inside any `extern "C"` block (any nesting depth),
    plus every #[link_name] target. These still resolve by linker name."""
    names = set()
    for text in texts.values():
        lines = text.split("\n")
        i = 0
        while i < len(lines):
            if lines[i].lstrip().startswith('extern "C" {'):
                depth = lines[i].count("{") - lines[i].count("}")
                i += 1
                while i < len(lines) and depth > 0:
                    depth += lines[i].count("{") - lines[i].count("}")
                    if depth <= 0:
                        break
                    m = EXTERN_ITEM_RE.match(lines[i])
                    if m and m.group(1) != "type":
                        names.add(m.group(2))
                    names.update(LINK_NAME_RE.findall(lines[i]))
                    i += 1
            i += 1
    return names


def strip_exports(texts, ledger, keep, write):
    """Remove the #[no_mangle] line above every internal-class export not in
    `keep`. Returns per-file stripped counts."""
    stripped = defaultdict(int)
    for rel, text in sorted(texts.items()):
        lines = text.split("\n")
        out = []
        changed = False
        for i, line in enumerate(lines):
            if line.strip() == "#[no_mangle]" and i + 1 < len(lines):
                m = EXPORT_RE.match(lines[i + 1])
                if m:
                    name = m.group(2)
                    e = ledger.get(name)
                    if e and e["class"] == "internal" and name not in keep:
                        stripped[rel] += 1
                        changed = True
                        continue
            out.append(line)
        if changed and write:
            (ROOT / rel).write_text("\n".join(out))
    return stripped


WORD_RE_CACHE = {}


def word_re(name):
    r = WORD_RE_CACHE.get(name)
    if r is None:
        r = re.compile(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])")
        WORD_RE_CACHE[name] = r
    return r


CLOSER_RE = re.compile(r"^[}\])]")


def item_span(lines, def_line, kind):
    """(start, end) inclusive 0-based line span of the item at `def_line`:
    contiguous attribute lines above, through the closing line, or None when
    the end can't be established. Formatted-tree assumptions: a top-level
    item's interior is indented and its closer sits at column 0 (`}` for
    fns; `};` / `});` / `]);` … for statics). The first column-0 line that
    is *not* a closer means the parse missed (e.g. a multi-line string
    literal) — give up and keep the item rather than guess."""
    start = def_line
    while start > 0 and ATTR_RE.match(lines[start - 1]):
        start -= 1
    if lines[def_line].rstrip().endswith(";"):
        return start, def_line
    end = def_line
    while end < len(lines) - 1:
        end += 1
        line = lines[end]
        if not line or line[0] in " \t":
            continue
        r = line.rstrip()
        if not CLOSER_RE.match(r):
            return None  # next top-level item before any closer: parse miss
        if kind == "fn":
            if r == "}":
                return start, end
        elif r.endswith(";"):
            return start, end
        # A column-0 closer that isn't the terminator (`})` continuation of
        # a wrapped expression): keep scanning.
    return None


def find_def(lines, name):
    """(line index, kind) of `pub ... fn|static name` in this file."""
    fn_re = re.compile(
        r'^pub (?:unsafe )?(?:extern "C" )?fn ' + re.escape(name) + r"[(<]"
    )
    static_re = re.compile(r"^pub static (?:mut )?" + re.escape(name) + r":")
    for i, line in enumerate(lines):
        if fn_re.match(line):
            return i, "fn"
        if static_re.match(line):
            return i, "static"
    return None, None


def use_statements(lines):
    """[(start, end)] inclusive line spans of every `use …;` statement,
    including rustfmt-wrapped multi-line ones."""
    spans = []
    i = 0
    while i < len(lines):
        if USE_LINE_RE.match(lines[i]):
            j = i
            while j < len(lines) - 1 and not lines[j].rstrip().endswith(";"):
                j += 1
            spans.append((i, j))
            i = j + 1
        else:
            i += 1
    return spans


def use_mask(lines):
    mask = [False] * len(lines)
    for s, e in use_statements(lines):
        for i in range(s, e + 1):
            mask[i] = True
    return mask


IDENT_RE = re.compile(r"^(?:r#)?[A-Za-z_][A-Za-z0-9_]*$")
FLAT_USE_RE = re.compile(r"^((?:pub\s+)?use\s+[\w:#]+::)\{(.*)\};$")
SINGLE_USE_RE = re.compile(r"^(?:pub\s+)?use\s+[\w:#]+::((?:r#)?[A-Za-z0-9_]+);$")


def scrub_use_statements(text, dead, dry):
    """Remove `dead` names from every `use` statement. Returns (new_text,
    blocked): names sitting in a statement the surgery can't parse (nested
    groups, `as` aliases) come back in `blocked` and must not be deleted —
    leaving their import behind would break the build."""
    lines = text.split("\n")
    blocked = set()
    drop = set()
    replace = {}  # line -> replacement text (joined statement, single line)
    for s, e in use_statements(lines):
        stmt = " ".join(l.strip() for l in lines[s : e + 1])
        hit = {n for n in dead if word_re(n).search(stmt)}
        if not hit:
            continue
        m = FLAT_USE_RE.match(stmt)
        if m:
            names = [n.strip() for n in m.group(2).split(",") if n.strip()]
            if all(IDENT_RE.match(n) for n in names):
                kept = [n for n in names if n.removeprefix("r#") not in dead]
                drop.update(range(s, e + 1))
                if kept:
                    replace[s] = f"{m.group(1)}{{{', '.join(kept)}}};"
                continue
        m = SINGLE_USE_RE.match(stmt)
        if m:
            drop.update(range(s, e + 1))
            continue
        blocked.update(hit)
    if dry:
        return text, blocked
    out = []
    for i, line in enumerate(lines):
        if i in replace:
            out.append(replace[i])
        if i in drop:
            continue
        out.append(line)
    return "\n".join(out), blocked


def reference_index(texts):
    """token -> [(file, line)] over every line outside a `use` statement."""
    idx = defaultdict(list)
    for rel, text in texts.items():
        lines = text.split("\n")
        mask = use_mask(lines)
        for i, line in enumerate(lines):
            if mask[i]:
                continue
            for tok in set(TOKEN_RE.findall(line)):
                idx[tok].append((rel, i))
    return idx


def prune(texts, candidates, write):
    """Fixpoint-delete candidate items nothing references. Returns the set of
    deleted names."""
    deleted = set()
    alive = dict(candidates)  # name -> defining file
    while True:
        line_cache = {rel: text.split("\n") for rel, text in texts.items()}
        spans = {}  # name -> (rel, start, end)
        for name, rel in list(alive.items()):
            lines = line_cache.get(rel)
            if lines is None:
                del alive[name]
                continue
            di, kind = find_def(lines, name)
            if di is None:
                del alive[name]  # unparsed shape; keep the item
                continue
            span = item_span(lines, di, kind)
            if span is None:
                del alive[name]  # end not provable; keep the item
                continue
            s, e = span
            if any(l.strip() == "#[no_mangle]" for l in lines[s:di]):
                del alive[name]  # still exported; not ours to delete
                continue
            spans[name] = (rel, s, e)

        idx = reference_index(texts)
        dead = set()
        for name, (srel, s, e) in spans.items():
            if all(rel == srel and s <= i <= e for rel, i in idx.get(name, [])):
                dead.add(name)

        # Names imported by a statement the scrubber can't rewrite (nested
        # groups, aliases) must survive: deleting the item would leave a
        # dangling import.
        blocked = set()
        for rel, text in texts.items():
            _, b = scrub_use_statements(text, dead, dry=True)
            blocked |= b
        dead -= blocked
        for n in blocked:
            del alive[n]
        if not dead:
            break

        by_file = defaultdict(set)
        for n in dead:
            rel, s, e = spans[n]
            by_file[rel].update(range(s, e + 1))
        for rel, drop in by_file.items():
            lines = line_cache[rel]
            texts[rel] = "\n".join(l for i, l in enumerate(lines) if i not in drop)
        for rel in texts:
            if any(word_re(n).search(texts[rel]) for n in dead):
                texts[rel], _ = scrub_use_statements(texts[rel], dead, dry=False)
        for n in dead:
            deleted.add(n)
            del alive[n]
    if write:
        for rel, text in texts.items():
            path = ROOT / rel
            if path.read_text() != text:
                path.write_text(text)
    return deleted


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true", help="apply the rewrite")
    ap.add_argument(
        "--prune",
        action="store_true",
        help="delete unreferenced de-exported items (fixpoint)",
    )
    args = ap.parse_args()

    ledger = load_ledger()
    texts = {}
    for path in rust_files():
        texts[str(path.relative_to(ROOT))] = path.read_text()

    residue = extern_declared_names(texts) & ledger.keys()
    test = {n for n, e in ledger.items() if e["class"] == "test"}
    keep = residue | test
    internal = {n for n, e in ledger.items() if e["class"] == "internal"}

    print(f"ledger: {len(internal)} internal, {len(test)} test")
    print(f"residue (extern-declared in-crate): {len(residue)}")
    print(f"  of which internal-class: {len(residue & internal)}")
    print(f"strip set: {len(internal - keep)}")

    if args.prune:
        # Prune runs on an already-stripped tree. Candidates come from the
        # tree itself: every top-level extern "C" fn or pub static; the ones
        # still carrying #[no_mangle] (test + residue keeps) are filtered out
        # per-item inside prune. Plain `pub fn` (hand-written rewrites) is
        # deliberately not a candidate.
        cand_re = re.compile(
            r'^pub (?:(?:unsafe )?extern "C" fn|static(?: mut)?) ([A-Za-z0-9_]+)',
            re.M,
        )
        candidates = {}
        dupes = set()
        for rel, text in texts.items():
            if rel == "lib.rs" or rel.startswith("src/bin/"):
                continue
            for m in cand_re.finditer(text):
                name = m.group(1)
                if candidates.setdefault(name, rel) != rel:
                    dupes.add(name)  # ambiguous span; leave both copies alone
        for name in dupes:
            del candidates[name]
        deleted = prune(texts, candidates, args.write)
        print(f"pruned: {len(deleted)} items")
        by_file = defaultdict(int)
        for n in deleted:
            by_file[candidates[n]] += 1
        for f, c in sorted(by_file.items(), key=lambda kv: -kv[1])[:20]:
            print(f"  {c:4d} {f}")
        return

    stripped = strip_exports(texts, ledger, keep, args.write)
    total = sum(stripped.values())
    print(f"stripped: {total} #[no_mangle] attributes in {len(stripped)} files")
    if not args.write:
        for f, c in sorted(stripped.items(), key=lambda kv: -kv[1])[:15]:
            print(f"  {c:4d} {f}")


if __name__ == "__main__":
    main()
