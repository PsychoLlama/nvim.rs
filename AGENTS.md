# Project

- A fork of neovim ported to Rust using `c2rust`.
- Original C code exists under git tag `v0.12.4`.
- No backward/forward compat guarantees. Breaking changes allowed.

# Developing

- Tests and builds are verbose; redirect to a log file and grep if failed.
- Pre-commit hooks (`$REPO/.gitconfig`) validate formatting, `abi-ledger`, and ratcheted metrics.
- Note big changes in `CHANGELOG.md`. It should not be exhaustive: only a hint about which release carried a sweeping change over a large feature. Small-to-mid size changes to not belong in the changelog.
- Transpilation inherited original authors' terse naming. Don't match for parity's sake. When appropriate, use clear names.
- When rewriting vendored code, keep the upstream copyright/license notice in the ported file and keep `LICENSE.txt` accurate.

# Ratchet

- The ratchet (`just ratchet --check`) constrains per-file unchecked lines (code inside `unsafe` regions), `static mut`/`#[no_mangle]`/variadic/C-ABI-signature counts, `GlobalCell` raw escape-hatch uses (`.ptr()`/`.as_raw()`), `unsafe fn`s with no `# Safety` section, file line counts, and internal exports to only shrink — plus the count of files not carrying `forbid(unsafe_code)`, `deny(unsafe_op_in_unsafe_fn)` or the cast deny.
- An `unsafe fn` body scores as unchecked _in full_ until its file carries `deny(unsafe_op_in_unsafe_fn)`. Land the deny first to see a file's true count, and take a `mod.rs` last — a lint attribute there reaches the whole subtree.
- The cast deny is the per-module opt-in to clippy's cast family; a module that has finished its casts writes `#![deny(clippy::cast_lossless, clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss, clippy::ptr_as_ptr)]` and the count of files without it may only fall. Adding it to a file that is not ready makes `just lint` fail outright, which is the point.
- Narrowing an `unsafe` region is progress even when it adds regions; splitting a transpiled body into functions with tight regions lowers the count. Blank and comment-only lines inside a region are free, so SAFETY notes cost nothing.
- Lines inside a `#[cfg(test)] mod … { … }` are exempt from the 1,000-line file cap, so tests can sit next to the code they cover. Every other metric still counts them — unchecked code in a test is still unchecked code.
- `metrics/clippy.json`'s per-file warning counts cover `crates/*/tests` as well as `crates/*/src` (clippy runs `--all-targets`), so a new warning in a *test* file fails `just lint`. Only its two posture totals (`unreachable_pub`, `unused_qualifications`) are restricted to `crates/*/src`, and `metrics/ratchet.json` never looks at `tests/` at all.
- After reducing any of them, run `just refresh` and commit the refreshed `metrics/*.{json,jsonl}`.
- Justified growth requires `just refresh --allow-growth`.

# Generators

- `tools/apigen` and `tools/ffigen` live outside the workspace; `just lint-tools` clippies them at `-D warnings` (baseline: zero) and `just fmt-check` already reaches them.
- Never hand-edit generated output. `just apigen`/`just ffigen` rewrite it; the `--check` form of each fails on drift and runs in `just minimal-ci`.
- `tools/ffigen/unit-cdefs.h` is a committed golden, not an input: the unit suite regenerates its own copy under `target/ffi`.

# Testing

Tests only run local, not in CI. Any failure, even out of scope, is your responsibility to fix.

- `just functionaltest [paths...] [--filter=pat]` — RPC-driven tests from `test/functional`. Pass spec files/dirs to run a subset.
- `just oldtest <all|test_name...|clean>` — legacy Vim tests from `test/old/testdir`. The mode is required; `all` runs the whole suite from scratch, names run a subset, e.g. `just oldtest test_arglist`.
- `just unittest [paths...]` — unit tests from `test/unit`. Their FFI declarations are generated from the Rust crate (`tools/ffigen`, wired in by `scripts/gen-unit-cdefs.sh`); the specs call the exported symbols of the nvim binary, and the C fixtures (`unit-fixtures.so`) compile against the same generated declarations.
- Run one suite at a time: the harnesses share `target/` scaffolding and interfere when run concurrently.
- `just asan <recipe>` — AddressSanitizer equivalents (`build`, `functionaltest`, `oldtest`), defined in `just/asan.just`. Reports land in `target/asan/asan.log.<pid>`.
- `just benchmark [paths...]` — benchmarks from `test/benchmark`.
