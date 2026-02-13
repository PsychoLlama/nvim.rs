Plan: ~/.claude/plans/ticklish-booping-sun.md

All 8 phases of the popupmenu.c → Rust migration plan are complete. Here's the summary:

| Phase | Functions | Commit | Approach |
|-------|-----------|--------|----------|
| 1 | `pum_border_width`, `pum_align_order`, `pum_get_item`, `pum_user_attr_combine`, `pum_compute_size` | `dbf446658c` | Real Rust logic |
| 2 | `pum_compute_vertical_placement`, `set_pum_width_aligned_with_cursor`, `pum_compute_horizontal_placement` | `7c509c9bc0` | Real Rust logic (wiring) |
| 3 | `pum_recompose`, `pum_check_clear`, `pum_set_event_info`, `pum_ui_flush` | `45f9c6b21c` | C `_impl` accessor pattern |
| 4 | `pum_compute_text_attrs`, `pum_grid_puts_with_attrs` | `807e4ad828` | C `_impl` accessor pattern |
| 5 | `pum_preview_set_text`, `pum_adjust_info_position`, `pum_set_info` | `92fbae5ddc` | C `_impl` accessor pattern |
| 6 | `pum_position_at_mouse`, `pum_select_mouse_pos`, `pum_execute_menu`, `pum_show_popupmenu`, `pum_make_popup` | `8d50bf7dcd` | C `_impl` accessor pattern |
| 7 | `pum_redraw`, `pum_set_selected` | `db6599424d` | C `_impl` accessor pattern |
| 8 | `pum_display` | `141abca179` | C `_impl` accessor pattern |

**What was done:**
- All 25 remaining C functions in `popupmenu.c` now have thin `rs_*` wrappers
- Simple functions (Phases 1-2) got real Rust implementations with existing Rust helpers
- Complex functions (Phases 3-8) used the `_impl` accessor pattern — original C logic moved to `nvim_pum_*_impl()` functions, Rust FFI exports delegate to them
- 31 `rs_*` extern declarations in `popupmenu.c` (25 new + 6 pre-existing)
- 111 `#[no_mangle]` Rust exports across the popupmenu crate
- All validation passed on every commit: build, smoke-test, rust-fmt-check, rust-clippy, rust-test (4155 tests)
