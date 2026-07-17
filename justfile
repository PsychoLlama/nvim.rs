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

# Run benchmarks. Args: same shape as functionaltest.
benchmark *args: build
  scripts/run-tests.sh benchmark {{ args }}

# NOTE: upstream's unit tests (test/unit) cannot run: they FFI into the C
# internals by preprocessing src/nvim/*.h headers, which the c2rust port no
# longer has. Internal coverage now comes from `cargo test` as Rust tests
# accumulate.

# Run the full test suite. This is slow! Prefer running specific tests.
check: fmt-check build
