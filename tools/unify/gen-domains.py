#!/usr/bin/env python3
"""Generate the unify --domains map: type name -> domain, where the domain is
the stem of the upstream v0.12.4 header (or source) that defined the type.

Usage:
    git archive v0.12.4 src | tar -x -C /tmp/upstream
    unify --repo . --verbose 2>&1 |
        sed -nE 's/^  ([A-Za-z_0-9]+) \[[0-9]+ copies\] -> .*/\1/p' |
        sort -u > /tmp/merged-names.txt
    gen-domains.py /tmp/upstream/src /tmp/merged-names.txt > domains.tsv

Names not found (system/libc/uv/lua types, derived anonymous names) fall back
to the tool's prefix rules; hand-mapped stragglers can be appended to the TSV.
"""

import re
import sys
from pathlib import Path

upstream = Path(sys.argv[1])
names = set(Path(sys.argv[2]).read_text().split())

# name -> (priority, domain); lower priority wins
found: dict[str, tuple[int, str]] = {}


def domain_of(path: Path) -> str:
    stem = path.stem
    if stem.endswith("_defs"):
        stem = stem[: -len("_defs")]
    if stem == "defs":
        # api/private/defs.h, msgpack_rpc/defs.h ...
        stem = path.parent.name
        if stem == "private":
            stem = "api"
    return stem


DEF_PATTERNS = [
    re.compile(r"^\s*(?:typedef\s+)?(?:struct|union|enum)\s+(\w+)\s*\{"),
    re.compile(r"^\}\s*(\w+)\s*;"),
    re.compile(r"^\s*typedef\s+(?:struct|union|enum)\s+\w+\s+(\w+)\s*;"),
    re.compile(r"^\s*typedef\s+[A-Za-z_][\w\s\*]*?(\w+)\s*;"),
    re.compile(r"^\s*typedef\s+.*\(\s*\*\s*(\w+)\s*\)\s*\("),  # fn-ptr typedef
]

for path in sorted(upstream.rglob("*")):
    if path.suffix not in (".h", ".c"):
        continue
    # priority: *_defs.h / defs.h < other .h < .c
    if path.suffix == ".h":
        prio = 0 if path.stem.endswith("_defs") or path.stem == "defs" else 1
    else:
        prio = 2
    dom = domain_of(path)
    try:
        text = path.read_text(errors="replace")
    except OSError:
        continue
    for line in text.splitlines():
        for pat in DEF_PATTERNS:
            m = pat.match(line)
            if m and m.group(1) in names:
                name = m.group(1)
                if name not in found or prio < found[name][0]:
                    found[name] = (prio, dom)

for name in sorted(names):
    if name in found:
        print(f"{name}\t{found[name][1]}")

print(f"mapped {len(found)}/{len(names)}", file=sys.stderr)
