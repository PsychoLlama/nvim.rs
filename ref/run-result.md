Plan: ~/.claude/plans/fix-abi-constants-vv-linenr-bool.md

All steps of the plan are complete. Here's a summary:

**Commit 1: `0aaf905` — fix(ffi): correct VV_* constants and LinenrT type across crates**
- Fixed `VV_EXCEPTION`: 16 → 30
- Fixed `VV_TESTING`: 35 → 76
- Fixed `VV_WARNINGMSG`: 38/39 → 4 (was also inconsistent between crates)
- Fixed `LinenrT`: `c_long`/`i64` → `i32` in change, drawline, extmark, undo crates (matches C `int32_t linenr_T`)
- Added `_Static_assert` guards for VV_* constants in `eval/vars.c`
- 8 files changed

**Commit 2: `a615054` — fix(ffi): correct bool/int type mismatches in FFI declarations**
- 6 functions: C returns `bool`, Rust had `-> c_int` → fixed to `-> bool`
- 10 functions: C returns `int`, Rust had `-> bool` → fixed to `-> c_int`
- 3 functions: C takes `int` param, Rust had `bool` → fixed to `c_int`
- Resolved 15 inconsistent cross-crate declarations
- 19 files changed

All checks pass: build, smoke-test (29/29), rust-fmt-check, rust-clippy, rust-test (3518/3518).
