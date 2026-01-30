Plan: ~/.claude/plans/polished-booping-ember.md

## Summary

Successfully completed the charset module migration to Rust, transferring g_chartab ownership and migrating chartab initialization and string translation functions.

### Changes Made

**Phase 1 - C Accessor Functions (`src/nvim/charset.c`)**:
- Added `nvim_charset_get_p_isi()` - returns 'isident' option value
- Added `nvim_charset_get_p_isp()` - returns 'isprint' option value
- Added `nvim_charset_get_p_isf()` - returns 'isfname' option value
- Added `nvim_charset_get_buf_p_isk()` - returns buffer 'iskeyword' value
- Added `nvim_charset_get_dy_flags()` - returns display flags
- Added `nvim_charset_get_buf_lisp()` - returns buffer lisp mode flag
- Added `nvim_charset_mb_ptr2char_adv()` - advance pointer and get UTF char
- Added `nvim_charset_mb_islower/isupper()` - character case checks
- Added `nvim_charset_get_g_chartab()` - returns g_chartab pointer
- Added `nvim_charset_is_initialized()` - returns chartab init status

**Phase 2 - Chartab Initialization (`src/nvim-rs/charset/src/lib.rs`)**:
- Implemented `rs_buf_init_chartab()` - initializes global and buffer chartab
- Implemented `rs_check_isopt()` - validates option string format
- Implemented `rs_parse_isopt()` - parses iskeyword/isident/isfname/isprint options
- Handles ranges (e.g., "48-57"), exclusions (^), and @ for alphabetic chars

**Phase 3 - String Translation**:
- Implemented `rs_trans_characters()` - in-place string translation
- Implemented `rs_transstr_buf()` - translate string to buffer
- Added `translate_byte()` helper for single-byte translation
- Added `translate_nonprint()` helper for non-printable representation

**Phase 4 - Character Translation** (partial):
- Core translation logic (`transchar_nonprint`, `transchar_hex`) already in Rust
- C wrappers retained for special key handling and static buffer management

**Phase 5 - g_chartab Ownership**:
- Defined `pub static mut g_chartab: [u8; 256]` in Rust with #[no_mangle]
- Changed C definition to `extern uint8_t g_chartab[256]`
- C code continues accessing via same symbol name

**Phase 6 - Cleanup**:
- Removed unused `SET_CHARTAB`, `RESET_CHARTAB`, `GET_CHARTAB_TAB` macros
- Removed `CT_*` flag definitions from C (now constants in Rust)

### Commits
1. `3c982e396c` - feat(charset): add C accessor functions for Rust FFI
2. `ccb1af50e3` - feat(charset): migrate chartab initialization to Rust
3. `5109bed266` - feat(charset): migrate string translation functions to Rust
4. `4bebb6c395` - feat(charset): move g_chartab ownership to Rust
5. `16092b12f7` - chore(charset): remove unused chartab macros

### Verification
- ✅ `just build` - passes
- ✅ `just rust-fmt-check` - passes
- ✅ `just rust-clippy` - passes
- ✅ `just rust-test` - 3671 tests pass
