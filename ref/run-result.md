Plan: ~/.claude/plans/nested-churning-cherny.md

All 7 phases of the plan have been completed. Here's a summary of what was migrated:

## Summary

**Phase 1 - Mark View Functions:**
- `rs_fmarkv_init()` - Initialize fmarkv_T with no view
- `rs_mark_view_make()` - Create mark view from topline and position
- `rs_mark_view_calc_topline()` - Calculate topline for view restoration
- `rs_fmarkv_has_view()` - Check if view is valid

**Phase 2 - Mark Structures and Validation:**
- Added `FmarkT` and `XfmarkT` structures matching C types
- `rs_mark_validate_lnum()` - Validate mark line number
- `rs_mark_validate_bounds()` - Validate against buffer bounds
- `rs_fmark_*` getters/setters for mark fields
- `rs_visual_mark_select()` - Logic for visual mark selection

**Phase 3 & 5 - Jumplist/Changelist Operations:**
- `rs_jumplist_new_len()` - Calculate new jumplist length
- `rs_jumplist_is_full()` - Check if jumplist is at capacity
- `rs_jumplist_stack_trim()` - Stack mode trimming
- `rs_jumplist_calc_idx()` - Jump navigation calculation
- `rs_changelist_calc_idx()` - Changelist navigation
- `rs_mark_target_type()` - Determine mark storage target
- `rs_pos_clamp_lnum_min()` - Position clamping

**Phase 4 - Mark Movement:**
- `rs_mark_move_calc_result()` - Calculate movement result flags
- `rs_mark_move_needs_cursor_check()` - Cursor check condition
- `rs_getnextmark_adjust_col()` - Column adjustment for search
- `rs_getnextmark_is_better()` - Mark comparison for next/prev search

**Phase 6 - Mark Adjustment:**
- `rs_mark_adjust_lnum()` - ONE_ADJUST logic
- `rs_mark_adjust_lnum_nodel()` - ONE_ADJUST_NODEL logic
- `rs_mark_adjust_cursor()` - ONE_ADJUST_CURSOR logic
- `rs_mark_col_adjust()` - COL_ADJUST logic
- `rs_mark_adjust_should_skip()` - Early bail-out check

**Phase 7 - Ex Command Helpers:**
- `rs_delmarks_parse_range()` - Parse :delmarks arguments
- `rs_delmarks_global_idx()` - Global mark index calculation
- `rs_delmarks_special_type()` - Special mark type detection
- `rs_marks_index_to_char()` - Mark display formatting

A total of **78 Rust functions** were added to the mark crate, with comprehensive test coverage. The C code in `mark.c` was updated to use these Rust implementations where appropriate.
