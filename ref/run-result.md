Plan: ~/.claude/plans/kind-sparking-snowflake.md

All steps complete. Here's a summary of the 3 commits made:

**Commit 1** (`7a763da`): `fix(testing): correct TYPVAL_SIZE and tv_equal ABI in Rust FFI`
- Fixed `TYPVAL_SIZE` from 24 to 16 (actual `sizeof(typval_T)`) in both `testing/viml_assert.rs` and `eval/funcs/dispatch.rs`
- Fixed `tv_equal` FFI declaration from `-> c_int` to `-> bool` (C returns `bool`, upper register bytes contained garbage)
- Added `_Static_assert` in `testing.c` to catch future sizeof changes
- **This was a pre-existing bug** that made all `assert_*` VimL functions silently broken

**Commit 2** (`f6e3065`): `fix(filesearch): fix NULL dereference in rs_vim_findfile for paths without wildcards`
- Changed `cbuf_to_string` to only return NULL for null data pointers, not zero-length strings
- Added `findfile` regression test to `test/regexp_smoke.vim` (now 29 tests)

**Commit 3** (`c827eb0`): `chore: add regexp-validate just target for regression detection`
- New `just regexp-validate` target that regenerates the corpus and checks for divergence

All checks pass: build, smoke-test (29/29), rust-fmt-check, rust-clippy, rust-test (3518/3518), regexp-validate.
