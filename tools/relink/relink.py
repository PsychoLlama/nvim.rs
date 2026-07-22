#!/usr/bin/env python3
"""relink — the phase-5b rewriter.

Replaces per-module `extern "C"` re-declarations with compiler-checked
resolution:

- Symbols defined in-crate (per metrics/abi-ledger.jsonl) lose their local
  declaration and gain `use crate::<defining module>::<name>;`. The item
  stays `extern "C"`/`#[no_mangle]`, so ABI and codegen are unchanged —
  only *resolution* moves from the linker to the compiler, which now
  verifies every signature the linker used to take on faith.
- External symbols (not in the ledger) from libc/system, LuaJIT/luv/lpeg,
  and libuv are consolidated into one shared declaration each under
  src/nvim/os/libc.rs, src/nvim/lua/ffi.rs, and src/nvim/event/libuv.rs.
- Everything else stays put: single-library externals (tree-sitter,
  unibilium, utf8proc), function-local extern blocks (`#[link_name]`
  shims), extern `type` items, and any declaration whose signature — on
  either the declaring or the defining side — mentions a module-local
  type (the phase-5a blacklist residue: terminal, quickfix, regexp, tui,
  and the singleton-anon leftovers). Those keep linker resolution until
  phase 8 rewrites their modules — a nominal decl/def difference is not
  necessarily a *structural* clash, so they are left unannotated and any
  surviving clashing-declaration warnings are triaged by hand.

Dry-run by default; `--write` applies. The shared FFI modules must be
registered in lib.rs by hand (the tool prints what it expects).
"""

import argparse
import json
import os
import re
import sys
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

RUST_KEYWORDS = {
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "try",
    "typeof",
    "unsized",
    "virtual",
    "yield",
}

SHARED_HOMES = {
    "system": ("src/nvim/os/libc.rs", "crate::src::nvim::os::libc"),
    "lua": ("src/nvim/lua/ffi.rs", "crate::src::nvim::lua::ffi"),
    "uv": ("src/nvim/event/libuv.rs", "crate::src::nvim::event::libuv"),
}

SHARED_HEADERS = {
    "system": [
        '//! Shared libc/system `extern "C"` declarations (phase 5b).',
        "//!",
        "//! One declaration per symbol, `use`d by every consumer, instead of",
        "//! the per-module copies c2rust emitted. Everything here resolves",
        "//! against the platform C library at link time.",
    ],
    "lua": [
        '//! Shared LuaJIT/luv/lpeg `extern "C"` declarations (phase 5b).',
        "//!",
        "//! One declaration per symbol, `use`d by every consumer, instead of",
        "//! the per-module copies c2rust emitted. Everything here resolves",
        "//! against the static LuaJIT/luv/lpeg libraries at link time.",
    ],
    "uv": [
        '//! Shared libuv `extern "C"` declarations (phase 5b).',
        "//!",
        "//! One declaration per symbol, `use`d by every consumer, instead of",
        "//! the per-module copies c2rust emitted. Everything here resolves",
        "//! against the static libuv at link time.",
    ],
}


def rust_ident(name):
    return f"r#{name}" if name in RUST_KEYWORDS else name


def parse_mod_tree_braced(lib_path):
    """Brace-aware lib.rs walk: maps module files to crate paths."""
    file_to_mod = {}
    stack = []  # (ident, dirpath)
    pending_path = None
    with open(lib_path) as f:
        lines = f.readlines()
    for line in lines:
        stripped = line.strip()
        m = re.match(r'#\[path = "([^"]+)"\]', stripped)
        if m:
            pending_path = m.group(1)
            continue
        m = re.match(r"pub mod (r#)?([A-Za-z_][A-Za-z0-9_]*)(;| \{)", stripped)
        if m:
            raw, name, tail = m.group(1) or "", m.group(2), m.group(3)
            ident = raw + name
            if tail == " {":
                parent = stack[-1][1] if stack else ROOT
                stack.append((ident, os.path.join(parent, name)))
            else:
                parent = stack[-1][1] if stack else ROOT
                if pending_path:
                    fpath = os.path.join(parent, pending_path)
                elif os.path.exists(os.path.join(parent, name + ".rs")):
                    fpath = os.path.join(parent, name + ".rs")
                else:
                    fpath = os.path.join(parent, name, "mod.rs")
                mods = [s[0] for s in stack] + [ident]
                file_to_mod[os.path.relpath(fpath, ROOT)] = "crate::" + "::".join(mods)
            pending_path = None
            continue
        if stripped.startswith("}") and stack:
            stack.pop()
    return file_to_mod


