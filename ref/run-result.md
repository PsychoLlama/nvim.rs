Plan: ~/.claude/plans/graceful-cuddling-wren.md

Phase 17 is complete. Here's a summary of what was accomplished:

## Summary

**Phase 17: BT Regex Opcode Migration - Complete C Fallback Removal**

### Completed Tasks:

1. **Phase 17a - Word Boundaries and Character Classes** (commit 413ad90c2c):
   - Migrated BOW, EOW, IDENT, SIDENT, KWORD, SKWORD, FNAME, SFNAME, PRINT, SPRINT opcodes to Rust
   - Added FFI wrappers for `mb_get_class_tab()`, `vim_isIDc()`, `vim_iswordp_buf()`, `vim_isfilec()`, `vim_isprintc()`

2. **Phase 17b - Exact Match Opcodes** (commit f59cc1ad93):
   - Migrated EXACTLY, ANYOF, ANYBUT, MULTIBYTECODE, RE_COMPOSING opcodes to Rust
   - Added FFI wrappers for `utf_composinglike()`, `utf_iscomposing_legacy()`, `utf_ptr2len()`

3. **Phase 17c - Submatch Markers and Backreferences** (commit 85b1740f49):
   - Migrated MOPEN+0..9, MCLOSE+0..9, NOPEN, NCLOSE, BACKREF+1..9, ZREF+1..9, BACK opcodes to Rust
   - Added `get_zref_num()` helper function
   - Created `nvim_match_with_backref` wrapper for multi-line backref matching

4. **Phase 17d - Control Flow Opcodes** (commit 28ae16074f):
   - Migrated BRANCH, STAR, PLUS, BRACE_SIMPLE, BRACE_LIMITS, BRACE_COMPLEX+0..9 opcodes to Rust
   - Added FFI functions for brace min/max/count management and regstar initialization

5. **Phase 17e - Look-around and Position Checks** (commit 0d956dde63):
   - Migrated NOMATCH, MATCH, SUBPAT, BEHIND, NOBEHIND, BHPOS, NEWL, CURSOR, RE_MARK, RE_VISUAL, RE_LNUM, RE_COL, RE_VCOL opcodes to Rust
   - Added FFI wrappers for position checking functions

6. **Phase 17f - Complete State Machine**:
   - Verified all RS_* states have handlers in `rs_regmatch_full()`
   - State machine was already complete from previous phases

7. **Phase 17g - Delete C Fallback** (commit 17054af068):
   - Removed `nvim_bt_match_op()` (632 lines) - the C fallback dispatcher
   - Removed `regmatch()` (1,188 lines) - the original C matching function
   - Removed `nvim_bt_regmatch()` (4 lines) - the wrapper for Rust
   - **Total: 1,845 lines of C code deleted**

### Verification:
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- `just rust-test` ✓ (3718 tests)

### Results:
- `regexp.c` reduced from 13,519 lines to 11,674 lines (-1,845 lines)
- All BT regex opcodes are now handled natively in Rust
- No remaining calls to C fallback functions from Rust
- The C `regmatch()` function is fully replaced by Rust's `rs_regmatch_full()`

The BT regex engine's core matching loop is now 100% in Rust.
