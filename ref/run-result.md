Plan: ~/.claude/plans/merry-strolling-pearl.md

All the work is committed. Here's a summary of what was accomplished.

## Summary

I have executed the plan at `/home/overlord/.claude/plans/merry-strolling-pearl.md`. Here's what was completed:

### Phase 24: Migrate State Save/Restore Functions ✓
- Implemented Rust versions of BT engine state management functions:
  - `rs_reg_save()` - saves current position for backtracking
  - `rs_reg_restore()` - restores position after backtrack
  - `rs_reg_save_equal()` - compares saved position with current
  - `rs_save_subexpr()` - saves subexpression match positions
  - `rs_restore_subexpr()` - restores subexpression match positions
- Added C accessor functions for Rust FFI access to C structs
- Updated C wrappers to call Rust implementations
- Removed original C implementations (~84 lines)
- **Commit**: `f3c2b2ab6a`

### Phase 25: Migrate failure_chance() ✓
- Discovered `failure_chance()` was already implemented in Rust (`rs_failure_chance` in `nfa_compiler.rs`)
- The C version was dead code since `nfa_postprocess()` calls `rs_nfa_postprocess()` which uses the Rust implementation
- Simply removed the 155-line dead C function
- **Commit**: `2444a485e9`

### Phase 26: Migrate BT Code Emission Helpers - SKIPPED
- Examined the BT code emission helpers (regc, regmbc, regtail, reginsert, etc.)
- These are ~220 lines and heavily used by the BT compiler still in C
- Skipped as MEDIUM risk requiring more extensive changes
- Best migrated when the entire BT compiler is migrated

### Phase 27: Clean Up Dead Code and Wrappers ✓
- Removed `#if 0` block containing `match_follows_OLD` (89 lines)
- This function was replaced by `rs_match_follows` in Rust
- Searched for other dead code - remaining wrappers are still needed for FFI
- **Commit**: `d87a6d2bd6`

### Results
- **Total C lines removed this session**: ~328 lines (from 8,739 to 8,480 - but note: 8,569 after phases 24-25, now 8,480 after phase 27)
- **regexp.c current size**: 8,480 lines
- **All verification steps pass**:
  - `just build` ✓
  - `just smoke-test` ✓
  - `just rust-test` (3,735 tests) ✓
  - `just rust-fmt-check` ✓
  - `just rust-clippy` ✓

### Commits Made
1. `f3c2b2ab6a` - refactor(regexp): migrate state save/restore functions to Rust (Phase 24)
2. `2444a485e9` - refactor(regexp): remove dead failure_chance() C function (Phase 25)
3. `d87a6d2bd6` - refactor(regexp): remove dead match_follows_OLD code (Phase 27)
