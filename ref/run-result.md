Plan: ~/.claude/plans/frolicking-knitting-fairy.md

The plan has been executed. Here's a summary of what was accomplished:

## Summary of Executed Plan

### Phase 1: Lineoff Calculation Functions
- Already migrated in previous work (`topline_back`, `topline_back_winheight`, `botline_forw`)

### Phase 2: Scroll Clamping Functions (Commit: 6f433751ba)
- Migrated `scrolldown_clamp()` → `rs_scrolldown_clamp()`
- Migrated `scrollup_clamp()` → `rs_scrollup_clamp()`
- Migrated `cursor_correct_sms()` → `rs_cursor_correct_sms()`
- Migrated `adjust_skipcol()` → `rs_adjust_skipcol()`
- Added `nvim_curbuf_line_count()` accessor

### Phase 3: Cursor Positioning Core
- Kept `curs_rows()` in C due to heavy dependency on internal `w_lines[]` array structure

### Phase 4: Topline Management (Commit: db5abaa6ab)
- Migrated `set_topline()` → `rs_set_topline()`
- Added `nvim_win_set_topline_was_set()` and `nvim_win_get_topline_was_set()` accessors

### Phase 5: Screen Position Helpers (Commit: 829cfe4367)
- Migrated `set_valid_virtcol()` → `rs_set_valid_virtcol()`
- Added `nvim_win_set_virtcol()` accessor
- Added `nvim_redraw_for_cursorcolumn()` wrapper
- Kept `virtcol2col()` and `textpos2screenpos()` in C due to complex buffer access patterns

### Phase 6: VimL Function Helpers
- Kept in C as they depend on the complex functions from Phase 5

All builds pass, formatting is correct, clippy has no warnings, and all 3712 Rust tests pass.
