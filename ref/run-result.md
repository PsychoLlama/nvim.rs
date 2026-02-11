Plan: ~/.claude/plans/buzzing-greeting-marble.md

All 5 phases of the eval.c migration plan are complete and committed:

| Phase | Functions | Commit |
|-------|-----------|--------|
| 1 | `string2float`, `char_from_string`, `string_slice` (+`char_idx2byte` private) | `da1aef3c45` |
| 2 | `get_env_len`, `get_id_len`, `to_name_end`, `find_name_end` | `990724f182` |
| 3 | `buf_byteidx_to_charidx`, `buf_charidx_to_byteidx`, `pattern_match` | `610f25c040` |
| 4 | `func_equal`, `callback_from_typval` | `671a2de080` |
| 5 | `set_ref_in_ht`, `set_ref_in_list_items`, `set_ref_in_item`, `set_ref_in_callback`, `set_ref_in_callback_reader` (+3 private helpers) | `1b3f54991c` |

**Total: ~21 functions migrated** (17 public `extern "C"` + 4 private helpers) across 5 new modules in the eval crate. All verification checks pass (build, smoke-test, fmt, clippy, rust-test with 4028 tests).
