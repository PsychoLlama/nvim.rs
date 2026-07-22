#!/usr/bin/env python3
"""Validate the ffigen-generated unit cdefs against the upstream C headers.

Compiles two probe programs from tools/ffigen's manifest:
  A: #includes the upstream (v0.12.4, target/upstream) headers the unit specs
     import, with the same defines the old preprocess pipeline used, and
     prints size/align/offset for every generated type and the value of every
     generated constant.
  B: #includes the generated target/ffi/unit-cdefs.h standalone and prints
     the same probes.
Then diffs A against B (layouts) and A against the manifest (constants), and
compiles the generated prototypes/extern statics in one translation unit with
the upstream headers so the C compiler cross-checks every signature
("conflicting types for ..." = a real ABI mismatch).

Names the upstream headers do not declare (module-private types, unified
anonymous-type names, de-macroized constants) are dropped iteratively on
compile errors and reported as "unprobed" -- they are unvalidatable against
the tag, not failures.

Usage: scripts/check-unit-cdefs.py [--keep-going]
Exit 0 iff no layout/constant/prototype mismatch.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UP = ROOT / "target" / "upstream"
FFI = ROOT / "target" / "ffi"
WORK = FFI / "check"

DEFINES = [
    # mirror test/unit/preprocess.lua Gcc:init_defines, minus the
    # attribute-stripping: probes measure the real C ABI, so attributes stay.
    "-DEXTERN=extern",
    "-DINIT(...)=",
    "-D_GNU_SOURCE",
    "-DUNIT_TESTING",
    "-DUNIT_TESTING_LUA_PREPROCESSING",
]


def include_flags():
    deps = os.environ.get("NVIM_DEPS_PREFIX")
    if not deps:
        sys.exit("NVIM_DEPS_PREFIX must be set (enter the flake dev shell)")
    return [
        f"-I{deps}/include/luajit-2.1",
        f"-I{deps}/include",
        f"-I{UP}/build/src/nvim/auto",
        f"-I{UP}/build/include",
        f"-I{UP}/build/cmake.config",
        f"-I{UP}/src/src",
        f"-I{UP}/src",
    ]


def spec_headers():
    """The ./src/... headers the unit specs (and unit testutil) cimport."""
    pat = re.compile(r"""['"](\./src/[^'"]+\.h)['"]""")
    headers = []
    seen = set()
    for lua in sorted((ROOT / "test" / "unit").rglob("*.lua")):
        for m in pat.finditer(lua.read_text()):
            h = m.group(1)
            if h not in seen:
                seen.add(h)
                headers.append(h)
    return headers


def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def compile_and_run(src: Path, exe: Path, extra_flags):
    """Compile with iterative dropping of probe lines upstream can't satisfy.

    Returns (output lines, dropped probe ids)."""
    dropped = {}
    lines = src.read_text().splitlines()
    for _ in range(60):
        cur = src.with_suffix(".cur.c")
        cur.write_text("\n".join(lines) + "\n")
        r = run(["cc", "-std=c11", "-O0", "-o", str(exe), str(cur)] + extra_flags)
        if r.returncode == 0:
            out = run([str(exe)])
            if out.returncode != 0:
                sys.exit(f"probe {exe} crashed:\n{out.stderr}")
            return out.stdout.splitlines(), dropped
        # drop the probe lines the errors point at
        bad = set()
        for m in re.finditer(r"\.cur\.c:(\d+):", r.stderr):
            lineno = int(m.group(1)) - 1
            if 0 <= lineno < len(lines) and "/*PROBE " in lines[lineno]:
                bad.add(lineno)
        if not bad:
            sys.exit(
                f"probe compile failed with non-probe errors:\n"
                + "\n".join(r.stderr.splitlines()[:40])
            )
        for lineno in bad:
            probe_id = lines[lineno].split("/*PROBE ")[1].split("*/")[0]
            dropped[probe_id] = True
            lines[lineno] = f"/* dropped {probe_id} */"
    sys.exit("probe compile did not converge")


PRELUDE = """
#include <stdio.h>
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>
"""

# The generated chunk assumes only LuaJIT's built-in types; glibc headers
# must stay out (the chunk defines its own copies of e.g. struct _IO_FILE,
# and stdio macros like BUFSIZ would mangle same-named generated constants).
PRELUDE_GEN = """
#define offsetof(t, f) __builtin_offsetof(t, f)
typedef __SIZE_TYPE__ size_t;
typedef long ssize_t;
typedef __PTRDIFF_TYPE__ ptrdiff_t;
typedef long intptr_t;
typedef unsigned long uintptr_t;
typedef __INT8_TYPE__ int8_t;
typedef __INT16_TYPE__ int16_t;
typedef __INT32_TYPE__ int32_t;
typedef __INT64_TYPE__ int64_t;
typedef __UINT8_TYPE__ uint8_t;
typedef __UINT16_TYPE__ uint16_t;
typedef __UINT32_TYPE__ uint32_t;
typedef __UINT64_TYPE__ uint64_t;
typedef __WCHAR_TYPE__ wchar_t;
typedef __builtin_va_list va_list;
#define bool _Bool
#define true 1
#define false 0
int printf(const char *, ...);
/* Bodies of the tools/ffigen/deny.txt tags: at test time the harness's
   system preamble defines them; this probe is freestanding, so inline the
   (stable) glibc x86-64 layouts. */
