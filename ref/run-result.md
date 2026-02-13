Plan: ~/.claude/plans/dazzling-zooming-rose.md

## Summary

All 3 phases of the `testing.c` migration plan are complete across 3 commits:

| Phase | Commit | Description |
|-------|--------|-------------|
| 1 | `bc6c480` | Migrated `assert_equalfile` (byte-by-byte file comparison using `std::fs`) and `assert_append_cmd_or_arg` to Rust |
| 2 | `edbcb0a` | Migrated `f_assert_fails` (~145 lines, the most complex VimL assertion) to Rust. Added ~20 C accessor functions for global state, list operations, and error strings. Added `_Static_assert` guards for `VAR_LIST`, `VV_ERRMSG`, `FAIL` |
| 3 | `d6a026d` | Deleted all dead C helper functions: `prepare_assert_error`, `fill_assert_error`, `assert_equal_common`, `assert_match_common`, `assert_bool`, `assert_beeps`, `assert_inrange`, `assert_append_cmd_or_arg`, and the `assert_type_T` enum |

**`testing.c` now contains only:**
- Thin `f_*` wrappers (all delegate to `rs_*`)
- `nvim_testing_*` C accessor functions
- `nvim_testing_fill_dict_diff` (dict hashtab traversal kept in C)
- `ga_concat_esc` / `ga_concat_shorten_esc` (used by `fill_dict_diff`)
- Error string constants + `_Static_assert` guards

**All checks pass:** build, smoke-test, rust-fmt-check, rust-clippy, rust-test (4106 tests).
