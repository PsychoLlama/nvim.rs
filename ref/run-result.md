Plan: ~/.claude/plans/mellow-riding-crown.md

All tasks are complete. Let me provide a summary of what was accomplished:

## Summary

I have successfully completed the plan to migrate the remaining plines.c functions to Rust. Here's what was done:

### Phase 1: Simple Function Migrations
- **`linetabsize`** - Now calls `rs_linetabsize`
- **`linetabsize_eol`** - Now calls `rs_linetabsize_eol`
- **`linetabsize_col`** - Added new `rs_linetabsize_col` function and `nvim_linetabsize_col` C wrapper for init_charsize_arg

### Phase 2: Virtual Column Functions
- **`getvcol_nolist`** - Now calls `rs_getvcol_nolist`
- **`getvvcol`** - Now calls `rs_getvvcol`
- **`getvcols`** - Now calls `rs_getvcols`

### Phase 3: Complex Function Migration
- **`win_text_height`** (68 lines) - Added `rs_win_text_height` with full folding and filler line support. Now the C function is just a one-line wrapper.

### Phase 4: Cleanup
- Removed `charsize_fast_impl` static function (duplicated in Rust)
- Removed `virt_text_cursor_off` static function (already has `nvim_virt_text_cursor_off`)

### Verification
All checks pass:
- ✅ `just build` - Full build completes successfully
- ✅ `just rust-fmt-check` - Code formatting passes
- ✅ `just rust-clippy` - No clippy warnings
- ✅ `just rust-test` - All 3712 unit tests pass

### Result
- **plines.c** now contains only thin wrappers calling `rs_*` functions, accessor functions for Rust FFI, and `init_charsize_arg` (kept in C due to marktree iterator complexity)
- **plines Rust module** grew by ~150 lines with the new implementations
- The commit `318783452a` documents all changes
