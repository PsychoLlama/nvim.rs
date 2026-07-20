#!/usr/bin/env python3
"""Ratchet the migration metrics: counts may hold or shrink, never grow.

The migration's promise is monotonic progress — every change leaves the tree
no less safe than it found it. This script is the mechanism. It measures, per
Rust source file (src/**/*.rs plus the crate roots):

  unsafe      occurrences of "unsafe "
  static_mut  occurrences of "static mut "
  no_mangle   occurrences of "#[no_mangle]"
  lines       line count. No file may exceed 1,000 lines; files already over
              the cap are grandfathered at their committed size and may
              shrink or hold, never grow. New files start at the cap.

plus one whole-tree metric: the number of internal-only exports in the
committed ABI ledger (docs/abi-ledger.jsonl — `just abi-ledger --check`
separately guarantees that file matches the tree).

Counting is plain substring matching. That over-counts (a comment saying
"unsafe " counts), but it is deterministic, matches how the migration plan's
baseline numbers were measured, and rustfmt (enforced by fmt-check) keeps the
spelling canonical. The point is monotonic pressure, not precision.

The baseline is committed at docs/ratchet.json (one file per line, so diffs
review like the ledger's). A metric above its baseline is a violation; a
metric below it means progress that must be locked in by regenerating the
baseline and committing it alongside the change.

Usage: ratchet.py [--check] [--allow-growth]
  --check         compare the tree against the committed baseline instead of
                  writing: exit 1 if any metric grew, or if the baseline is
                  stale (a metric shrank but docs/ratchet.json wasn't
                  regenerated).
  --allow-growth  write a baseline even though a metric grew. The override
                  for justified cases — the growth shows up in the
                  docs/ratchet.json diff; explain it in the commit message.
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "docs" / "ratchet.json"
LEDGER = ROOT / "docs" / "abi-ledger.jsonl"

LINE_CAP = 1000
COUNTED = {
    "unsafe": "unsafe ",
    "static_mut": "static mut ",
    "no_mangle": "#[no_mangle]",
}


def measure():
    """repo-relative file -> {metric: count}, zeros included."""
    stats = {}
    for path in sorted([*ROOT.glob("src/**/*.rs"), *ROOT.glob("*.rs")]):
        text = path.read_text()
        counts = {name: text.count(needle) for name, needle in COUNTED.items()}
        counts["lines"] = len(text.splitlines())
        stats[str(path.relative_to(ROOT))] = counts
    return stats


def internal_exports():
    if not LEDGER.exists():
        sys.exit(
            f"ratchet: {LEDGER.relative_to(ROOT)} is missing; "
            "run `just abi-ledger`"
        )
    return sum(
        json.loads(line)["class"] == "internal"
        for line in LEDGER.read_text().splitlines()
    )


def render(stats, internal):
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
            entries.append(f"    {json.dumps(file)}: {json.dumps(kept, sort_keys=True)}")
    body = ",\n".join(entries)
    return (
        "{\n"
        f'  "internal_exports": {internal},\n'
        '  "files": {\n'
        f"{body}\n"
        "  }\n"
        "}\n"
    )


def violations(stats, internal, baseline):
    """Every metric that grew past the committed baseline."""
    found = []
    base_internal = baseline["internal_exports"]
    if internal > base_internal:
        found.append(f"abi-ledger internal exports: {base_internal} -> {internal}")
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


def summary(stats, internal):
    totals = {name: sum(c[name] for c in stats.values()) for name in COUNTED}
    over = sum(c["lines"] > LINE_CAP for c in stats.values())
    parts = [f"{n} {name}" for name, n in totals.items()]
    parts += [f"{over} files over {LINE_CAP} lines", f"{internal} internal exports"]
    return ", ".join(parts)


def main():
    args = set(sys.argv[1:])
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"ratchet: unknown argument(s): {' '.join(sorted(unknown))}")

    stats = measure()
    internal = internal_exports()
    content = render(stats, internal)
    committed = BASELINE.read_text() if BASELINE.exists() else None

    if "--check" in args:
        if committed is None:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is missing; "
                "run `just ratchet`"
            )
        if grew := violations(stats, internal, json.loads(committed)):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "ratchet: counts may only shrink. Reduce them, or if the "
                "growth is justified run `just ratchet --allow-growth` and "
                "explain it in the commit message."
            )
        if committed != content:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is stale (progress "
                "to lock in); run `just ratchet` and commit the result"
            )
        return

    if committed is not None and "--allow-growth" not in args:
        if grew := violations(stats, internal, json.loads(committed)):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "ratchet: refusing to raise the baseline. If the growth is "
                "justified, rerun with --allow-growth."
            )
    BASELINE.write_text(content)
    print(f"wrote {BASELINE.relative_to(ROOT)}: {summary(stats, internal)}")


if __name__ == "__main__":
    main()
