#!/usr/bin/env bash
# Reconstruct the upstream C tree that the unit tests (test/unit) need.
#
# Unit tests work by preprocessing the original C headers with `cc -E`,
# feeding the declarations to LuaJIT's ffi.cdef, and calling the symbols of
# the *running nvim process* (ffi.C; the binary links with --export-dynamic).
# The port keeps C ABI and struct layouts, so the v0.12.4 headers still
# describe the transpiled Rust faithfully.
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
