#!/usr/bin/env bash
# Compile test/unit/fixtures into unit-fixtures.so.
#
# Upstream compiled these helpers into the test build of libnvim. We build
# them as a shared library instead; test/unit/preload.lua loads it RTLD_GLOBAL
# so ffi.C sees the symbols, and its nvim references resolve from the binary
# (linked with --export-dynamic).
#
# Shared by run-tests.sh (the unit suite loads it) and abi-ledger.py (its
# undefined symbols are part of the export contract: they must keep resolving
# from the nvim binary). Skips the compile when the output is newer than the
# sources.
#
# Usage: build-unit-fixtures.sh <output.so>
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
: "${NVIM_DEPS_PREFIX:?NVIM_DEPS_PREFIX must be set (enter the flake dev shell)}"

out=${1:?usage: $0 <output.so>}

# The fixtures include the upstream C headers; reconstruct them if needed.
"$root/scripts/prep-unit-headers.sh"

up=$root/target/upstream
unit_fixtures=$root/test/unit/fixtures
if [[ ! -e $out || $unit_fixtures/multiqueue.c -nt $out ||
  $unit_fixtures/vterm_test.c -nt $out ]]; then
  echo "compiling test fixture: $(basename "$out")" >&2
  mkdir -p "$(dirname "$out")"
  cc -shared -fPIC -O2 -o "$out" \
    "$unit_fixtures/multiqueue.c" "$unit_fixtures/vterm_test.c" \
    -I"$unit_fixtures" -I"$up/src/src" -I"$up/build/src/nvim/auto" \
    -I"$up/build/include" -I"$up/build/cmake.config" \
    -I"$NVIM_DEPS_PREFIX/include"
fi
