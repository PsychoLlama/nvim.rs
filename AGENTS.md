# Project

- A fork of neovim ported to Rust using `c2rust`.
- Original C code exists under git tag `v0.12.4`.
- No backward/forward compat guarantees. Breaking changes allowed.

# Testing

- `just functionaltest [paths...] [--filter=pat]` — RPC-driven tests from `test/functional`. Pass spec files/dirs to run a subset.
- `just oldtest [test_name]` — legacy Vim tests from `test/old/testdir`, e.g. `just oldtest test_arglist`.
- `just benchmark [paths...]` — benchmarks from `test/benchmark`.
- Upstream's unit tests (`test/unit`) are defunct: they FFI'd into C headers that the port no longer generates.
- Test runs are verbose; redirect to a log file and grep the summary.