class ExternItem:
    __slots__ = ("start", "end", "body", "kind", "name")

    def __init__(self, start, end, body, kind, name):
        self.start = start  # 0-based line, inclusive
        self.end = end  # 0-based line, inclusive
        self.body = body
        self.kind = kind
        self.name = name


ITEM_RE = re.compile(
    r"^\s*(?:pub\s+)?(fn|static|type)\s+(?:mut\s+)?(?:r#)?([A-Za-z_][A-Za-z0-9_]*)"
)


def parse_extern_blocks(lines):
    """Yield (block_start, block_end, [ExternItem]) for column-0 blocks."""
    blocks = []
    i = 0
    while i < len(lines):
        if lines[i].startswith('extern "C" {'):
            start = i
            depth = lines[i].count("{") - lines[i].count("}")
            i += 1
            items = []
            cur_start = i
            while i < len(lines) and depth > 0:
                depth += lines[i].count("{") - lines[i].count("}")
                if depth == 0:
                    break
                if depth == 1 and lines[i].rstrip().endswith(";"):
                    body = "\n".join(lines[cur_start : i + 1])
                    sig = None
                    for l in body.split("\n"):
                        ls = l.strip()
                        if ls.startswith("#[") or ls.startswith("//") or not ls:
                            continue
                        sig = ls
                        break
                    m = ITEM_RE.match(sig) if sig else None
                    if not m:
                        raise RuntimeError(
                            f"unparsed extern item at line {cur_start + 1}: {body[:80]}"
                        )
                    items.append(ExternItem(cur_start, i, body, m.group(1), m.group(2)))
                    cur_start = i + 1
                i += 1
            blocks.append((start, i, items))
        i += 1
    return blocks


TYPE_DEF_RE = re.compile(r"^pub (?:struct|union|enum) ([A-Za-z_][A-Za-z0-9_]*)")
TYPE_ALIAS_RE = re.compile(r"^pub type ([A-Za-z_][A-Za-z0-9_]*)\s*=")


def local_type_names(rel, text, blocks):
    """Type names *defined* (not re-exported) in this file."""
    names = set()
    for line in text.split("\n"):
        m = TYPE_DEF_RE.match(line) or TYPE_ALIAS_RE.match(line)
        if m:
            names.add(m.group(1))
    for _, _, items in blocks:
        for it in items:
            if it.kind == "type":
                names.add(it.name)
    return names


FN_DEF_RE = re.compile(
    r"^pub (?:unsafe )?(?:extern \"C\" )?fn ([A-Za-z_][A-Za-z0-9_]*)", re.M
)
STATIC_DEF_RE = re.compile(r"^pub static (?:mut )?([A-Za-z_][A-Za-z0-9_]*):", re.M)


def def_signatures(text):
    """Map name -> definition signature text (fn: to body `{`; static: to `=`)."""
    sigs = {}
    for m in FN_DEF_RE.finditer(text):
        name = m.group(1)
        i = m.end()
        depth = 0
        while i < len(text):
            c = text[i]
            if c == "(":
                depth += 1
            elif c == ")":
                depth -= 1
            elif c == "{" and depth == 0:
                break
            elif c == ";" and depth == 0:
                break
            i += 1
        sigs[name] = text[m.start() : i]
    for m in STATIC_DEF_RE.finditer(text):
        name = m.group(1)
        eq = text.find("=", m.end())
        nl = text.find("\n", m.end())
        end = eq if eq != -1 and (nl == -1 or eq < nl + 200) else nl
        sigs[name] = text[m.start() : end if end != -1 else m.end()]
    return sigs


WORD_RE_CACHE = {}


def mentions(text, name):
    r = WORD_RE_CACHE.get(name)
    if r is None:
        r = re.compile(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])")
        WORD_RE_CACHE[name] = r
    return r.search(text) is not None


def lib_of(name):
    if re.match(r"^(lua_|luaL_|luaJIT_|luaopen_|luv_)", name):
        return "lua"
    if name.startswith("uv_"):
        return "uv"
    if name.startswith("ts_"):
        return "treesitter"
    if name.startswith("utf8proc_"):
        return "utf8proc"
    if name.startswith("unibi_"):
        return "unibilium"
    return "system"


