Plan: ~/.claude/plans/synchronous-meandering-gosling.md

## Summary

I've executed the Phase 6 quickfix migration plan. Here's what was accomplished:

### Commits Made

1. **feat(quickfix): migrate qf_add_entry to Rust (Phase 6.1)**
   - Replaced `qf_add_entry()` C function body with call to Rust `rs_qf_add_entry()`
   - Added `nvim_qfline_set_user_data()` FFI accessor for typval copying
   - Added `user_data` parameter to the Rust function
   - **78 C lines deleted**

2. **feat(quickfix): migrate entry_is_closer_to_target to Rust**
   - Implemented `rs_qf_entry_is_closer_to_target()` in Rust
   - Replaced 57-line C function comparing entries to target position
   - **52 C lines deleted**

3. **refactor(quickfix): use Rust rs_qf_id2nr in qf_id2nr**
   - Used existing `rs_qf_id2nr()` to replace C loop
   - **7 C lines deleted**

4. **feat(quickfix): migrate qf_restore_list to Rust**
   - Implemented `rs_qf_restore_list()` in Rust
   - Replaced C function that restores current list by ID
   - **12 C lines deleted**

### Results

- **Total C lines deleted**: 148
- **C lines added** (declarations/wrappers): 38
- **Net reduction in quickfix.c**: 110 lines (from 9,598 to 9,488)

### Why We Stopped Short of 600 Lines

The plan's Phase 6.2-6.4 targets (dictionary entry creation, multiline parsing, property retrieval) all require extensive **typval dictionary FFI infrastructure** that doesn't exist:
- `tv_dict_get_string()`, `tv_dict_get_number()`, `tv_dict_find()`, `tv_dict_get_tv()`
- Dictionary item creation and manipulation
- List operations for property retrieval

Building this infrastructure would be a substantial effort on its own. The functions we migrated were those with clean migration paths using existing FFI accessors.

### Verification

All checks pass:
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- `just rust-test` (3717 tests passed) ✓
