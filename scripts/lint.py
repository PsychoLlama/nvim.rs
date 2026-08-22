#!/usr/bin/env python3
"""Ratchet the clippy warning count: per-file counts may hold or shrink.

Runs `cargo clippy --all-targets` and holds the tree to three rules:

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
  posture   two rustc lints the migration is burning to zero,
            `unreachable_pub` and `unused_qualifications`, forced to warn
            for this run only (they are allow-by-default) and counted as
            whole-tree totals that may hold or shrink.

            They ride along here rather than in metrics/ratchet.json
            because they are compiler findings, not needles a source scan
            can count, and ratchet.py is a pre-commit hook that must not
            invoke cargo. Counted as totals rather than per file on
            purpose: the target is zero and most of the findings are
            `--fix`-applicable, so a per-file baseline would be several
            hundred lines of churn for a metric with a short life. A total
            that may only fall still forbids trading a fix in one module
            for a new one in another.

            `--force-warn` is passed after `--`, so it reaches the
            workspace's own crates and not the dependency graph, and the
            count is restricted to crates/*/src -- the same surface
            ratchet.py measures, so a `pub` helper shared between two
            integration-test modules is not counted as debt.

            Baselined at 602 / 148. The `--lib`-only survey figure for
            `unused_qualifications` was 140; the other 8 are inside
            `#[cfg(test)] mod tests` blocks, which --all-targets compiles
            and ratchet.py likewise counts.

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
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "metrics" / "clippy.json"

# The two rustc lints counted separately below. Both are allow-by-default,
# so they have to be asked for; `--force-warn` also makes it impossible for
# the dev shell's `-D warnings` to turn them into errors mid-count.
POSTURE_LINTS = ("unreachable_pub", "unused_qualifications")
# What the posture totals are measured over: the same files ratchet.py
# measures, i.e. crate sources and not the integration tests beside them.
MIGRATION_SURFACE = re.compile(r"^crates/[^/]+/src/")

CLIPPY = [
    "cargo",
    "clippy",
    "--all-targets",
    "--message-format=json",
    "--",
    *(f"--force-warn={lint}" for lint in POSTURE_LINTS),
]


def run_clippy():
    """(per-file warning Counter, posture Counter, rendered errors)."""
    env = {**os.environ, "RUSTFLAGS": ""}
    proc = subprocess.run(CLIPPY, cwd=ROOT, env=env, capture_output=True, text=True)
    warnings = Counter()
    posture = Counter()
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
        elif code in POSTURE_LINTS:
            # Migration surface only, as ratchet.py measures it: the
            # integration tests under crates/*/tests are not code the
            # migration is burning down, and a `pub` helper shared between
            # two test modules is not a posture problem.
            if MIGRATION_SURFACE.match(primary.get("file_name", "")):
                posture[code] += 1
        else:
            warnings[primary.get("file_name", "<unknown>")] += 1
    # A nonzero exit with no error diagnostics means clippy itself fell
    # over (broken build, bad flags) — surface that instead of counting.
    if proc.returncode != 0 and not errors:
        sys.stderr.write(proc.stderr)
        sys.exit("lint: cargo clippy failed without emitting diagnostics")
    return warnings, posture, errors


def render(warnings, posture):
    entries = [
        f"    {json.dumps(file)}: {count}"
        for file, count in sorted(warnings.items())
        if count > 0
    ]
    files = "{}" if not entries else "{\n" + ",\n".join(entries) + "\n  }"
    totals = "".join(f',\n  "{lint}": {posture[lint]}' for lint in POSTURE_LINTS)
    return f'{{\n  "files": {files}{totals}\n}}\n'


def violations(warnings, posture, baseline):
    base_files = baseline["files"]
    found = [
        f"{file}: clippy warnings {base_files.get(file, 0)} -> {warnings[file]}"
        for file in sorted(warnings.keys() | base_files.keys())
        if warnings.get(file, 0) > base_files.get(file, 0)
    ]
    for lint in POSTURE_LINTS:
        # .get: absent from baselines committed before the metric existed.
        base = baseline.get(lint, posture[lint])
        if posture[lint] > base:
            found.append(f"{lint}: {base} -> {posture[lint]}")
    return found


def main():
    args = set(sys.argv[1:])
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"lint: unknown argument(s): {' '.join(sorted(unknown))}")

    warnings, posture, errors = run_clippy()
    if errors:
        print("\n".join(errors), file=sys.stderr)
        sys.exit(
            f"lint: {len(errors)} deny-level finding(s). Fix them (they are "
            "bugs, not style), or justify a false positive with a "
            "site-level #[allow]."
        )

    content = render(warnings, posture)
    committed = BASELINE.read_text() if BASELINE.exists() else None
    summary = (
        f"{sum(warnings.values())} clippy warnings in {len(warnings)} files, "
        + ", ".join(f"{posture[lint]} {lint}" for lint in POSTURE_LINTS)
    )

    if "--check" in args:
        if committed is None:
            sys.exit(f"lint: {BASELINE.relative_to(ROOT)} is missing; run `just lint`")
        if grew := violations(warnings, posture, json.loads(committed)):
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
        if grew := violations(warnings, posture, json.loads(committed)):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "lint: refusing to raise the baseline. If the growth is "
                "justified, rerun with --allow-growth."
            )
    BASELINE.write_text(content)
    print(f"wrote {BASELINE.relative_to(ROOT)}: {summary}")


if __name__ == "__main__":
    main()
