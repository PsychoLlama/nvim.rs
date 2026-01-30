Plan: ~/.claude/plans/giggly-sparking-yao.md

All tasks are complete. Here's a summary of what was accomplished:

## Summary

Successfully migrated three core scroll functions from `src/nvim/move.c` to the Rust `src/nvim-rs/move` crate:

### Functions Migrated
1. **`rs_scrolldown()`** (~161 lines) - Scrolls window down (CTRL-Y)
2. **`rs_scrollup()`** (~109 lines) - Scrolls window up (CTRL-E)
3. **`rs_scroll_redraw()`** (~50 lines) - Orchestrates scroll with cursor adjustment

### C Accessor Functions Added
- **window.c**: `nvim_win_set_wrow`, `nvim_win_get_p_sms`
- **move.c**: `nvim_cursor_correct`, `nvim_cursor_correct_sms`, `nvim_validate_cursor_win`, `nvim_validate_virtcol`, `nvim_validate_cheight`, `nvim_check_topfill`, `nvim_invalidate_botline`, `nvim_win_col_off`, `nvim_win_col_off2`
- **plines.c**: `nvim_linetabsize_eol`, `nvim_plines_win`, `nvim_win_may_fill`
- **edit.c**: `nvim_scroll_cursor_up`, `nvim_scroll_cursor_down`

### Features Preserved
- Smooth scroll ('sms' option) with width calculations
- Filler lines (diff mode)
- Folded lines via `hasFolding()`
- Concealed lines via `decor_conceal_line()`
- Cursor repositioning when scrolling off-screen
- 'scrolloff' handling for cursor position

### Verification
- ✅ `just build` - full build with linking
- ✅ `just rust-fmt-check` - formatting
- ✅ `just rust-clippy` - lints
- ✅ `just rust-test` - 3672 unit tests passed
