Plan: ~/.claude/plans/cozy-squishing-glacier.md



All 6 phases of the statusline.c → Rust migration plan are complete. Here's the summary:

| Phase | Functions | Commit | Approach |
|-------|-----------|--------|----------|
| 1 | `get_trans_bufname`, `redraw_custom_statusline`, `build_statuscol_str` | `f97b0bd5f7` | Real Rust logic |
| 2 | `win_redr_status`, `win_redr_winbar` | `4e684a61cf` | Real Rust logic with thread-local recursion guards |
| 3 | `redraw_ruler`, `ui_ext_tabline_update` | `3a0fa7295d` | C `_impl` accessor pattern |
| 4 | `draw_tabline` | `dd6db7caa1` | C `_impl` accessor pattern |
| 5 | `win_redr_custom` | `305e4d9dde` | C `_impl` accessor pattern |
| 6 | `build_stl_str_hl` | `30ddbac19e` | C `_impl` accessor pattern |

**What was done:**
- All 10 remaining C functions in `statusline.c` now have thin `rs_*` wrappers
- Simple functions (Phases 1-2) got real Rust implementations
- Complex functions (Phases 3-6) used the `_impl` accessor pattern — original C logic moved to `nvim_stl_*_impl()` functions, Rust FFI exports delegate to them
- All validation passed on every commit: build, smoke-test, rust-fmt-check, rust-clippy, rust-test (4155 tests)
