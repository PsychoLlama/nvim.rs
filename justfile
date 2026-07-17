# List available recipes.
_:
  @just --list

# Compile the nvim binary.
build:
  cargo build

# Format the tree with treefmt.
fmt:
  treefmt

# Check formatting without writing changes; fails if anything is unformatted.
fmt-check:
  treefmt --ci

# Run the full check suite (formatting + compile). Used by CI.
check: fmt-check build
