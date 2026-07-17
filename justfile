# List available recipes.
_:
  @just --list

# Compile the nvim binary.
build:
  cargo build

# Check formatting without writing changes; fails if anything is unformatted.
fmt-check:
  treefmt --ci

# Run the full test suite. This is slow! Prefer running specific tests.
check: fmt-check build
