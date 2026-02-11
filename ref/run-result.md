Plan: ~/.claude/plans/tranquil-singing-wilkes.md

All 5 phases of the plan are complete. Here's the summary:

## Wave 2 Migration Complete — 31 functions migrated

| Phase | Functions | Status |
|---|---|---|
| 1 — Pure Calculations | 7 (`set_fraction`, `win_default_scroll`, `win_setheight`, `win_setwidth`, `min_rows`, `min_rows_for_all_tabpages`, `win_get_tabwin`) | Done |
| 2 — Option Validation | 4 (`did_set_winminheight`, `did_set_winminwidth`, `win_new_height`, `win_new_width`) | Done |
| 3 — Snapshot Lifecycle | 8 (`clear_snapshot`, `clear_snapshot_rec`, `make_snapshot`, `make_snapshot_rec`, `get_snapshot_curwin`, `get_snapshot_curwin_rec`, `check_snapshot_rec`, `restore_snapshot_rec`) | Done |
| 4 — Size Save/Restore | 6 (`win_size_save`, `win_size_restore`, `check_lnums_both`, `check_lnums`, `check_lnums_nested`, `reset_lnums`) | Done |
| 5 — Status Line Mgmt | 6 (`last_status`, `last_status_rec`, `win_remove_status_line`, `find_horizontally_resizable_frame`, `resize_frame_for_status`, `resize_frame_for_winbar`) | Done |

**New files created**: `resize/fraction.rs`, `resize/minsize.rs`, `resize/validate.rs`, `resize/save_restore.rs`, `state_validation.rs`, and `statusline.rs` (replaced placeholder).

**All checks pass**: build, smoke-test, rust-fmt-check, rust-clippy, rust-test (4014 tests).
