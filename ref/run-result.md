Plan: ~/.claude/plans/modular-spinning-puddle.md

## Summary

I've executed the plan for wiring undo, quickfix, and search components to Rust. Here's what was accomplished:

### Phase 1: Undo/Redo System
- **Status**: Already complete from previous work
- The undo system was already extensively wired to Rust, with functions like `u_write_undo`, `u_read_undo`, `u_savecommon`, `u_undo`, `u_redo`, `ex_undolist`, `u_sync`, `u_unchanged`, `u_update_save_nr`, and `u_clearall` all forwarding to Rust implementations

### Phase 2: Quickfix
- Added FFI declarations and wrapper functions for:
  - **Navigation functions**: `QfJumpTarget` struct, `rs_qf_calc_jump_target`, `rs_qf_idx_for_lnum`, `rs_qf_first_entry_in_file`, `rs_qf_last_entry_in_file`, `rs_qf_entry_after_pos_idx`, `rs_qf_entry_before_pos_idx`
  - **Entry position checks**: `rs_qf_at_first`, `rs_qf_at_last`
  - **Entry validation**: `rs_qf_entry_covers_line`, `rs_qf_entry_has_valid_range`, `rs_qf_entry_has_valid_col_range`, `rs_qf_entry_has_module`, `rs_qf_entry_has_nr`
  - **Entry analysis**: `rs_qf_entry_line_span`, `rs_qf_entry_is_complete`, `rs_qf_entry_is_diagnostic`, `rs_qf_entry_severity`
  - **Entry counting**: `rs_qf_count_errors`, `rs_qf_count_warnings`, `rs_qf_curlist_entry_count`, `rs_qf_curlist_valid_count`

### Phase 3: Search
- Added FFI declarations for:
  - **Pattern utilities**: `rs_pat_has_uppercase`, `rs_ignorecase`, `rs_ignorecase_opt`, `rs_needs_previous_pattern`
  - **Pattern accessors**: `rs_get_search_pattern`, `rs_get_subst_pattern`, `rs_get_last_used_pattern`, `rs_get_mr_pattern`
  - **Incremental search**: `rs_incsearch_state_save`, `rs_incsearch_state_restore`

### Commits
1. `b999711904` - feat(quickfix): wire navigation Rust functions to C
2. `576f936bb6` - feat(search): add FFI declarations for Rust pattern utilities
3. `c75bb9c58f` - feat(quickfix,search): add more Rust FFI declarations

### Verification
- Build passes: ✅
- Rust format check: ✅
- Rust clippy: ✅
- Rust unit tests (undo): ✅
