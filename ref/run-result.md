Plan: ~/.claude/plans/drifting-herding-whale.md

All tasks are complete. Here's a summary of what was accomplished:

## Summary

I successfully executed the plan to complete the charset.c migration to Rust. Here's what was done:

### Changes Made

**Phase 1: Accessor Function**
- Added `nvim_charset_get_cursor_line_ptr()` in `charset.c` to expose the cursor line pointer to Rust

**Phase 2: getwhitecols_curline Migration**
- Added `rs_getwhitecols_curline()` in Rust that calls the accessor and delegates to existing `rs_getwhitecols()`
- Updated C wrapper to use the Rust function

**Phase 3: transchar_buf Migration**
- Added `rs_transchar_buf()` in Rust which implements the character translation logic:
  - Handles special keys (negative values) with `~@` prefix
  - Handles printable ASCII characters
  - Delegates to `rs_transchar_nonprint()` for non-printable chars
  - Delegates to `rs_transchar_hex()` for multibyte chars (> 0xFF)
- Updated C `transchar_buf()` to be a thin wrapper calling Rust

**Phase 4: transchar_byte_buf Migration**
- Added `rs_transchar_byte_buf()` in Rust for byte translation
- Bytes >= 0x80 are treated as non-printable and go through `rs_transchar_nonprint()`
- Lower bytes delegate to `rs_transchar_buf()`
- Updated C `transchar_byte_buf()` to be a thin wrapper calling Rust

**Phase 5: kv_transstr Verification**
- Verified `kv_transstr` stays in C as it uses kvec_t macros (C-only data structure)
- It already delegates the actual translation to Rust via `transstr_len` and `transstr_buf`

**Phase 6: Verification**
- ✅ `just build` - passed
- ✅ `just rust-fmt-check` - passed
- ✅ `just rust-clippy` - passed  
- ✅ `just rust-test` - 3717 tests passed
- ✅ Manual verification of character translation working correctly

### Files Modified
- `src/nvim/charset.c` - Added accessor, updated thin wrappers, added extern declarations
- `src/nvim-rs/charset/src/lib.rs` - Added `rs_getwhitecols_curline`, `rs_transchar_buf`, `rs_transchar_byte_buf`

### Commit
```
feat(charset): complete migration of transchar functions to Rust

- Add rs_getwhitecols_curline for cursor line whitespace
- Add rs_transchar_buf for character translation logic
- Add rs_transchar_byte_buf for byte translation
- Keep kv_transstr in C (kvec dependency)
- C retains static buffer management, Rust handles logic
```
