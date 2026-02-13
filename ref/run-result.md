Plan: ~/.claude/plans/expressive-sprouting-breeze.md

All 5 `_impl` functions in statusline.c have been rewritten as real Rust implementations and the C originals deleted:

| Phase | Commit | Function | C Lines Deleted |
|-------|--------|----------|-----------------|
| 1 | `ba264ee` | ui_ext_tabline_update_impl | ~45 |
| 2a+2b | `8a40e22` | build_stl_str_hl_impl (port) | ~1500 |
| 2c | `5399f01` | build_stl_str_hl_impl (delete C) | — |
| 3 | `8bf6ea6` | win_redr_custom_impl | ~100 |
| 4 | `8bb5843` | redraw_ruler_impl | ~130 |
| 5 | `7ddd7c4` | draw_tabline_impl | ~170 |

statusline.c reduced from 2,332 → 1,373 lines (~960 lines of real C logic deleted). All checks pass: build, smoke-test.
