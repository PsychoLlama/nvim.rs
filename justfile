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

# Fail if `TypVal`'s or `Object`'s drop shim grew a cleanup path.
#
# Both types release themselves -- an iterative walk that takes each owning
# payload out of its slot -- so every payload they hold sits in a
# `ManuallyDrop` and the compiler appends nothing after `Drop::drop`. Give
# one of them a field the compiler *does* drop and `drop_in_place` grows a
# landing pad, which stops the shim being a tail call and stops it inlining
# at the hundreds of sites that drop a value; that is worth ~0.5 % on four
# of the five benches, and nothing in the source or the suites shows it.
#
# Reads the shipped shape, so it wants a `--release` binary: the argument
# defaults to `target/release/nvim` and `just build-release` makes one.
drop-glue binary='target/release/nvim':
  scripts/drop-glue.py {{ binary }}

# A/B two nvim binaries on one whole-binary bench, e.g.
# `just bench-ab scrbench /tmp/a/nvim /tmp/b/nvim`. The bench is a name from
# test/benchmark/ab (evalbench, inbench, mlbench, scrbench, spellbench);
# extra args go to the driver (`[rounds]`, or `--cachegrind <nvim>` in place
# of the two binaries, which is the measurement that actually settles a perf
# question). Nothing gates on these: read test/benchmark/ab/README.md before
# quoting a number, and note that the wall-clock lane wants both sides built
# `--release` with `codegen-units = 1`, which `just build-release` is not.
bench-ab bench *args:
  @test/benchmark/ab/{{ bench }}.sh {{ args }}

# Thirty-two differential oracles plus the startup probe, run over the binary
# this tree builds. Every row must say IDENTICAL and the last line must be
# `BATTERY_EXIT=0`; anything else is a behaviour change, intended or not, and
# the pin it is measured against (test/battery/BASE) moves in a commit of its
# own — never the one it gates. See test/battery/README.md.
#
# No baseline is committed: each row's is cut from the binary BASE pins and
# cached under target/battery/base/<sha>/. A cold run builds that binary once
# (~1 min) and then runs every row twice, ~14 min; a warm one is ~7 min.
#
# `label` names the log set under target/battery-logs (default: short HEAD), so
# two runs can be diffed line for line.
#
# Run the differential battery over the built binary.
battery label='':
  test/battery/battery.sh "{{ if label == '' { '$(git rev-parse --short HEAD)' } else { label } }}"

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
#
# It used to also count `unreachable_pub` and `unused_qualifications` as
# whole-tree, shrink-only totals. Both reached zero and are now `deny` in the
# packages' [lints.rust], so a new one is a build error rather than a number:
# inside the crate, `pub` means externally reachable.
# `--check` compares against the committed baseline instead of writing;
# `--allow-growth` mirrors the ratchet's override.
lint *args: lint-tools
  @scripts/lint.py {{ args }}

# Build the crate's rustdoc and fail on a broken intra-doc link.
#
# `[`foo`]` in a doc comment is a claim that `foo` exists and is reachable
# from here; rustdoc leaves a broken one as literal text, so the prose keeps
# naming a function that was renamed or moved and nothing says so. Denied,
# not warned, because the baseline is zero and a ratchet would be a worse
# instrument than a gate for a number that small.
#
# `--no-deps` because the dependencies' docs are not ours to fix, and `--lib`
# because the binaries carry no prose of their own. Private items are NOT
# documented, so `private_intra_doc_links` stays a warning: a link into a
# private item is legitimate here — most of the crate is private — and
# denying it would mean deleting links that read correctly in the source.
doc *args:
  @RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --workspace --no-deps --lib --quiet {{ args }}

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

# The `--lib` half of that lane: the crate's #[cfg(test)] modules alone,
# without the integration tests under tests/. ~6.5 minutes against the full
# lane's ~15, and it is the half that covers the safe cores a rewrite actually
# moves — the tests/ half drives a live editor and spends its time in setup.
# Still far too slow for the push gate; reach for it when a rewrite touches
# aliasing or provenance and the full lane's extra ~8 minutes aren't worth
# the wait.
miri-lib *args:
  MIRIFLAGS=-Zmiri-disable-isolation cargo miri test --lib {{ args }}

# Run the cargo-test lane with the struct layouts shuffled, to catch code
# that assumes a `repr(Rust)` type's field order or size. `seed` picks the
# permutation; run at least two, since one seed proves nothing on its own.
#
# `--cfg randomized_layout` travels with it (declared in crates/nvim/
# Cargo.toml): a handful of const assertions pin what a dictionary item or a
# list item *costs*, which is a real budget for the shipped layout and a
# false positive here -- randomizing the layout is the compiler taking the
# freedom those asserts describe. They stand down for this lane only.
#
# Its own target dir, so it does not clobber a normal build.
randomize-layout seed='1':
  @CARGO_TARGET_DIR=target/randlayout-{{ seed }} \
    RUSTFLAGS="-Zrandomize-layout -Zlayout-seed={{ seed }} --cfg randomized_layout" \
    cargo test --lib --tests

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

