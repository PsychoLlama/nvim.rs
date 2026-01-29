Plan: ~/.claude/plans/logical-moseying-tarjan.md

I have completed the plan execution. Here's a summary of what was accomplished:

## Summary

The plan requested wiring several functions from C to Rust. After thorough analysis of the codebase, I found that:

1. **Many functions were already wired**: Functions like `diffopt_horizontal()`, `diffopt_filler()`, `diff_cmp()`, `diff_copy_entry()`, `stl_connected()`, `stl_clear_click_defs()`, `stl_fill_click_defs()`, `fillchar_status()`, `calc_percentage()`, `parse_diff_ed()`, `parse_diff_unified()` were already wired to Rust.

2. **Some functions couldn't be wired directly**: Functions like `get_rel_pos()` and `append_arg_number()` have Rust implementations, but the Rust versions don't support localization (`_()` calls to gettext), so wiring them would break internationalization.

3. **Complex functions need more work**: Functions like `build_stl_str_hl()` (~1300 lines) involve complex state management, expression evaluation, and localization that the Rust implementation doesn't fully cover yet.

4. **Side-effect functions can't be wired as-is**: Many diff functions call `ex_diffupdate(NULL)` for side effects, which pure Rust implementations can't replicate.

### Wired in this commit:

I successfully wired two functions:
- **`diff_buf_idx(buf, tp)`** → `rs_diff_buf_idx_tp()` - Finds the index of a buffer in a tabpage's diff list
- **`diff_check_sanity(tp, dp)`** → `rs_diff_check_sanity()` - Validates that diff block line numbers are within buffer bounds

Both functions had clean Rust implementations that exactly matched the C logic. The build passes and all tests succeed.
