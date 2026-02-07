## Revert: Back out regexp Rust migration

The regexp Rust migration has been reverted due to persistent bugs (infinite loops, crashes on startup and file open). The entire `regexp` crate has been removed and `regexp.c` restored to its original upstream state.

### Changes
- Restored `src/nvim/regexp.c` to merge-base (upstream) state (16,262 lines)
- Deleted `src/nvim-rs/regexp/` crate entirely (28 Rust source files)
- Removed `nvim-regexp` from workspace Cargo.toml, crate Cargo.toml, lib.rs, justfile
- Updated `src/nvim-rs/search/` to call C `skip_regexp_ex` and `vim_regcomp_had_eol` directly
- Updated `src/nvim-rs/ex_docmd/` to call C `skip_regexp` directly
- Removed stale comment in `src/nvim-rs/message/src/keys.rs`
- Removed entry from `ref/crates.md`

### Verification
- `just build` passes
- `just smoke-test` passes
- `just rust-fmt-check` passes
- `just rust-clippy` passes
- `just rust-test` passes (3434 tests)
- Regex substitution confirmed working
