Plan: ~/.claude/plans/ticklish-booping-sun.md

All 8 phases of the `popupmenu.c` → Rust migration are complete. Here's the summary:

| Phase | Functions | Commit |
|-------|-----------|--------|
| 1 | `pum_border_width`, `pum_align_order`, `pum_get_item`, `pum_user_attr_combine`, `pum_compute_size` | `dbf446658c` |
| 2 | `pum_compute_vertical_placement`, `set_pum_width_aligned_with_cursor`, `pum_compute_horizontal_placement` | `7c509c9bc0` |
| 3 | `pum_recompose`, `pum_check_clear`, `pum_set_event_info`, `pum_ui_flush` | `45f9c6b21c` |
| 4 | `pum_compute_text_attrs`, `pum_grid_puts_with_attrs` | `807e4ad828` |
| 5 | `pum_preview_set_text`, `pum_adjust_info_position`, `pum_set_info` | `92fbae5ddc` |
| 6 | `pum_position_at_mouse`, `pum_select_mouse_pos`, `pum_execute_menu`, `pum_show_popupmenu`, `pum_make_popup` | `8d50bf7dcd` |
| 7 | `pum_redraw`, `pum_set_selected` | `db6599424d` |
| 8 | `pum_display` | `141abca179` |

**25 functions migrated** across 8 commits. All validations (build, smoke-test, fmt, clippy, 4155 unit tests) passed on every commit.
