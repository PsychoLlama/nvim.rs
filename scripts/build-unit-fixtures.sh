#!/usr/bin/env bash
# Compile test/unit/fixtures into unit-fixtures.so.
#
# Upstream compiled these helpers into the test build of libnvim. We build
# them as a shared library instead; test/unit/preload.lua loads it RTLD_GLOBAL
# so ffi.C sees the symbols, and its nvim references resolve from the binary
# (linked with --export-dynamic).
#
# The nvim declaration surface is the ffigen-generated cdefs chunk (see
# test/unit/fixtures/shim.h), so the fixtures compile against the layouts of
# the crate under test — no C headers beyond this repo. VTERM_TEST_FILE is
# where the vterm fixture logs callbacks; must match paths.vterm_test_file in
# test/cmakeconfig/paths.lua (upstream wired the same pair through CMake).
#
# Shared by run-tests.sh (the unit suite loads it) and abi-ledger.py (its
# undefined symbols are part of the export contract: they must keep resolving
# from the nvim binary). Skips the compile when the output is newer than the
# sources.
#
# Usage: build-unit-fixtures.sh <output.so>
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)

out=${1:?usage: $0 <output.so>}

# The fixtures include the generated cdefs chunk; (re)generate it if stale.
"$root/scripts/gen-unit-cdefs.sh"

chunk=$root/target/ffi/unit-cdefs.h
unit_fixtures=$root/test/unit/fixtures
stale=0
for src in "$unit_fixtures"/multiqueue.{c,h} "$unit_fixtures"/vterm_test.{c,h} \
  "$unit_fixtures/shim.h" "$chunk"; do
  if [[ ! -e $out || $src -nt $out ]]; then
    stale=1
  fi
done
if [[ $stale == 1 ]]; then
  echo "compiling test fixture: $(basename "$out")" >&2
  mkdir -p "$(dirname "$out")"
  cc -shared -fPIC -O2 -o "$out" \
    "$unit_fixtures/multiqueue.c" "$unit_fixtures/vterm_test.c" \
    -I"$unit_fixtures" -I"$root/target/ffi" \
    -DVTERM_TEST_FILE="\"$root/target/vterm_test_output\""
fi
