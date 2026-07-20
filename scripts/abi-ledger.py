#!/usr/bin/env python3
"""Classify every `#[no_mangle]` export by who actually needs the symbol.

The transpiled code exports ~6k symbols with C ABI, but only a fraction have
consumers that resolve them *by name*. The rest keep `#[no_mangle]
extern "C"` purely by inertia and are candidates for real Rust signatures in
later migration phases. This script writes the ledger that tells the two
apart (docs/abi-ledger.jsonl, one record per export, sorted by symbol —
e.g. `jq -r 'select(.class == "internal") | .file'` to list de-export
candidates per file), classifying every export as:

  external  referenced by the linked C archives or a dlopened library from
            the deps prefix. The linker/loader needs the name.
  test      referenced by test/unit — either FFI'd from the specs' Lua
            (ffi.C.<name> against the running nvim, which links with
            --export-dynamic) or an undefined symbol of unit-fixtures.so.
            The name must stay exported while the spec/fixture exists, but
            the consumer is in-repo, so it can be migrated together with a
            signature change.
  internal  nobody resolves the name. Only in-crate callers, which the
            compiler resolves regardless of mangling. Safe to de-export.

Spec references are found by intersecting export names with every
identifier-shaped token in test/unit/**/*.lua. That over-approximates (a
symbol named in a comment counts), which errs in the safe direction: a
symbol is only ever misclassified toward "keep exported", never toward
"safe to change". Dynamically constructed names ('os_' .. name) would be
missed, but the suite has none. NB: user Lua could in principle ffi.C into
any export (--export-dynamic exposes them all); the project makes no compat
guarantees, so that surface is deliberately not part of the contract.

Usage: abi-ledger.py [--check]
  --check   regenerate and diff against the committed ledger; exit 1 on
            drift instead of writing.

Needs the flake dev shell ($NVIM_DEPS_PREFIX, cc, nm) and builds
unit-fixtures.so (first run reconstructs the upstream headers, see
prep-unit-headers.sh).
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "docs" / "abi-ledger.jsonl"
FIXTURE_SO = ROOT / "target" / "bin" / "unit-fixtures.so"

EXPORT_RE = re.compile(r'^pub (unsafe extern "C" fn|static mut) ([A-Za-z0-9_]+)')
TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def collect_exports():
    """name -> (kind, repo-relative file) for every #[no_mangle] item."""
    exports = {}
    for path in sorted(ROOT.glob("src/**/*.rs")):
        lines = path.read_text().splitlines()
        for i, line in enumerate(lines):
            if line.strip() != "#[no_mangle]":
                continue
            decl = lines[i + 1] if i + 1 < len(lines) else ""
            m = EXPORT_RE.match(decl)
            if not m:
                sys.exit(
                    f"abi-ledger: {path}:{i + 2}: unrecognized declaration "
                    f"after #[no_mangle]: {decl!r}\n"
                    "(teach abi-ledger.py the new shape)"
                )
            name = m.group(2)
            kind = "fn" if m.group(1).endswith("fn") else "static"
            rel = str(path.relative_to(ROOT))
            if name in exports:
                sys.exit(
                    f"abi-ledger: duplicate export {name}: "
                    f"{exports[name][1]} and {rel}"
                )
            exports[name] = (kind, rel)
    return exports


def undefined_symbols(lib: Path):
    """The names `lib` needs someone else to provide, versions stripped."""
    cmd = ["nm", "--undefined-only"]
    if lib.suffix == ".so":
        cmd.append("--dynamic")
    out = subprocess.run(
        cmd + [str(lib)], check=True, capture_output=True, text=True
    ).stdout
    syms = set()
    for line in out.splitlines():
        line = line.strip()
        if not line or line.endswith(":"):  # blank or "member.o:" headers
            continue
        syms.add(line.split()[-1].split("@")[0])
    return syms


def deps_libraries():
    """Everything from the deps prefix that links or dlopens against nvim:
    the static archives build.rs links, the bundled tree-sitter parsers, and
    any Lua C modules."""
    prefix = os.environ.get("NVIM_DEPS_PREFIX")
    if not prefix:
        sys.exit(
            "abi-ledger: NVIM_DEPS_PREFIX must be set "
            "(enter the flake dev shell)"
        )
    prefix = Path(prefix)
    libs = [
        *prefix.glob("lib/*.a"),
        *prefix.glob("lib64/*.a"),
        *prefix.glob("lib/nvim/parser/*.so"),
        *prefix.glob("lib/lua/**/*.so"),
    ]
    if not libs:
        sys.exit(f"abi-ledger: no libraries found under {prefix}")
    return sorted(libs)


def spec_tokens():
    tokens = set()
    for spec in ROOT.glob("test/unit/**/*.lua"):
        tokens.update(TOKEN_RE.findall(spec.read_text()))
    return tokens


def build_ledger():
    exports = collect_exports()

    # refs: name -> sorted consumer labels (drives the classification below)
    refs = {name: [] for name in exports}
    for lib in deps_libraries():
        for name in undefined_symbols(lib) & exports.keys():
            refs[name].append(lib.name)

    subprocess.run(
        [ROOT / "scripts" / "build-unit-fixtures.sh", FIXTURE_SO], check=True
    )
    for name in undefined_symbols(FIXTURE_SO) & exports.keys():
        refs[name].append("unit-fixtures.so")

    for name in spec_tokens() & exports.keys():
        refs[name].append("unit-specs")

    counts = {cls: 0 for cls in ("external", "test", "internal")}
    lines = []
    for name in sorted(exports):
        kind, file = exports[name]
        r = refs[name]
        if any(not ref.startswith("unit-") for ref in r):
            cls = "external"
        elif r:
            cls = "test"
        else:
            cls = "internal"
        counts[cls] += 1
        record = {"symbol": name, "kind": kind, "file": file, "class": cls, "refs": r}
        lines.append(json.dumps(record, separators=(",", ":")))
    return "\n".join(lines) + "\n", counts


def main():
    check = "--check" in sys.argv[1:]
    content, counts = build_ledger()
    if check:
        committed = LEDGER.read_text() if LEDGER.exists() else ""
        if committed != content:
            sys.exit(
                f"abi-ledger: {LEDGER.relative_to(ROOT)} is stale; "
                "run `just abi-ledger` and commit the result"
            )
        return
    LEDGER.parent.mkdir(exist_ok=True)
    LEDGER.write_text(content)
    total = sum(counts.values())
    print(f"wrote {LEDGER.relative_to(ROOT)}: {total} exports", end=" (")
    print(", ".join(f"{n} {cls}" for cls, n in counts.items()) + ")")


if __name__ == "__main__":
    main()
