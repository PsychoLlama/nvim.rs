# AddressSanitizer builds and test runs, e.g. `just asan functionaltest`.
mod asan 'just/asan.just'

set indentation := "  "
set default-list

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

# Check formatting without writing changes; fails if anything is unformatted.
fmt-check:
  treefmt --fail-on-change

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

# This is the gate CI runs on every push. It deliberately runs no tests: the
# suites are slow and worth invoking directly (`just functionaltest`,
# `just oldtest`, ...).
#
# Check that the tree is formatted and still compiles.
smoke-test: fmt-check build
