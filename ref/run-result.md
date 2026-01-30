Plan: ~/.claude/plans/greedy-humming-peach.md

All tasks in the plan have been completed. Here's a summary of what was implemented:

## Summary

I successfully executed the plan to migrate 6 cursor functions from C to Rust.

### Phase 1: Line Accessors (4 functions)
- `get_cursor_line_ptr()` - Returns pointer to cursor line
- `get_cursor_pos_ptr()` - Returns pointer to cursor position in line  
- `get_cursor_line_len()` - Returns length of cursor line
- `get_cursor_pos_len()` - Returns length from cursor position to end of line

### Phase 2: Character Access (2 functions)
- `char_before_cursor()` - Returns the character immediately before the cursor, using UTF-8 aware functions from nvim-mbyte
- `adjust_cursor_col()` - Ensures cursor isn't positioned on the line-ending NUL

### Files Modified
| File | Changes |
|------|---------|
| `src/nvim/cursor.c` | Added 5 C accessor functions, updated 6 functions to call Rust |
| `src/nvim-rs/cursor/src/lib.rs` | Added 6 extern declarations, implemented 6 rs_* functions |
| `src/nvim-rs/cursor/Cargo.toml` | Added nvim-mbyte dependency |

### Verification
- ✅ `just build` - Full build passes
- ✅ `just rust-fmt-check` - Formatting check passes
- ✅ `just rust-clippy` - Lints pass
- ✅ `just rust-test` - All 3672 unit tests pass
- ✅ Manual verification - Neovim starts correctly
