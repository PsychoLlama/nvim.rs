#!/usr/bin/env bash
# Reconstruct the upstream C tree under target/upstream.
#
# The unit specs' ffi.cdef surface is generated from the Rust crate these
# days (tools/ffigen); this tree remains for the consumers that still need
# real C headers: compiling test/unit/fixtures into unit-fixtures.so, and
# scripts/check-unit-cdefs.py, which diffs the generated declarations
# against these headers.
#
# The headers include CMake-generated files (*.h.generated.h declarations,
# auto/config.h, ...), so this extracts the tag into target/upstream/src,
# configures it with the same Nix deps the port builds against, and builds
# only the generator targets (no C objects).
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
: "${NVIM_DEPS_PREFIX:?NVIM_DEPS_PREFIX must be set (enter the flake dev shell)}"

tag=v0.12.4
up=$root/target/upstream
stamp=$up/.generated-stamp

if [[ -f $stamp && $(cat "$stamp") == "$tag" ]]; then
  exit 0
fi

if [[ ! -f $up/src/CMakeLists.txt ]]; then
  mkdir -p "$up/src"
  git -C "$root" archive "$tag" | tar -x -C "$up/src"
fi

cmake -S "$up/src" -B "$up/build" -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo \
  -DDEPS_PREFIX="$NVIM_DEPS_PREFIX" -DCMAKE_PREFIX_PATH="$NVIM_DEPS_PREFIX" \
  >/dev/null
ninja -C "$up/build" src/nvim/generated-sources >/dev/null

echo "$tag" >"$stamp"
