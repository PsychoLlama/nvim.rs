Plan: ~/.claude/plans/stateful-noodling-map.md

All checks pass. The plan has been fully executed:

## Summary

Successfully migrated `update_topline()` from C to Rust, completing the move.c migration.

### Changes Made

**Phase 1: Added C Accessor Functions**
- `nvim_get_skip_update_topline()` in `src/nvim/move.c` - for checking the splitkeep skip flag
- `nvim_win_get_scbind_pos()` / `nvim_win_set_scbind_pos()` in `src/nvim/window.c` - for scroll binding position
- `nvim_win_buf_is_empty()` in `src/nvim/window.c` - for checking if window's buffer is empty

**Phase 2: Implemented `rs_update_topline()` in Rust**
- Added ~320 lines of Rust code in `src/nvim-rs/move/src/lib.rs`
- Faithfully ported the complex scrolling logic with proper handling of:
  - Skip update check for 'splitkeep'
  - Empty buffer special case
  - Topline adjustments (cursor above/near top)
  - Botline adjustments (cursor below/near bottom)
  - Concealed line handling
  - Smooth scrolling support

**Phase 3: Converted C Function to Thin Wrapper**
- Reduced `update_topline()` from ~208 lines to a 4-line wrapper calling `rs_update_topline()`

**Phase 4: Verification**
- ✅ `just build` - Full build with linking passes
- ✅ `just rust-fmt-check` - Formatting passes
- ✅ `just rust-clippy` - Lints pass
- ✅ `just rust-test` - All 3712 tests pass

**Phase 5: Created Commits**
1. `fba25eac6e feat(move): add C accessor functions for update_topline`
2. `1f738ed2f8 feat(move): migrate update_topline to Rust`
