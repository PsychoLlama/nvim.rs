Plan: ~/.claude/plans/eager-gliding-sifakis.md

The plan has been executed successfully. Here's a summary of what was accomplished:

## Summary

Successfully migrated cursor positioning functions from C to Rust:

### Functions Migrated (~450 lines of C → ~770 lines of Rust)
1. **`scroll_cursor_halfway`** - Handles the `zz` command (position cursor in middle of window)
2. **`scroll_cursor_top`** - Handles the `zt` command (position cursor at top with scrolloff)
3. **`scroll_cursor_bot`** - Handles the `zb` command (position cursor at bottom with scrolloff)
4. **`set_empty_rows`** - Helper for computing empty rows at window bottom

### C Accessor Functions Added
- `nvim_win_get_filler_rows` / `nvim_win_set_filler_rows` (window.c)
- `nvim_win_get_botfill` / `nvim_win_set_botfill` (window.c)
- `nvim_get_mouse_dragging` (mouse.c)
- `nvim_validate_botline`, `nvim_plines_win_full` (move.c)

### Verification Completed
- ✅ `just build` - Full build with linking passes
- ✅ `just rust-fmt-check` - Formatting clean
- ✅ `just rust-clippy` - No lint warnings  
- ✅ `just rust-test` - All 3672 unit tests pass
- ✅ `test_move` oldtest - Passes

Note: The `test_scroll_opt.vim` test has pre-existing failures unrelated to this migration (the same failures occur on the base branch before these changes).