ATTR_LINE_RE = re.compile(r"^\s*#\[")


def strip_body(body):
    """Normalized signature for variant comparison."""
    b = re.sub(r"#\[[^\]]*\]\s*", "", body)
    return re.sub(r"\s+", " ", b).strip()


def pubify(body):
    """Make an extern item declaration `pub` for the shared modules."""
    lines = body.split("\n")
    out = []
    done = False
    for l in lines:
        ls = l.strip()
        if not done and not ATTR_LINE_RE.match(l) and ls and not ls.startswith("//"):
            indent = l[: len(l) - len(l.lstrip())]
            rest = l.lstrip()
            if not rest.startswith("pub "):
                rest = "pub " + rest
            out.append(indent + rest)
            done = True
        else:
            out.append(l)
    return "\n".join(out)


def reindent(body, target="    "):
    lines = body.split("\n")
    base = min((len(l) - len(l.lstrip()) for l in lines if l.strip()), default=0)
    return "\n".join(target + l[base:] if l.strip() else l for l in lines)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true", help="apply the rewrite")
    ap.add_argument(
        "--keep",
        action="append",
        default=[],
        help="symbol to leave declared in place (override, repeatable)",
    )
    args = ap.parse_args()

    ledger = {}
    with open(os.path.join(ROOT, "metrics/abi-ledger.jsonl")) as f:
        for line in f:
            e = json.loads(line)
            ledger[e["symbol"]] = e

    file_to_mod = parse_mod_tree_braced(os.path.join(ROOT, "lib.rs"))

    rs_files = []
    for dirpath, dirs, files in os.walk(os.path.join(ROOT, "src")):
        for fn in files:
            if fn.endswith(".rs"):
                rs_files.append(os.path.join(dirpath, fn))

    texts = {}
    blocks_by_file = {}
    local_types = {}
    for path in sorted(rs_files):
        rel = os.path.relpath(path, ROOT)
        with open(path) as f:
            text = f.read()
        texts[rel] = text
        lines = text.split("\n")
        blocks = parse_extern_blocks(lines)
        if blocks:
            blocks_by_file[rel] = blocks
        if not rel.startswith("src/nvim/types/"):
            local_types[rel] = local_type_names(rel, text, blocks)
        else:
            local_types[rel] = set()

    def_sigs = {}  # def file -> {name: sig}

    def def_sig(sym):
        e = ledger.get(sym)
        if not e:
            return None
        f = e["file"]
        if f not in def_sigs:
            def_sigs[f] = def_signatures(texts.get(f, ""))
        return def_sigs[f].get(sym)

    stats = defaultdict(int)
    kept_detail = defaultdict(list)
    shared = {"system": {}, "lua": {}, "uv": {}}  # lib -> name -> body variants
    shared_votes = {
        "system": defaultdict(lambda: defaultdict(int)),
        "lua": defaultdict(lambda: defaultdict(int)),
        "uv": defaultdict(lambda: defaultdict(int)),
    }
    plans = {}  # rel -> dict(delete=set(lines), imports={modpath: set(names)},
    #             annotate=set(item start lines), blocks=blocks)

    for rel, blocks in sorted(blocks_by_file.items()):
        if rel.startswith("src/nvim/types/"):
            continue  # canonical extern types live here; not our business
        text = texts[rel]
        lines = text.split("\n")
        # File text minus all column-0 extern blocks, for usage checks.
        block_lines = set()
        for s, e, _ in blocks:
            block_lines.update(range(s, e + 1))
        rest_text = "\n".join(l for i, l in enumerate(lines) if i not in block_lines)

        plan = {"delete": set(), "imports": defaultdict(set), "annotate": set()}
        my_locals = local_types.get(rel, set())

        for bstart, bend, items in blocks:
            for it in items:
                if it.kind == "type":
                    stats["kept type"] += 1
                    continue
                entry = ledger.get(it.name)
                if it.name in args.keep:
                    stats["kept override"] += 1
                    continue
                if entry:
                    # In-crate: candidate for a real import.
                    deff = entry["file"]
                    if any(mentions(it.body, t) for t in my_locals):
                        stats["kept local-type decl"] += 1
                        kept_detail["decl-side"].append((rel, it.name))
                        plan["annotate"].add(it.start)
                        continue
                    dsig = def_sig(it.name)
                    if dsig is None:
                        stats["kept no-def-sig"] += 1
                        kept_detail["no-def-sig"].append((rel, it.name))
                        continue
                    if any(mentions(dsig, t) for t in local_types.get(deff, set())):
                        stats["kept local-type def"] += 1
                        kept_detail["def-side"].append((rel, it.name, deff))
                        plan["annotate"].add(it.start)
                        continue
                    modpath = file_to_mod.get(deff)
                    if not modpath:
                        raise RuntimeError(f"no module path for {deff}")
                    plan["delete"].update(range(it.start, it.end + 1))
                    if mentions(rest_text, it.name):
                        plan["imports"][modpath].add(it.name)
                        stats["imported"] += 1
                    else:
                        stats["deleted unused"] += 1
                else:
                    lib = lib_of(it.name)
                    if lib not in SHARED_HOMES:
                        stats[f"kept external {lib}"] += 1
                        continue
                    if "link_name" in it.body:
                        stats["kept link_name"] += 1
                        continue
                    if any(mentions(it.body, t) for t in my_locals):
                        stats["kept external local-type"] += 1
                        kept_detail["ext-local"].append((rel, it.name))
                        continue
                    plan["delete"].update(range(it.start, it.end + 1))
                    norm = strip_body(it.body)
                    shared_votes[lib][it.name][norm] += 1
                    shared[lib].setdefault(it.name, {})[norm] = it.body
                    if mentions(rest_text, it.name):
                        plan["imports"][SHARED_HOMES[lib][1]].add(it.name)
                        stats["imported external"] += 1
                    else:
                        stats["deleted unused external"] += 1
        plans[rel] = (plan, blocks)

    print("== plan ==")
    for k in sorted(stats):
        print(f"  {stats[k]:6d} {k}")
    for cls, entries in kept_detail.items():
        print(f"\n  kept ({cls}): {len(entries)}")
        by_file = defaultdict(int)
        for e in entries:
            by_file[e[0]] += 1
        for f, c in sorted(by_file.items(), key=lambda kv: -kv[1])[:12]:
            print(f"    {c:4d} {f}")

    # Shared-module contents.
    for lib, (home, modpath) in SHARED_HOMES.items():
        n = len(shared[lib])
        multi = sum(1 for v in shared[lib].values() if len(v) > 1)
        print(f"\n  shared {home}: {n} symbols ({multi} with signature variants)")

    if not args.write:
        return

    # Emit shared modules first.
    for lib, (home, modpath) in SHARED_HOMES.items():
        if not shared[lib]:
            continue
        out = list(SHARED_HEADERS[lib])
        out.append("")
        out.append("use crate::src::nvim::types::*;")
        out.append("")
        out.append('extern "C" {')
        for name in sorted(shared[lib]):
            variants = shared[lib][name]
            votes = shared_votes[lib][name]
            norm = max(votes, key=lambda k: (votes[k], k))
            body = variants[norm]
            out.append(reindent(pubify(body)))
        out.append("}")
        with open(os.path.join(ROOT, home), "w") as f:
            f.write("\n".join(out) + "\n")
        print(f"wrote {home}")

    # Rewrite modules.
    for rel, (plan, blocks) in sorted(plans.items()):
        if not plan["delete"] and not plan["imports"] and not plan["annotate"]:
            continue
        lines = texts[rel].split("\n")
        delete = set(plan["delete"])
        # Drop blocks that lost every item (header/brace/stray lines too).
        first_block_start = None
        for s, e, items in blocks:
            if first_block_start is None:
                first_block_start = s
            item_lines = set()
            for it in items:
                item_lines.update(range(it.start, it.end + 1))
            if items and item_lines <= delete:
                delete.update(range(s, e + 1))
        out = []
        for i, l in enumerate(lines):
            if i == first_block_start and plan["imports"]:
                for modpath in sorted(plan["imports"]):
                    names = sorted(plan["imports"][modpath], key=str.lower)
                    names = [rust_ident(n) for n in names]
                    out.append(f"use {modpath}::{{{', '.join(names)}}};")
            if i in delete:
                continue
            out.append(l)
        with open(os.path.join(ROOT, rel), "w") as f:
            f.write("\n".join(out))
    print("rewrote modules")
    print("\nRegister the shared modules in lib.rs:")
    print("  os   -> pub mod libc;")
    print("  lua  -> pub mod ffi;")
    print("  event-> pub mod libuv;")


if __name__ == "__main__":
    main()
