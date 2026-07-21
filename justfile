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

# Format the tree in place.
fmt:
  @treefmt --quiet

# Fail if anything was unformatted. NB: treefmt always writes; `--fail-on-change`
# only adds the nonzero exit. So by the time this recipe fails it has already
# rewritten the worktree, and any measurement taken before it (line counts, the
# baselines) is stale — which is why `just refresh` formats first.
# `--quiet` keeps success silent (pre-commit hooks only speak up on failure);
# the offending paths are still reported on failure.
fmt-check:
  @treefmt --fail-on-change --quiet

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

# Run the crate's Rust tests (`cargo test --lib`, i.e. #[cfg(test)]
# modules). Today these cover the safe cores' pure logic below the C-ABI
# shims, but the suite is general and will grow beyond that.
cargo-test *args:
  cargo test --lib {{ args }}

# Regenerate the ABI ledger (docs/abi-ledger.jsonl): classifies every
# #[no_mangle] export by who resolves it by name. `--check` diffs against the
# committed ledger instead of writing.
abi-ledger *args:
  @scripts/abi-ledger.py {{ args }}

# Regenerate the ratchet baseline (docs/ratchet.json): per-file unsafe /
# static mut / #[no_mangle] counts, file sizes (1k-line cap, current
# offenders grandfathered), and the ledger's internal-export count may only
# shrink. `--check` compares against the committed baseline instead.
ratchet *args:
  @scripts/ratchet.py {{ args }}

# Regenerate every committed baseline, in the one order that is self-consistent:
# format, then the ABI ledger, then the ratchet, then re-check formatting. This
# is the entry point; running the pieces by hand invites a baseline that
# describes a tree that no longer exists.
#
# Formatting leads because rustfmt rewrapping a line changes the line counts the
# ratchet measures — and `fmt-check` (the pre-commit hook) rewrites the tree, so
# a baseline taken before it silently stops matching mid-commit. The ledger
# precedes the ratchet because the ratchet snapshots its internal-export count.
# The closing pass is uncached on purpose: cached, it would skip the files `fmt`
# just rewrote and prove nothing, where uncached it asserts formatting reached a
# fixed point that the pre-commit hook can't move.
#
# Args are forwarded to the ratchet, e.g. `just refresh --allow-growth`.
refresh *args: fmt abi-ledger (ratchet args)
  @treefmt --no-cache --fail-on-change --quiet

# This is the gate CI runs on every push. It deliberately skips the slow
# suites, which are worth invoking directly (`just functionaltest`,
# `just oldtest`, ...); only the fast Rust-side tests run here.
#
# Check that the tree is formatted, the ratchet and ABI ledger hold, the
# crate still compiles, and the safe-core tests pass. Same order as the
# pre-commit hooks in .gitconfig: fmt-check rewrites the tree, and the ratchet
# reads the ledger.
minimal-ci: fmt-check (abi-ledger "--check") (ratchet "--check") build cargo-test
