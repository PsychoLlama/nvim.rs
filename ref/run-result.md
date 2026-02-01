Plan: ~/.claude/plans/lazy-scribbling-minsky.md

## Summary

I have successfully executed Phase 20 of the plan - migrating the NFA pattern parsing pipeline to Rust. Here's what was accomplished:

### Changes Made

1. **Updated `src/nvim-rs/regexp/src/nfa_parser.rs`** (+90 lines):
   - Added `skipchr_keepstart` import from scanner module
   - Added magic mode constants (`MAGIC_NONE`, `MAGIC_MODE_OFF`, `MAGIC_MODE_ON`, `MAGIC_ALL`)
   - Added regflags constants (`RF_ICASE`, `RF_NOICASE`, `RF_ICOMBINE`)
   - Added FFI declarations for `nvim_parse_set_reg_magic` and `nvim_parse_set_curchr`
   - Updated `nfa_regconcat()` to handle magic mode switches (`\Z`, `\c`, `\C`, `\v`, `\m`, `\M`, `\V`)
   - Added `is_magic()` helper function

2. **Updated `src/nvim/regexp.c`** (-345 lines):
   - Added extern declarations for Rust functions (`rs_nfa_regpiece`, `rs_nfa_regconcat`, `rs_nfa_regbranch`, `rs_nfa_reg`)
   - Replaced ~345 lines of C implementations with thin wrappers calling Rust

### Test Results
- ✅ `just build` - compilation successful
- ✅ `just smoke-test` - nvim starts without crashing
- ✅ `just rust-fmt-check` - formatting passes
- ✅ `just rust-clippy` - no lint warnings
- ✅ `just rust-test` - all 3,735 tests pass
- ✅ Manual regex tests with alternation, repetition, very magic, and case-insensitive patterns

### Line Count Impact
| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| regexp.c lines | ~9,372 | 9,027 | **-345** |
| nfa_parser.rs lines | ~1,573 | 1,663 | +90 |
| Net C reduction | - | - | **-345** |
