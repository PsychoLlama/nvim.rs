#!/usr/bin/env python3
"""Ratchet the clippy warning count: per-file counts may hold or shrink.

Runs `cargo clippy --all-targets` and holds the tree to two rules:

  errors    fail the run outright, always. Clippy ships its `correctness`
            group at deny — those are bugs, not style — so any hit (or any
            group later promoted to deny) must be fixed on the spot, or
            carry a site-level `#[allow]` whose comment justifies the false
            positive.
  warnings  are counted per file into a shrink-only baseline
            (metrics/clippy.json), the same contract as metrics/ratchet.json:
            above the committed count is a violation, below it is progress
            that must be locked in by regenerating the baseline. The groups
            that warn (suspicious/complexity/perf — style and the opt-in
            groups are configured off in Cargo.toml's [lints.clippy]) burn
            down through this ratchet as modules are rewritten.

A third rule used to live here: `unreachable_pub` and
`unused_qualifications` counted as whole-tree, shrink-only totals (baselined
at 602/148). Both are zero, and both are now `deny` in the packages'
[lints.rust] tables, so a new finding is an error this script reports under
the first rule rather than a number that may not grow. The counter is retired
the way ratchet.py's `warnings` metric was when RUSTFLAGS took it over: an
invariant the compiler enforces beats a baseline that records it.

RUSTFLAGS is cleared for the clippy invocation: the dev shell's
`-D warnings` would promote the ratcheted groups to hard errors before they
could be counted. Regular builds keep enforcing it; this lane only decides
lint levels through clippy's defaults plus Cargo.toml.

Identical diagnostics are deduplicated across targets (the lib compiles
once per target that links it), keyed by lint, primary span, and message.

Usage: lint.py [--check] [--allow-growth]
  --check         compare against the committed baseline instead of
                  writing: exit 1 if any error was emitted, any per-file
                  count grew, or the baseline is stale (a count shrank but
                  metrics/clippy.json wasn't regenerated).
  --allow-growth  write a baseline even though a count grew; explain the
                  growth in the commit message.

Regenerate through `just refresh` (which runs this) or `just lint`.
"""

import json
import os
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "metrics" / "clippy.json"

CLIPPY = ["cargo", "clippy", "--all-targets", "--message-format=json"]


def run_clippy():
    """(per-file warning Counter, rendered errors)."""
    env = {**os.environ, "RUSTFLAGS": ""}
    proc = subprocess.run(CLIPPY, cwd=ROOT, env=env, capture_output=True, text=True)
    warnings = Counter()
    errors = []
    seen = set()
    for line in proc.stdout.splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("reason") != "compiler-message":
            continue
        msg = record["message"]
        level = msg.get("level")
        # No code means a per-target "N warnings emitted" summary.
        if level not in ("warning", "error") or msg.get("code") is None:
            continue
        primary = next((s for s in msg["spans"] if s["is_primary"]), {})
        code = msg["code"]["code"]
        key = (
            code,
            primary.get("file_name"),
            primary.get("line_start"),
            primary.get("column_start"),
            msg["message"],
        )
        if key in seen:
            continue
        seen.add(key)
        if level != "warning":
            errors.append(msg.get("rendered") or msg["message"])
        else:
            warnings[primary.get("file_name", "<unknown>")] += 1
    # A nonzero exit with no error diagnostics means clippy itself fell
    # over (broken build, bad flags) — surface that instead of counting.
    if proc.returncode != 0 and not errors:
        sys.stderr.write(proc.stderr)
        sys.exit("lint: cargo clippy failed without emitting diagnostics")
    return warnings, errors


def render(warnings):
    entries = [
        f"    {json.dumps(file)}: {count}"
        for file, count in sorted(warnings.items())
        if count > 0
    ]
    files = "{}" if not entries else "{\n" + ",\n".join(entries) + "\n  }"
    return f'{{\n  "files": {files}\n}}\n'


def violations(warnings, baseline):
    base_files = baseline["files"]
    return [
        f"{file}: clippy warnings {base_files.get(file, 0)} -> {warnings[file]}"
        for file in sorted(warnings.keys() | base_files.keys())
        if warnings.get(file, 0) > base_files.get(file, 0)
    ]


def main():
    args = set(sys.argv[1:])
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"lint: unknown argument(s): {' '.join(sorted(unknown))}")

    warnings, errors = run_clippy()
    if errors:
        print("\n".join(errors), file=sys.stderr)
        sys.exit(
            f"lint: {len(errors)} deny-level finding(s). Fix them (they are "
            "bugs, not style), or justify a false positive with a "
            "site-level #[allow]."
        )

    content = render(warnings)
    committed = BASELINE.read_text() if BASELINE.exists() else None
    summary = f"{sum(warnings.values())} clippy warnings in {len(warnings)} files"

    if "--check" in args:
        if committed is None:
            sys.exit(f"lint: {BASELINE.relative_to(ROOT)} is missing; run `just lint`")
        if grew := violations(warnings, json.loads(committed)):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "lint: warning counts may only shrink. Reduce them, or if "
                "the growth is justified run `just lint --allow-growth` and "
                "explain it in the commit message."
            )
        if committed != content:
            sys.exit(
                f"lint: {BASELINE.relative_to(ROOT)} is stale (progress to "
                "lock in); run `just lint` and commit the result"
            )
        print(f"lint: clean ({summary})")
        return

    if committed is not None and "--allow-growth" not in args:
        if grew := violations(warnings, json.loads(committed)):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "lint: refusing to raise the baseline. If the growth is "
                "justified, rerun with --allow-growth."
            )
    BASELINE.write_text(content)
    print(f"wrote {BASELINE.relative_to(ROOT)}: {summary}")


if __name__ == "__main__":
    main()