# Refuse a commit carrying generated output instead of source: a path that is
# gitignored yet staged anyway (i.e. force-added), a blob over 512 KB at a path
# that wasn't already that large, or more than 4 MB of newly added files in one
# commit. It names no specific artifact: .gitignore is where a shape gets
# named, and the force-added rule turns that list into a gate, so the fix for
# one that slips through is a .gitignore entry rather than a rule here.
#
# Runs as the first pre-commit hook (.gitconfig): it reads the index, costs
# milliseconds, and nothing else about a commit matters if it has a build
# directory in it.
#
# `--range <range>` asks the same questions of every blob a push would send,
# e.g. `just artifact-guard --range origin/main..HEAD`. It walks objects rather
# than diff endpoints, so an artifact that was committed and then deleted a few
# commits later still shows up -- deleting it never got it out of the pack.
artifact-guard *args:
  @scripts/artifact-guard.py {{ args }}

# Regenerate the ABI ledger (metrics/abi-ledger.jsonl): classifies every
# #[no_mangle] export by who resolves it by name. `--check` diffs against the
# committed ledger instead of writing.
abi-ledger *args:
  @scripts/abi-ledger.py {{ args }}

# Regenerate the visibility ledger (metrics/visibility-ledger.jsonl): every
# `pub` item whose only reacher from outside the crate is an integration test
# under crates/nvim/tests, naming the test. The ABI ledger's question in its
# Rust form — a ported spec links the library from outside, so the entry point
# it drives cannot be narrowed, and nothing in the module says so. It reads the
# tests' `use` trees rather than the crate's `pub` items, so a path it cannot
# account for is an error instead of a quietly missing row. `--check` diffs
# against the committed ledger instead of writing.
visibility-ledger *args:
  @scripts/visibility-ledger.py {{ args }}

# Regenerate the ratchet baseline (metrics/ratchet.json): per-file
# unchecked-statement / static mut / #[no_mangle] / variadic / GlobalCell-ptr
# counts, file sizes (1k-line cap, current offenders grandfathered), the size
# of the crate's `pub` surface, and a count from each ledger may only shrink.
# It also writes the wrapper-callee table (metrics/wrapper-callees.tsv), the
# work order for the callee phases, and rewrites docs/perimeter.md's measured
# "Today" line. `--check` compares against the committed copies instead.
ratchet *args:
  @scripts/ratchet.py {{ args }}

# Regenerate every committed baseline, in the one order that is self-consistent:
# the generated wrappers and the keycode table, then format, then the two
# ledgers, then the ratchet, then re-check formatting. This is the entry point;
# running the pieces by hand invites a baseline that describes a tree that no
# longer exists.
#
# Code generation leads because it writes crate source every later step reads.
# The cdefs golden regenerates after formatting rather than with the other
# generators: it is derived from the crate source apigen just wrote, and it is
# not crate source itself, so nothing downstream reads it.
# Formatting comes next because rustfmt rewrapping a line changes the line counts
# the ratchet measures — and `fmt-check` (the pre-commit hook) rewrites the tree, so
# a baseline taken before it silently stops matching mid-commit. The ledgers
# precede the ratchet because the ratchet snapshots a count from each.
# The lint baseline comes last: it runs cargo clippy, by far the slowest step
# (a full check-mode compile when the tree changed), and depends on nothing
# the earlier steps write.
# The closing pass is uncached on purpose: cached, it would skip the files `fmt`
# just rewrote and prove nothing, where uncached it asserts formatting reached a
# fixed point that the pre-commit hook can't move.
#
# Args are forwarded to the ratchet and lint, e.g. `just refresh --allow-growth`.
refresh *args: apigen keycodes-lua fmt ffigen abi-ledger visibility-ledger (ratchet args) (lint args)
  @treefmt --no-cache --fail-on-change --quiet

# This is the gate CI runs on every push. It deliberately skips the slow
# suites, which are worth invoking directly (`just functionaltest`,
# `just oldtest`, ...); only the fast Rust-side tests run here.
#
# Check that the tree is formatted, every generator still reproduces its
# committed output, both ledgers are current and the ratchet holds, the
# generators and the crate compile clean, and the safe-core tests pass.
# fmt-check leads because it rewrites the tree. The ledger checks precede the
# ratchet check because the ratchet snapshots a count from each and cannot tell
# a stale ledger from a fresh one (the ABI ledger and the ratchet also run as
# pre-commit hooks, see .gitconfig). lint-tools is here rather than in `lint`
# alone because it is seconds, where the crate's clippy pass is minutes.
# build-release follows the debug build because nothing else in the tree
# compiles with `debug_assertions` off: a `#[cfg(debug_assertions)]` block can
# leave an import or a helper unused in release, which `-D warnings` rejects,
# and that break once sat unnoticed for a whole phase. It costs ~40 s.
# `lint` and `doc` come last: the clippy pass is the slowest step here by
# minutes, and rustdoc's ~10 s wants the crate already compiled. `lint-tools`
# stays listed on its own so the seconds-long generator lint still runs before
# anything expensive; just runs a recipe once per invocation, so naming it
# twice costs nothing.
minimal-ci: fmt-check (apigen "--check") (ffigen "--check") (keycodes-lua "--check") (abi-ledger "--check") (visibility-ledger "--check") (ratchet "--check") lint-tools build build-release cargo-test lint doc
