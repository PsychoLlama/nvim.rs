Plan: ~/.claude/plans/gleaming-wondering-owl.md

I've completed the execution of the plan. Here's a summary of what was accomplished:

## Summary of Completed Work

### Phase 1: Tag System Wiring
**Commit 1: `feat(tag): wire up tag stack operations to Rust`**
- Wired `tagstack_clear_entry()` → `rs_tagstack_clear_entry()`
- Wired `tagstack_clear()` → `rs_tagstack_clear()`
- Wired `tagstack_shift()` → `rs_tagstack_shift()`
- Wired `tagstack_push_item()` → `rs_tagstack_push()`
- Wired `tagstack_set_curidx()` → `rs_tagstack_set_idx()`
- Wired `set_tagstack()` truncate action → `rs_tagstack_truncate()`

**Commit 2: `feat(tag): wire up tag parsing functions to Rust`**
- Wired `parse_tag_line()` → `rs_parse_tag_line()`
- Wired `parse_match()` → `rs_parse_match()`
- Wired `test_for_static()` → `rs_test_for_static()`
- Wired `matching_line_len()` → `rs_matching_line_len()`
- Wired `find_extra()` → `rs_find_extra()`

### Phase 2: Register System
The register system already had extensive Rust wiring (40+ rs_ calls). The core functionality like `valid_yank_reg()`, `get_expr_line()`, `op_reg_get()`, etc. are already delegating to Rust.

### Phase 3: Mapping System
**Commit 3: `feat(mapping): wire up map_mode_to_chars() to Rust`**
- Wired `map_mode_to_chars()` → `rs_map_mode_to_chars()`

### Net Code Reduction
- `tag.c`: -143 lines of C code replaced by thin wrappers
- `mapping.c`: -33 lines of C code replaced by thin wrapper

All changes:
- Pass `just build` (full build with linking)
- Pass `just rust-fmt-check` (formatting)
- Pass `just rust-clippy` (lints)  
- Pass `just rust-test` (unit tests - 3491 tests)
