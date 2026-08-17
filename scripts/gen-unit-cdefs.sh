#!/usr/bin/env bash
# Generate the unit tests' LuaJIT ffi.cdef chunk from the Rust crate.
#
# tools/ffigen parses crates/nvim/src/**/*.rs and emits target/ffi/unit-cdefs.h.
# test/unit/testutil.lua cdefs the chunk, and the C fixture helpers compile
# against it (test/unit/fixtures/shim.h), so the declarations the specs run
# against always describe the code actually built.
#
# Skips regeneration when the chunk is newer than every crate source, the
# tool, and its deny list.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
out=$root/target/ffi/unit-cdefs.h

if [[ -f $out ]]; then
  newer=$(find "$root/crates/nvim" "$root/tools/ffigen/src" "$root/tools/ffigen/deny.txt" \
    -name '*.rs' -newer "$out" -print -quit 2>/dev/null)
  deny_newer=$(find "$root/tools/ffigen/deny.txt" -newer "$out" -print -quit)
  if [[ -z $newer && -z $deny_newer ]]; then
    exit 0
  fi
fi

# The tool builds with the dev-shell toolchain. RUSTFLAGS from the dev shell
# (-D warnings) is fine here.
cargo build --release --quiet --manifest-path "$root/tools/ffigen/Cargo.toml"

echo "generating unit-test cdefs from the crate" >&2
# --root is the crate dir, not the repo root: ffigen walks <root>/src and
# derives its keys ("src/...") relative to it. Those keys are internal — the
# cimport paths the unit specs pass are labels testutil rewrites away — so
# they stayed stable across the crates/nvim move and the src/nvim flatten.
"$root/tools/ffigen/target/release/ffigen" \
  --root "$root/crates/nvim" \
  --deny "$root/tools/ffigen/deny.txt" \
  --out "$out" \
  2> >(grep -Ev '^ffigen: (parsing|[0-9]+ type names|emitted)' >&2 || true)
