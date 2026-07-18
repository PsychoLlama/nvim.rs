# List available recipes.
_:
  @just --list

# Compile the nvim binary.
build:
  cargo build

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
