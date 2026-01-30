Plan: ~/.claude/plans/agile-strolling-turing.md

The plan is complete. Here's a summary:

## Summary

I successfully executed the plan to complete the move.c Rust migration. Here's what was done:

### Phase 1: C Accessor Functions
Added accessor functions in:
- `fold.c`: `nvim_wline_get_size()`, `nvim_wline_get_lastlnum()`
- `drawscreen.c`: `nvim_redrawing()`, `nvim_win_scroll_lines()`
- `edit.c`: `nvim_get_dollar_vcol()`, `nvim_set_dollar_vcol()`
- `option.c`: `nvim_get_p_ss()`
- `window.c`: `nvim_win_grid_has_target()`

### Phase 2: Implemented `rs_curs_rows()`
Migrated the ~70-line `curs_rows()` function which:
- Validates w_lines[] cache state
- Iterates w_lines[] entries to compute w_cline_row
- Handles folded/concealed lines
- Sets w_cline_height and w_cline_folded

### Phase 3: Implemented `rs_curs_columns()`
Migrated the ~235-line `curs_columns()` function which:
- Computes cursor column and wcol
- Handles line wrapping with width1/width2 calculations
- Manages horizontal scrolling (leftcol, sidescrolloff)
- Handles smooth scrolling (skipcol adjustments)
- Calls win_scroll_lines for screen updates

### Phase 4: Converted C Functions
Converted both `curs_rows()` and `curs_columns()` in `move.c` to thin wrappers that call the Rust implementations.

### Phase 5: Verification & Commit
All checks passed:
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- `just rust-test` ✓ (3712 tests passed)

Created 2 commits:
1. `feat(move): add C accessor functions for curs_rows/curs_columns`
2. `feat(move): migrate curs_rows and curs_columns to Rust`

### Results
- Added ~520 lines of Rust code
- Removed ~296 lines of C logic
- Total `#[no_mangle]` function count: **10,280**
- move.c now contains only thin wrappers (no logic except type conversion)
