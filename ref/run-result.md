## Previous: Regexp differential testing infrastructure added

After reverting the failed regexp migration, we built a testing foundation:

### What exists now
- `src/nvim-rs/test/regexp_patterns.txt` — 527 hand-curated pattern/input pairs
- `src/nvim-rs/test/regexp_corpus.json` — expected results from the C engine (re=0,1,2)
- `test/regexp_baseline.vim` — VimL script to regenerate the corpus via `just regexp-baseline`
- `src/nvim-rs/test/compare_regexp.c` — FFI comparison test for `skip_regexp` logic (24 tests, in `just rust-ffi-test`)
- `src/nvim-rs/test/shadow_regexp.md` — strategy guide for shadow mode migration
- `fuzz/` — cargo-fuzz skeleton targets (activate when regexp crate exists)
- `CLAUDE.md` — updated with regexp migration requirements

### Current state of regexp in the codebase
- `src/nvim/regexp.c` — full C implementation (16,262 lines), restored to upstream
- `src/nvim-rs/search/` — calls C `skip_regexp_ex` and `vim_regcomp_had_eol` via extern
- `src/nvim-rs/ex_docmd/` — calls C `skip_regexp` via extern
- No Rust regexp crate exists yet

### What was verified
- `just build`, `just smoke-test`, `just rust-test` (3434), `just rust-ffi-test` (24), `just rust-fmt-check`, `just rust-clippy` all pass
- `just regexp-baseline` generates 527 entries successfully
