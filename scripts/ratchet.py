#!/usr/bin/env python3
"""Ratchet the migration metrics: counts may hold or shrink, never grow.

The migration's promise is monotonic progress — every change leaves the tree
no less safe than it found it. This script is the mechanism. It measures, per
Rust source file (crates/*/src/**/*.rs plus the crate-root .rs files;
integration tests under crates/*/tests are not migration surface and stay
unmeasured, as they were when they lived at the repo root):

  unsafe      occurrences of "unsafe ", less the unsafe-fn *declarations*
              ("unsafe fn name", "unsafe extern \"C\" fn name") in files that
              carry #![deny(unsafe_op_in_unsafe_fn)]. Under that lint a
              function's real unsafe surface is spelled by the explicit
              unsafe blocks in its body, which stay counted; charging the
              declaration too would make adopting the lint *raise* this
              metric while lowering the surface, pitting it against
              files_without_deny_unsafe_op below. Declarations in files
              without the lint keep costing their token: there the body is
              implicitly unsafe, so the declaration is the only thing
              standing in for it.
  static_mut  occurrences of "static mut "
  no_mangle   occurrences of "#[unsafe(no_mangle)]"
  variadic    occurrences of ": ..." — C-variadic parameters, whose calls
              are format-string-unchecked. They retire as their callers
              migrate to the format_args!-based macros (semsg! and friends)
              or their modules are rewritten; vim_snprintf/vim_vsnprintf
              (vim's own user-visible format language) are expected to be
              the long-lived remainder.
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

Counting is plain substring matching. That over-counts (a comment saying
"unsafe " counts), but it is deterministic, matches how the migration plan's
baseline numbers were measured, and rustfmt (enforced by fmt-check) keeps the
spelling canonical. The point is monotonic pressure, not precision.

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
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "metrics" / "ratchet.json"
LEDGER = ROOT / "metrics" / "abi-ledger.jsonl"

LINE_CAP = 1000
COUNTED = {
    "unsafe": "unsafe ",
    "static_mut": "static mut ",
    "no_mangle": "#[unsafe(no_mangle)]",
    "variadic": ": ...",
}
FORBID = "#![forbid(unsafe_code)]"
DENY_UNSAFE_OP = "#![deny(unsafe_op_in_unsafe_fn)]"
# Unsafe-fn declaration forms, discounted from the "unsafe" metric in files
# denying unsafe_op_in_unsafe_fn. The trailing space is what separates a
# declaration (a name follows) from a function-pointer type ("fn(" follows),
# which is not a declaration and keeps costing its token.
UNSAFE_FN_DECLS = ('unsafe extern "C" fn ', "unsafe fn ")


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
        counts = {name: text.count(needle) for name, needle in COUNTED.items()}
        if DENY_UNSAFE_OP in text:
            counts["unsafe"] -= sum(text.count(decl) for decl in UNSAFE_FN_DECLS)
        counts["lines"] = len(text.splitlines())
        stats[str(path.relative_to(ROOT))] = counts
        without_forbid += FORBID not in text
        without_deny += FORBID not in text and DENY_UNSAFE_OP not in text
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
    for file in sorted(stats.keys() | base_files.keys()):
        cur = stats.get(file, {**dict.fromkeys(COUNTED, 0), "lines": 0})
        base = base_files.get(file, {})
        for name in COUNTED:
            if cur[name] > base.get(name, 0):
                found.append(f"{file}: {name} {base.get(name, 0)} -> {cur[name]}")
        limit = max(LINE_CAP, base.get("lines", 0))
        if cur["lines"] > limit:
            grandfathered = " (grandfathered)" if limit > LINE_CAP else ""
            found.append(f"{file}: {cur['lines']} lines > {limit}{grandfathered}")
    return found


def summary(stats, internal, without_forbid, without_deny):
    totals = {name: sum(c[name] for c in stats.values()) for name in COUNTED}
    over = sum(c["lines"] > LINE_CAP for c in stats.values())
    parts = [f"{n} {name}" for name, n in totals.items()]
    parts += [
        f"{over} files over {LINE_CAP} lines",
        f"{internal} internal exports",
        f"{without_forbid} files without forbid(unsafe_code)",
        f"{without_deny} files also without deny(unsafe_op_in_unsafe_fn)",
    ]
    return ", ".join(parts)


def main():
    args = set(sys.argv[1:])
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"ratchet: unknown argument(s): {' '.join(sorted(unknown))}")

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
