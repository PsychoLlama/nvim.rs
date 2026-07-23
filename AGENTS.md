# Project

- A fork of neovim ported to Rust using `c2rust`.
- Original C code exists under git tag `v0.12.4`.
- No backward/forward compat guarantees. Breaking changes allowed.

# Developing

- Tests and builds are verbose; redirect to a log file and grep if failed.
- Pre-commit hooks (`$REPO/.gitconfig`) validate formatting, `abi-ledger`, and ratcheted metrics.
- Note big changes in `CHANGELOG.md`. It should not be exhaustive: only a hint about which release carried a sweeping change over a large feature. Small-to-mid size changes to not belong in the changelog.

# Ratchet

- The ratchet (`just ratchet --check`) constrains `unsafe `/`static mut`/`#[no_mangle]` counts, file line counts, and internal exports to only shrink.
- After reducing any of them, run `just refresh` and commit the refreshed `metrics/*.{json,jsonl}`.
- Justified growth requires `just refresh --allow-growth`.

# Testing

Tests only run local, not in CI. Any failure, even out of scope, is your responsibility to fix.

- `just functionaltest [paths...] [--filter=pat]` — RPC-driven tests from `test/functional`. Pass spec files/dirs to run a subset.
- `just oldtest <all|test_name...|clean>` — legacy Vim tests from `test/old/testdir`. The mode is required; `all` runs the whole suite from scratch, names run a subset, e.g. `just oldtest test_arglist`.
- `just unittest [paths...]` — unit tests from `test/unit`. Their FFI declarations are generated from the Rust crate (`tools/ffigen`, wired in by `scripts/gen-unit-cdefs.sh`); the specs call the exported symbols of the nvim binary, and the C fixtures (`unit-fixtures.so`) compile against the same generated declarations.
- Run one suite at a time: the harnesses share `target/` scaffolding and interfere when run concurrently.
- `just asan <recipe>` — AddressSanitizer equivalents (`build`, `functionaltest`, `oldtest`), defined in `just/asan.just`. Reports land in `target/asan/asan.log.<pid>`.
- Broken: `just benchmark [paths...]` — benchmarks from `test/benchmark`. We haven't gotten it working since the rewrite.
