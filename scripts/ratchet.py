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

plus two whole-tree metrics:

  internal_exports  the number of internal-only exports in the committed ABI
                    ledger (metrics/abi-ledger.jsonl — `just abi-ledger
                    --check` separately guarantees that file matches the
                    tree).
  warnings          the number of `cargo build` warnings, from the
                    machine-readable diagnostic stream
                    (--message-format=json). Cargo replays cached
                    diagnostics for up-to-date crates, so on a built tree
                    this costs well under a second; on a stale tree it pays
                    for the build the commit was going to need anyway. When
                    phase 5 drives this to zero, flip CI to `-D warnings`
                    and retire the metric.

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
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "metrics" / "ratchet.json"
LEDGER = ROOT / "metrics" / "abi-ledger.jsonl"

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
        sys.exit(f"ratchet: {LEDGER.relative_to(ROOT)} is missing; run `just refresh`")
    return sum(
        json.loads(line)["class"] == "internal"
        for line in LEDGER.read_text().splitlines()
    )


def build_warnings():
    """Count `cargo build` warnings via the JSON diagnostic stream."""
    proc = subprocess.run(
        ["cargo", "build", "--quiet", "--message-format=json"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        sys.exit(
            "ratchet: `cargo build` failed; the warning count needs a building tree"
        )
    count = 0
    for line in proc.stdout.splitlines():
        message = json.loads(line)
        if message.get("reason") != "compiler-message":
            continue
        diag = message["message"]
        # `code` is null on rustc's closing "N warnings emitted" tally;
        # every real lint carries its lint name there.
        if diag["level"] == "warning" and diag.get("code"):
            count += 1
    return count


def render(stats, internal, warnings):
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
        f'{{\n  "internal_exports": {internal},\n  "warnings": {warnings},\n'
        f'  "files": {{\n{body}\n  }}\n}}\n'
    )


def violations(stats, internal, warnings, baseline):
    """Every metric that grew past the committed baseline."""
    found = []
    base_internal = baseline["internal_exports"]
    if internal > base_internal:
        found.append(f"abi-ledger internal exports: {base_internal} -> {internal}")
    # .get: baselines predating the metric ratify whatever the tree emits
    # (the staleness check still forces a refresh to record it).
    base_warnings = baseline.get("warnings", warnings)
    if warnings > base_warnings:
        found.append(f"cargo build warnings: {base_warnings} -> {warnings}")
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


def summary(stats, internal, warnings):
    totals = {name: sum(c[name] for c in stats.values()) for name in COUNTED}
    over = sum(c["lines"] > LINE_CAP for c in stats.values())
    parts = [f"{n} {name}" for name, n in totals.items()]
    parts += [
        f"{over} files over {LINE_CAP} lines",
        f"{internal} internal exports",
        f"{warnings} build warnings",
    ]
    return ", ".join(parts)


def main():
    args = set(sys.argv[1:])
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"ratchet: unknown argument(s): {' '.join(sorted(unknown))}")

    stats = measure()
    internal = internal_exports()
    warnings = build_warnings()
    content = render(stats, internal, warnings)
    committed = BASELINE.read_text() if BASELINE.exists() else None

    if "--check" in args:
        if committed is None:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is missing; run `just refresh`"
            )
        if grew := violations(stats, internal, warnings, json.loads(committed)):
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
        if grew := violations(stats, internal, warnings, json.loads(committed)):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "ratchet: refusing to raise the baseline. If the growth is "
                "justified, rerun with --allow-growth."
            )
    BASELINE.write_text(content)
    print(f"wrote {BASELINE.relative_to(ROOT)}: {summary(stats, internal, warnings)}")


if __name__ == "__main__":
    main()
