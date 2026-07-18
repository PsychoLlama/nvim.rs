# Project

- A fork of neovim ported to Rust using `c2rust`.
- Original C code exists under git tag `v0.12.4`.
- No backward/forward compat guarantees. Breaking changes allowed.

# Testing

- `just functionaltest [paths...] [--filter=pat]` — RPC-driven tests from `test/functional`. Pass spec files/dirs to run a subset.
- `just oldtest [test_name]` — legacy Vim tests from `test/old/testdir`, e.g. `just oldtest test_arglist`.
- `just benchmark [paths...]` — benchmarks from `test/benchmark`.
- `just unittest [paths...]` — unit tests from `test/unit`. They preprocess the upstream v0.12.4 C headers (reconstructed under `target/upstream` on first run) and FFI into the transpiled symbols the nvim binary exports.
- Test runs are verbose; redirect to a log file and grep the summary.
- Run one suite at a time: the harnesses share `target/` scaffolding and interfere when run concurrently.
