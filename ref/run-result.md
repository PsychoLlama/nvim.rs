Plan: ~/.claude/plans/radiant-munching-hopper.md

All phases of the drawscreen migration plan are complete. Here's the summary:

## Drawscreen Migration — Complete

**8 commits** on `rust-migration`:

| Commit | Description |
|--------|-------------|
| `7e0bd71` | Fix UPD_VALID test assertion (20→10) |
| `95af3f1` | Phase 1: `redrawing()`, `check_screensize()`, `cmdline_number_prompt()` |
| `6e75ffc` | Phase 2: `comp_col()` |
| `c33dce7` | Phase 3: `skip_showmode()`, `unshowmode()` |
| `5dcfde7` | Phase 4: `redraw_statuslines()`, `redraw_custom_title_later()` |
| `4f4e18e` | Phase 5: `conceal_check_cursor_line()`, `win_update_cursorline()` |
| `3f3c43e` | Phase 6: `setcursor()`, `setcursor_mayforce()` |
| `4b88932` | Phase 7: `start_search_hl()`, `end_search_hl()` |

**Stats**: 12 C functions migrated, 14 new `#[no_mangle]` FFI exports (81→95 total), ~40 new C accessor functions, 5 `_Static_assert` verifications.

**All checks passing**: `just build`, `just smoke-test`, `just rust-fmt-check`, `just rust-clippy`, `just rust-test` (3977 tests).
