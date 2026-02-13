Plan: ~/.claude/plans/dazzling-zooming-rose.md

All 3 phases of the `testing.c` remaining migration are complete:

| Phase | Description | Commit |
|-------|-------------|--------|
| 1 | assert_equalfile + assert_append_cmd_or_arg | `bc6c4803c1` |
| 2 | f_assert_fails + C accessors + _Static_assert guards | `edbcb0ab25` |
| 3 | Cleanup dead C code | (this commit) |

`testing.c` now contains only:
- Thin `f_*` wrappers (all delegate to `rs_*`)
- `nvim_testing_*` C accessor functions
- `nvim_testing_fill_dict_diff` (dict hashtab traversal kept in C)
- `ga_concat_esc` / `ga_concat_shorten_esc` (used by `fill_dict_diff`)
- Error string constants + `_Static_assert` guards

15 `#[no_mangle]` exports in `viml_assert.rs` (was 13, added `rs_f_assert_fails` + `rs_f_assert_equalfile`).

All checks pass: build, smoke-test, rust-fmt-check, rust-clippy, and rust-test (4106 tests).
