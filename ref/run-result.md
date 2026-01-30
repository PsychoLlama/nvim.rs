Plan: ~/.claude/plans/warm-shimmying-kazoo.md

The plan has been executed with what can be reasonably migrated in the current session. Let me provide a summary of what was accomplished:

## Summary

I have executed the migration plan for move.c to Rust as much as feasible. Here's what was accomplished:

### Completed Migrations

**Phase 1 - Foundation Functions (3 functions migrated):**
- `redraw_for_cursorline()` → `rs_redraw_for_cursorline()`
- `redraw_for_cursorcolumn()` → `rs_redraw_for_cursorcolumn()`
- `plines_correct_topline()` → `rs_plines_correct_topline()`

**Phase 2 - Validation Functions (5 functions migrated):**
- `validate_virtcol()` → `rs_validate_virtcol()`
- `validate_cheight()` → `rs_validate_cheight()`
- `validate_botline()` → `rs_validate_botline()`
- `validate_cursor()` → `rs_validate_cursor()`
- `update_curswant()` / `update_curswant_force()` - Updated existing Rust functions to call `rs_validate_virtcol()`

**Phase 3 - Core Computation (1 of 3 functions migrated):**
- `comp_botline()` → `rs_comp_botline()` - Computes w_botline from w_topline

**Commits made:**
1. `feat(move): migrate foundation functions to Rust`
2. `feat(move): migrate validation functions to Rust`
3. `feat(move): migrate comp_botline to Rust`

### Functions Deferred (require extensive new accessors)

These functions access the `w_lines[]` array or use complex iteration/globals:
- `curs_rows()` - Needs `w_lines[]` array accessors
- `curs_columns()` - Needs `w_lines[]` array accessors, 200+ lines
- `update_topline()` - Complex coordinator, 200+ lines
- `textpos2screenpos()` - Needs multiple helpers
- `virtcol2col()` - Needs vcol2col wrapper
- `do_check_cursorbind()` - Needs window iteration

### Stats
- Started with 54 `#[no_mangle]` functions
- Now have 62 `#[no_mangle]` functions
- Added 8 new Rust functions for this migration

All verification checks pass:
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- `just rust-test` ✓ (3712 tests passed)
