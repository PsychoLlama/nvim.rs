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
# `version` names the archive, e.g. `just package 2026.07.18-a1b2c3d4e`, and is
# also what the binary reports as its version: build.rs would otherwise infer
# one from git, which a CI checkout of a tag can't be trusted to answer for.
# Requires the devshell: $NVIM_DEPS_PREFIX is the source of the parsers.
package version:
  #!/usr/bin/env bash
  set -euo pipefail
  export NVIM_RS_VERSION='{{ version }}'
  cargo build --release
  name="nvim-{{ version }}-x86_64-linux"
  stage="target/dist/$name"
  rm -rf "$stage"
  mkdir -p "$stage/bin" "$stage/share/nvim" "$stage/lib/nvim"
  cp target/release/nvim "$stage/bin/nvim"
  cp -r runtime "$stage/share/nvim/runtime"
  cp -r "$NVIM_DEPS_PREFIX/lib/nvim/parser" "$stage/lib/nvim/parser"
  # License texts must travel with the binary: the LGPL'd xdiff/unibilium
  # ports are compiled in, and the (L)GPL requires conveying their texts.
  mkdir -p "$stage/share/doc/nvim"
  cp -r LICENSE.txt licenses "$stage/share/doc/nvim/"
  # Generate the vimscript syntax tables into the staged runtime, as upstream
  # releases ship them. The source runtime deliberately omits generated.vim
  # (the test suites' default runtime must not carry it), so it only exists
  # in generated trees: target/runtime for tests, staged runtimes here.
  HOME="$(mktemp -d)" scripts/gen.sh --nvim target/release/nvim \
    --runtime "$stage/share/nvim/runtime"
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

# Run old (Vim) tests. The mode is required:
#
#   just oldtest all                  # the whole suite, always from scratch
#   just oldtest test_arglist [more]  # only the named tests
#   just oldtest clean                # delete test artifacts, run nothing
#
# There is no incremental full run: `all` always starts from scratch, because
# a resumed one reports only the tests it re-ran. See scripts/run-oldtest.sh.
oldtest +args: build
  scripts/run-oldtest.sh {{ args }}

# Run unit tests. Args: same shape as functionaltest. The LuaJIT FFI
# declarations are generated from the Rust crate itself (tools/ffigen via
# scripts/gen-unit-cdefs.sh), and the tests call the exported symbols of the
# nvim binary. The C fixture helpers (unit-fixtures.so) compile against that
# same generated chunk (test/unit/fixtures/shim.h).
unittest *args: build
  scripts/run-tests.sh unit {{ args }}

# Run benchmarks. Args: same shape as functionaltest.
benchmark *args: build
  scripts/run-tests.sh benchmark {{ args }}

# Run clippy over the two generators (tools/apigen, tools/ffigen). They carry
# their own workspace and lockfile on purpose — membership would make every
# `just build` compile syn — so `cargo clippy` in the root workspace never sees
# them, and this recipe names their manifests explicitly.
#
# Pass/fail at -D warnings rather than ratcheted like the crate: both baselines
# are zero and these are ~8k lines of ordinary hand-written Rust, not
# transpiler output. `--` scopes the denial to the tool itself, leaving the
# vendored syn/proc-macro2 alone. Their formatting needs no recipe: treefmt's
# rustfmt formatter globs *.rs tree-wide, so `just fmt-check` already covers
# them.
lint-tools:
  @cargo clippy --quiet --all-targets --manifest-path tools/apigen/Cargo.toml -- -D warnings
  @cargo clippy --quiet --all-targets --manifest-path tools/ffigen/Cargo.toml -- -D warnings

# Run clippy over every target and ratchet the warning count
# (metrics/clippy.json): per-file counts may only shrink, and deny-level
# findings (the `correctness` group) fail the run outright. Lint levels live
# in Cargo.toml's [lints.clippy]; the script clears RUSTFLAGS so the dev
# shell's `-D warnings` can't promote the counted groups to errors first.
# `--check` compares against the committed baseline instead of writing;
# `--allow-growth` mirrors the ratchet's override.
lint *args: lint-tools
  @scripts/lint.py {{ args }}

# Run the crate's Rust tests: the #[cfg(test)] modules (safe cores' pure
# logic below the C-ABI shims) plus the integration tests under tests/
# (ports of former test/unit specs; they call the same exported surface the
# LuaJIT FFI harness did, minus the child process).
cargo-test *args:
  cargo test --lib --tests {{ args }}

