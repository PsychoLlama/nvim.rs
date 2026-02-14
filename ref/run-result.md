Plan: ~/.claude/plans/ancient-plotting-scroll.md

All 7 phases of the mouse.c migration are complete. Here's the summary:

## Completed Migration Summary

| Phase | Functions | Approach |
|---|---|---|
| 1 | `set_mouse_topline`, `setmouse`, `move_tab_to_mouse`, `mouse_tab_close` | Full Rust implementations |
| 2 | `scroll_line_len`, `find_longest_lnum`, `do_mousescroll_horiz` | Full Rust implementations |
| 3 | `mouse_comp_pos`, `vcol2col` | `mouse_comp_pos` full Rust; `vcol2col` delegates to C accessor (charsize infrastructure) |
| 4 | `mouse_find_grid_win`, `mouse_find_win_inner`, `mouse_find_win_outer` | Rust orchestrates; frame tree + grid resolution in C accessors |
| 5 | `mouse_check_grid`, `get_fpos_of_mouse`, `do_popup`, `do_mousescroll`, `nv_mousescroll`, `ins_mouse`, `ins_mousescroll` | `get_fpos_of_mouse` full Rust; others delegate to C accessors |
| 6 | `jump_to_mouse` | Delegates to C accessor (352 lines, 8 static vars) |
| 7 | `do_mouse`, `nv_mouse` | Delegates to C accessors (619 lines) |

**Final state of `mouse.c`:** Only thin `rs_` wrapper calls, C accessor functions, and the two permanent C functions (`f_getmousepos`, `call_click_def_func`).

**Rust crate:** 34 `#[no_mangle]` exports in `src/nvim-rs/mouse/src/lib.rs`.

**All checks pass:** build, smoke-test, rust-fmt-check, rust-clippy, rust-test (4239 tests).
