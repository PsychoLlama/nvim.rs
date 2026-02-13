Plan: ~/.claude/plans/polished-yawning-harbor.md

All steps complete. Here's a summary:

## Summary

Migrated remaining self-contained `fuzzy.c` functions to Rust across 4 commits:

| Phase | Commit | Functions Migrated |
|-------|--------|-------------------|
| 1 | `cb42b8ca27` | `fuzzy_match_str_compare`, `fuzzy_match_str_sort`, `fuzzy_match_func_compare`, `fuzzy_match_func_sort`, `fuzmatch_str_free` (bug fix), `fuzzymatches_to_strmatches` |
| 2 | `2973d9cc53` | `fuzzy_match_str` |
| 3 | `995e284e44` | `fuzzy_match_str_with_pos` |
| 4 | `58b1a3f6ec` | `fuzzy_match_str_in_line`, `search_for_fuzzy_match` |

**Key changes:**
- Added `FuzmatchStr`, `GArray`, `PosT` as `#[repr(C)]` structs with `_Static_assert` verification
- Fixed bug in `fuzmatch_str_free`: `fuzmatch[count].str` → `fuzmatch[i].str`
- Inner call from `search_for_fuzzy_match` → `fuzzy_match_str_in_line` is now a direct Rust call (no FFI hop)
- `fuzzy.c` reduced from 652 → 413 lines (thin wrappers + VimL functions)
- 5 VimL-heavy functions remain in C as planned

**All checks pass:** build, smoke-test, rust-fmt-check, rust-clippy, rust-test (4121 tests)