# Run the cargo-test lane under Miri: UB detection (aliasing, provenance,
# uninitialized memory) on the pure-logic tests — the class of bug ASan
# structurally cannot see. Slow (it interprets MIR), so it is not part of
# minimal-ci or the pre-commit hooks; run it before merging any rewrite.
# Uses its own target dir (target/miri), so it doesn't clobber normal builds.
# Isolation is off because the unibi terminfo tests build real directory trees
# in a tempdir; UB detection is unaffected.
miri *args:
  MIRIFLAGS=-Zmiri-disable-isolation cargo miri test --lib --tests {{ args }}

# Regenerate the committed msgpack-RPC dispatch wrappers
# (crates/nvim/src/api/private/dispatch_wrappers/) from the `nvim_*`
# signatures themselves plus tools/apigen/functions.txt, the attributes the
# signatures can't carry. `--check` fails on drift instead of writing.
apigen *args:
  @scripts/gen-api-dispatch.sh {{ args }}

# Regenerate the unit suite's ffi.cdef chunk (tools/ffigen/unit-cdefs.h) from
# the crate's #[repr(C)] types and #[unsafe(no_mangle)] exports. The committed
# copy is a golden, not an input: the harness regenerates its own under
# target/ffi, and this one exists so `--check` can fail when ffigen's output
# drifts — the tool's only test. `--check` regenerates and diffs.
ffigen *args:
  @scripts/gen-unit-cdefs.sh {{ args }}

# Regenerate crates/nvim/src/keycodes.lua from the Rust key-name table
# (crates/nvim/src/keycodes/tables.rs). Nothing in the editor reads it —
# the port answers key-name lookups from the Rust table directly — but
# test/benchmark/keycodes_spec.lua does, and generating beats letting a
# benchmark keep its own copy of a 187-row table. `--check` fails on drift.
keycodes-lua *args:
  @scripts/gen-keycodes-lua.py {{ args }}

# Regenerate the ABI ledger (metrics/abi-ledger.jsonl): classifies every
# #[no_mangle] export by who resolves it by name. `--check` diffs against the
# committed ledger instead of writing.
abi-ledger *args:
  @scripts/abi-ledger.py {{ args }}

# Regenerate the ratchet baseline (metrics/ratchet.json): per-file
# unchecked-line / static mut / #[no_mangle] / variadic / GlobalCell-ptr
# counts, file sizes (1k-line cap, current offenders grandfathered), and the
# ledger's internal-export count may only shrink. `--check` compares against
# the committed baseline instead.
ratchet *args:
  @scripts/ratchet.py {{ args }}

# Regenerate every committed baseline, in the one order that is self-consistent:
# the generated wrappers and the keycode table, then format, then the ABI
# ledger, then the ratchet, then re-check formatting. This is the entry point;
# running the pieces by hand invites a baseline that describes a tree that no
# longer exists.
#
# Code generation leads because it writes crate source every later step reads.
# The cdefs golden regenerates after formatting rather than with the other
# generators: it is derived from the crate source apigen just wrote, and it is
# not crate source itself, so nothing downstream reads it.
# Formatting comes next because rustfmt rewrapping a line changes the line counts
# the ratchet measures — and `fmt-check` (the pre-commit hook) rewrites the tree, so
# a baseline taken before it silently stops matching mid-commit. The ledger
# precedes the ratchet because the ratchet snapshots its internal-export count.
# The lint baseline comes last: it runs cargo clippy, by far the slowest step
# (a full check-mode compile when the tree changed), and depends on nothing
# the earlier steps write.
# The closing pass is uncached on purpose: cached, it would skip the files `fmt`
# just rewrote and prove nothing, where uncached it asserts formatting reached a
# fixed point that the pre-commit hook can't move.
#
# Args are forwarded to the ratchet and lint, e.g. `just refresh --allow-growth`.
refresh *args: apigen keycodes-lua fmt ffigen abi-ledger (ratchet args) (lint args)
  @treefmt --no-cache --fail-on-change --quiet

# This is the gate CI runs on every push. It deliberately skips the slow
# suites, which are worth invoking directly (`just functionaltest`,
# `just oldtest`, ...); only the fast Rust-side tests run here.
#
# Check that the tree is formatted, every generator still reproduces its
# committed output, the ABI ledger is current and the ratchet holds, the
# generators and the crate compile clean, and the safe-core tests pass.
# fmt-check leads because it rewrites the tree. The ledger check precedes the
# ratchet check because the ratchet snapshots the ledger's internal-export
# count and cannot tell a stale ledger from a fresh one (both also run as
# pre-commit hooks, see .gitconfig). lint-tools is here rather than in `lint`
# alone because it is seconds, where the crate's clippy pass is minutes.
minimal-ci: fmt-check (apigen "--check") (ffigen "--check") (keycodes-lua "--check") (abi-ledger "--check") (ratchet "--check") lint-tools build cargo-test