struct iovec { void *iov_base; size_t iov_len; };
struct __pthread_internal_list {
  struct __pthread_internal_list *__prev;
  struct __pthread_internal_list *__next;
};
struct __pthread_mutex_s {
  int __lock; unsigned int __count; int __owner; unsigned int __nusers;
  int __kind; short __spins; short __elision;
  struct __pthread_internal_list __list;
};
struct __pthread_rwlock_arch_t {
  unsigned int __readers; unsigned int __writers;
  unsigned int __wrphase_futex; unsigned int __writers_futex;
  unsigned int __pad3; unsigned int __pad4; int __cur_writer; int __shared;
  signed char __rwelision; unsigned char __pad1[7]; unsigned long __pad2;
  unsigned int __flags;
};
"""


def probe_body(manifest):
    out = []
    # nvim's main.h declares main(int, char**); match it to coexist in the TU
    out.append("int main(int argc, char **argv) {")
    out.append("(void)argc; (void)argv;")
    for tname, tinfo in manifest["types"].items():
        # both the tag and the typedef name exist in the generated chunk;
        # upstream may only have the tag -- such names fall out through the
        # drop loop and are reported as unprobed.
        out.append(
            f'printf("TYPE {tname} %zu %zu\\n", sizeof({tname}), _Alignof({tname})); /*PROBE type:{tname}*/'
        )
        for f in tinfo["fields"]:
            out.append(
                f'printf("FIELD {tname}.{f} %zu\\n", offsetof({tname}, {f})); /*PROBE field:{tname}.{f}*/'
            )
    for cname, val in manifest["consts"].items():
        out.append(
            f'printf("CONST {cname} %lld\\n", (long long)({cname})); /*PROBE const:{cname}*/'
        )
    out.append("return 0;")
    out.append("}")
    return "\n".join(out)


def main():
    keep_going = "--keep-going" in sys.argv
    subprocess.run([str(ROOT / "scripts" / "prep-unit-headers.sh")], check=True)
    manifest = json.loads((FFI / "manifest.json").read_text())
    WORK.mkdir(parents=True, exist_ok=True)

    body = probe_body(manifest)

    # ---- probe A: upstream headers
    includes = "\n".join(f'#include "{h}"' for h in spec_headers())
    a_src = WORK / "probe_upstream.c"
    a_src.write_text(PRELUDE + includes + "\n" + body)
    print(
        f"check-unit-cdefs: compiling upstream probe ({len(manifest['types'])} types, "
        f"{len(manifest['consts'])} consts) ..."
    )
    a_out, a_dropped = compile_and_run(
        a_src, WORK / "probe_upstream", include_flags() + DEFINES
    )

    # ---- probe B: the generated chunk
    b_src = WORK / "probe_gen.c"
    b_src.write_text(PRELUDE_GEN + f'#include "{FFI}/unit-cdefs.h"\n' + body)
    print("check-unit-cdefs: compiling generated-chunk probe ...")
    b_out, b_dropped = compile_and_run(b_src, WORK / "probe_gen", [])
    if b_dropped:
        print(
            f"WARNING: generated chunk failed to compile {len(b_dropped)} of its own probes:"
        )
        for p in sorted(b_dropped):
            print(f"  gen-probe dropped: {p}")

    def to_map(lines):
        m = {}
        for line in lines:
            parts = line.split(" ")
            m[(parts[0], parts[1])] = parts[2:]
        return m

    a_map = to_map(a_out)
    b_map = to_map(b_out)

    mismatches = []
    unprobed = sorted(a_dropped)

    for key, aval in sorted(a_map.items()):
        kind, name = key
        if kind == "CONST":
            want = manifest["consts"].get(name)
            got = int(aval[0])
            # generated consts are emitted as 32-bit static const; compare
            # both raw and unsigned-wrapped so -1 == 4294967295u.
            if (
                want is not None
                and got != want
                and (got & 0xFFFFFFFF) != (want & 0xFFFFFFFF)
            ):
                mismatches.append(f"CONST {name}: upstream {got}, generated {want}")
        else:
            bval = b_map.get(key)
            if bval is None:
                unprobed.append(f"{kind.lower()}:{name} (gen probe missing)")
            elif bval != aval:
                mismatches.append(f"{kind} {name}: upstream {aval}, generated {bval}")

    # ---- prototype/static cross-check: one TU, compiler verifies signatures
    protos = (
        "\n".join(manifest["protos"].values())
        + "\n"
        + "\n".join(manifest["statics"].values())
    )
    p_src = WORK / "probe_protos.c"
    p_src.write_text(PRELUDE + includes + "\n" + protos + "\n")
    print(f"check-unit-cdefs: cross-checking {len(manifest['protos'])} prototypes ...")
    lines = p_src.read_text().splitlines()
    proto_conflicts = []
    proto_unknown = []
    for _ in range(60):
        cur = p_src.with_suffix(".cur.c")
        cur.write_text("\n".join(lines) + "\n")
        r = run(
            ["cc", "-std=c11", "-fsyntax-only", str(cur)] + include_flags() + DEFINES
        )
        if r.returncode == 0:
            break
        bad = set()
        for m in re.finditer(r"\.cur\.c:(\d+):(?:\d+:)? error: (.*)", r.stderr):
            lineno = int(m.group(1)) - 1
            msg = m.group(2)
            if lineno >= len(lines):
                continue
            decl = lines[lineno].strip()
            if "conflicting types" in msg or "redeclared" in msg:
                proto_conflicts.append(f"{decl}  <-- {msg}")
            else:
                proto_unknown.append(f"{decl}  <-- {msg}")
            bad.add(lineno)
        if not bad:
            sys.exit(
                "prototype TU failed with unattributable errors:\n"
                + "\n".join(r.stderr.splitlines()[:40])
            )
        for lineno in bad:
            lines[lineno] = ""

    # GlobalCell wrapping erased C constness, so a conflicted static gets one
    # retry with const injected: if that version coexists with the upstream
    # declaration, the difference was qualifier-only (no ABI consequence).
    static_decls = set(manifest["statics"].values())
    qualifier_only = []
    real_conflicts = []
    for entry in proto_conflicts:
        decl = entry.split("  <-- ")[0]
        if decl in static_decls:
            konst = decl.replace("extern ", "extern const ", 1).replace("*", "*const ")
            probe = WORK / "probe_static.c"
            probe.write_text(PRELUDE + includes + "\n" + konst + "\n")
            r = run(
                ["cc", "-std=c11", "-fsyntax-only", str(probe)]
                + include_flags()
                + DEFINES
            )
            if r.returncode == 0:
                qualifier_only.append(decl)
                continue
        real_conflicts.append(entry)
    mismatches.extend(f"PROTO {c}" for c in real_conflicts)

    # ---- report
    print()
    print(
        f"check-unit-cdefs: {len(a_map)} probes compared, "
        f"{len(unprobed)} unprobed (not declared upstream), "
        f"{len(proto_unknown)} prototypes skipped (types not declared upstream), "
        f"{len(mismatches)} mismatches"
    )
    if qualifier_only:
        print(
            f"  {len(qualifier_only)} statics differ from upstream only in constness "
            f"(GlobalCell erased the qualifier): accepted"
        )
    if unprobed:
        (WORK / "unprobed.txt").write_text("\n".join(map(str, unprobed)) + "\n")
        print(f"  unprobed list: {WORK}/unprobed.txt")
    if proto_unknown:
        (WORK / "protos-skipped.txt").write_text("\n".join(proto_unknown) + "\n")
        print(f"  skipped prototypes: {WORK}/protos-skipped.txt")
    if mismatches:
        print()
        for m_ in mismatches[:200]:
            print(f"MISMATCH {m_}")
        (WORK / "mismatches.txt").write_text("\n".join(mismatches) + "\n")
        if not keep_going:
            sys.exit(1)
    else:
        print("check-unit-cdefs: OK")


if __name__ == "__main__":
    main()
