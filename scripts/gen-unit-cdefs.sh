#!/usr/bin/env bash
# Generate the unit tests' LuaJIT ffi.cdef chunk from the Rust crate.
#
# tools/ffigen parses src/**/*.rs and emits target/ffi/unit-cdefs.h (plus a
# manifest for scripts/check-unit-cdefs.py). test/unit/testutil.lua cdefs the
# chunk instead of preprocessing the reconstructed v0.12.4 C headers, so the
# declarations the specs run against always describe the code actually built.
#
# Skips regeneration when the chunk is newer than every crate source, the
# tool, and its deny list.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
out=$root/target/ffi/unit-cdefs.h
manifest=$root/target/ffi/manifest.json

if [[ -f $out ]]; then
  newer=$(find "$root/src" "$root/tools/ffigen/src" "$root/tools/ffigen/deny.txt" \
    -name '*.rs' -newer "$out" -print -quit 2>/dev/null)
  deny_newer=$(find "$root/tools/ffigen/deny.txt" -newer "$out" -print -quit)
  if [[ -z $newer && -z $deny_newer && -f $manifest ]]; then
    exit 0
  fi
fi

# The tool builds with the dev-shell toolchain (its Cargo.lock is v3 for that
# reason). RUSTFLAGS from the dev shell (-D warnings) is fine here.
cargo build --release --quiet --manifest-path "$root/tools/ffigen/Cargo.toml"

echo "generating unit-test cdefs from the crate" >&2
"$root/tools/ffigen/target/release/ffigen" \
  --root "$root" \
  --deny "$root/tools/ffigen/deny.txt" \
  --out "$out" \
  --manifest "$manifest" \
  2> >(grep -Ev '^ffigen: (parsing|[0-9]+ type names|emitted)' >&2 || true)
