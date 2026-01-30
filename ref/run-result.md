Plan: ~/.claude/plans/sunny-foraging-nest.md

The plan has been successfully executed. Here's a summary of what was accomplished:

## Summary

I successfully migrated the page scroll functions from C to Rust as specified in the plan:

### Functions Migrated

1. **`cursor_correct()`** - Corrects cursor position to stay within scrolloff bounds when scrolling
2. **`get_scroll_overlap()`** - Calculates optimal overlap (0-2 lines) for page-up/page-down scrolling to maintain symmetric behavior
3. **`scroll_with_sms()`** - Handles smooth scrolling during page scroll operations, temporarily enabling smoothscroll and adjusting for partially visible lines
4. **`pagescroll()`** - Main page scrolling function handling CTRL-F/B (full page) and CTRL-D/U (half page) commands

### New C Accessor/Wrapper Functions Added

To support the Rust implementation, I added several C accessor and wrapper functions:
- `nvim_win_set_p_sms()` - Setter for smoothscroll option
- `nvim_cursor_down_inner()` / `nvim_cursor_up_inner()` - Cursor movement helpers
- `nvim_nv_screengo()` - Screen-based cursor movement
- `nvim_beginline_flags()` - Move cursor to beginning of line
- `nvim_beep_flush_wrapper()` - Error notification
- `nvim_nv_g_home_m_cmd()` - Move to first non-blank with 'sol'
- `nvim_one_window()` - Check if single window
- `nvim_get_p_sol()` - Get 'startofline' option
- `nvim_win_get_p_scr()` / `nvim_win_set_p_scr()` - Window scroll option
- `nvim_plines_correct_topline()` - Screen line calculation
- `nvim_plines_m_win()` - Multi-line screen calculation

### Verification

All checks passed:
- `just build` - Full build with linking ✓
- `just rust-fmt-check` - Formatting ✓
- `just rust-clippy` - Lints ✓
- `just rust-test` - All 3712 unit tests pass ✓
