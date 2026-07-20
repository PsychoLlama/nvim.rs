# List available recipes.
_:
  @just --list

# Compile the nvim binary.
build:
  cargo build

# Compile the nvim binary in release mode (stripped; see [profile.release]).
build-release:
  cargo build --release

# Assemble a relocatable release tarball under `target/dist`. Mirrors the
# layout nix/package.nix installs (bin/ + runtime + tree-sitter parsers) but
# with a cargo-built binary: the baked default paths don't exist on a consumer
# machine, so nvim falls through to exe-relative resolution of this tree.
# `version` names the archive, e.g. `just package 2026.07.18-a1b2c3d4e`.
# Requires the devshell: $NVIM_DEPS_PREFIX is the source of the parsers.
package version: build-release
  #!/usr/bin/env bash
  set -euo pipefail
  name="nvim-{{ version }}-x86_64-linux"
  stage="target/dist/$name"
  rm -rf "$stage"
  mkdir -p "$stage/bin" "$stage/share/nvim" "$stage/lib/nvim"
  cp target/release/nvim "$stage/bin/nvim"
  cp -r runtime "$stage/share/nvim/runtime"
  cp -r "$NVIM_DEPS_PREFIX/lib/nvim/parser" "$stage/lib/nvim/parser"
  # Regenerate help tags against the staged docs, as nix/package.nix does.
  HOME="$(mktemp -d)" target/release/nvim --headless -u NONE \
    -c "helptags $stage/share/nvim/runtime/doc" -c "qa!"
  chmod -R u+w "$stage"
  tar czf "$stage.tar.gz" -C target/dist "$name"
  echo "Wrote $stage.tar.gz"

# AddressSanitizer builds (the phase-1 safety net from the migration plan).
# All heap allocation flows through libc malloc, so ASan's interposer catches
# use-after-free/double-free/out-of-bounds across the whole binary even though
# the C deps are not (yet) instrumented. An explicit --target keeps RUSTFLAGS
# off host build scripts; -Zbuild-std instruments std's own allocations; a
# separate target dir keeps ASan artifacts out of the normal build cache.
asan_triple := `rustc -vV | sed -n 's/^host: //p'`
asan_bin := justfile_directory() / "target/asan" / asan_triple / "debug/nvim"

# ASan reports go to files (nvim is a TUI; stderr is often a terminal the
# harness owns). Leak checking stays off until the corruption classes are
# triaged. Callers may override by exporting ASAN_OPTIONS.
asan_options := "detect_leaks=0:log_path=" / justfile_directory() / "target/asan/asan.log"

# Stack-redzone checking is off (-asan-stack=0): this nightly's C-ABI lowering
# over-reads by-value aggregates whose size is not a multiple of 8 (e.g. the
# 12-byte pos_T is read with a 16-byte load), tripping stack-buffer-overflow
# on ~every by-value struct call in the transpiled code. The over-read never
# leaves the caller's frame, so it is benign outside ASan. Revisit when the
# pinned toolchain moves past the cast-ABI fixes.
#
# Compile the nvim binary with AddressSanitizer.
build-asan:
  RUSTFLAGS="-Zsanitizer=address -Cllvm-args=-asan-stack=0" cargo build -Zbuild-std --target {{ asan_triple }} --target-dir target/asan

# setarch -R disables ASLR (inherited by every spawned nvim): this toolchain's
# ASan runtime predates LLVM's fix for kernels with 32-bit mmap entropy, so
# with ASLR on, ~1/3 of processes randomly land a mapping inside ASan's fixed
# shadow regions and SIGSEGV in the dynamic loader before main.
#
# Reports land in target/asan/asan.log.<pid>.
#
# Run functional tests against the ASan build. Args as for `functionaltest`.
functionaltest-asan *args: build-asan
  #!/usr/bin/env bash
  set -euo pipefail
  export ASAN_OPTIONS="${ASAN_OPTIONS:-{{ asan_options }}}"
  export NVIM_TEST_ASAN=1
  # Tests run target/bin/nvim (the layout specs assert); prep-test-tree.sh
  # points it at $NVIM_BIN.
  export NVIM_BIN={{ asan_bin }}
  exec setarch "$(uname -m)" -R scripts/run-tests.sh functional {{ args }}

# Reports land in target/asan/asan.log.<pid>.
#
# Run old (Vim) tests against the ASan build. Args as for `oldtest`.
oldtest-asan *args: build-asan
  #!/usr/bin/env bash
  set -euo pipefail
  export ASAN_OPTIONS="${ASAN_OPTIONS:-{{ asan_options }}}"
  export NVIM_TEST_ASAN=1
  export NVIM_BIN={{ asan_bin }}
  scripts/prep-test-tree.sh
  exec setarch "$(uname -m)" -R make -C test/old/testdir NVIM_PRG={{ justfile_directory() }}/target/bin/nvim {{ args }}

# Check formatting without writing changes; fails if anything is unformatted.
fmt-check:
  treefmt --ci

# Run functional tests. Args: spec paths and/or harness flags, e.g.
# `just functionaltest test/functional/core --filter='startup'`.
functionaltest *args: build
  scripts/run-tests.sh functional {{ args }}

# Run old (Vim) tests. Args: test names, e.g. `just oldtest test_arglist`.
# Full runs are incremental (make): `just oldtest clean` forces a fresh pass.
oldtest *args: build
  scripts/prep-test-tree.sh
  make -C test/old/testdir NVIM_PRG={{ justfile_directory() }}/target/debug/nvim {{ args }}

# Run unit tests. Args: same shape as functionaltest. The upstream v0.12.4 C
# headers (reconstructed under target/upstream on first run) are preprocessed
# into LuaJIT FFI declarations, and the tests call the transpiled symbols
# exported by the nvim binary itself.
unittest *args: build
  scripts/run-tests.sh unit {{ args }}

# Run benchmarks. Args: same shape as functionaltest.
benchmark *args: build
  scripts/run-tests.sh benchmark {{ args }}

# Run the full test suite. This is slow! Prefer running specific tests.
check: fmt-check build
