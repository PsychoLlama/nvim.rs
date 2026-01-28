Plan: ~/.claude/plans/idempotent-wondering-sky.md

## Quickfix Subsystem Rust Migration - Phase 1 Complete

Successfully migrated the **core list lifecycle functions** from C to Rust.

### Functions Migrated

#### `qf_new_list` (rs_qf_new_list)
Creates a new quickfix list in the stack:
1. Deletes lists beyond current position
2. Handles stack full case by popping oldest
3. Initializes new list with title, type, and unique ID

#### `qf_free_items` (rs_qf_free_items)
Frees all entries in a quickfix list:
1. Walks through linked list freeing each entry
2. Handles circular reference edge case
3. Resets list pointers and state
4. Cleans directory/file stacks
5. Resets multiline flags

#### `qf_free` (rs_qf_free_list)
Frees all resources of a quickfix list:
1. Frees all items via rs_qf_free_items
2. Frees title, context, callback
3. Resets id and changedtick

#### `qf_pop_stack` (rs_qf_pop_stack)
Pops oldest list from the stack:
1. Frees first list via rs_qf_free_list
2. Shifts all lists down
3. Zeros the now-unused top list
4. Optionally adjusts listcount and curlist

### C Accessor Functions Added

For list lifecycle operations:
- `nvim_qf_set_id`, `nvim_qf_set_qfl_type`, `nvim_qf_set_has_user_data`
- `nvim_qf_get_list_at_mut`, `nvim_qf_alloc_next_id`, `nvim_qf_clear_list_struct`
- `nvim_qf_free_title`, `nvim_qf_free_ctx`, `nvim_qf_free_callback`
- `nvim_qf_set_changedtick`
- `nvim_qf_shift_lists_down`, `nvim_qf_zero_top_list`
- `nvim_qf_decr_curlist`, `nvim_qf_decr_listcount`

### Stats

- C file (quickfix.c): Started at 9,394 lines
- Rust quickfix crate: 13,376 lines across 14 modules
- Exported Rust functions: 383

### Commits

1. `feat(quickfix): Phase 1 - migrate qf_new_list to Rust`
2. `feat(quickfix): Phase 1 - migrate qf_free_items to Rust`
3. `feat(quickfix): Phase 1 - migrate qf_free to Rust`
4. `feat(quickfix): Phase 1 - migrate qf_pop_stack to Rust`

### Remaining Work

Phases 2-10 from the original plan:
- Phase 2: Error Format Parsing (~800 lines)
- Phase 3: List Population (~600 lines)
- Phase 4: Navigation (~700 lines)
- Phase 5: Window Management (~500 lines)
- Phase 6: Ex Commands (~1,500 lines)
- Phase 7: VimL Functions (~1,200 lines)
- Phase 8: :cdo/:ldo Commands (~300 lines)
- Phase 9: Grep and Help Integration (~400 lines)
- Phase 10: Cleanup and Optimization
