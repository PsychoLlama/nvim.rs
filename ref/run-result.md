Plan: ~/.claude/plans/glowing-whistling-kurzweil.md

The digraph module migration is complete. Let me provide a summary of what was done:

## Summary

I have successfully executed the digraph module Rust migration plan. Here's what was accomplished:

### Files Created
- `src/nvim-rs/digraph/src/validate.rs` - Character validation logic
- `src/nvim-rs/digraph/src/register.rs` - User digraph registration and reverse lookup  
- `src/nvim-rs/digraph/src/input.rs` - Input state machine for digraph composition
- `src/nvim-rs/digraph/src/viml.rs` - Vimscript function helper utilities
- `src/nvim-rs/digraph/src/list.rs` - Digraph iteration and formatting helpers

### Files Modified
- `src/nvim-rs/digraph/src/lib.rs` - Added module declarations and re-exports
- `src/nvim/digraph.c` - Converted C functions to call Rust implementations
- `ref/crates.md` - Updated digraph crate description

### Functions Migrated to Rust
| Function | Description |
|----------|-------------|
| `check_digraph_chars_valid` | Validates digraph character pairs |
| `registerdigraph` | Adds user-defined digraphs to garray |
| `get_digraph_for_char` | Reverse lookup (char → digraph string) |
| `do_digraph` | Input state machine for digraph composition |

### New C Accessor Functions
- `nvim_get_user_digraphs_ptr()` - Get garray pointer for mutation
- `nvim_user_digraphs_grow()` - Grow user digraphs array
- `nvim_user_digraphs_inc_len()` - Increment array length
- `nvim_get_p_dg()` - Get 'digraph' option value

### Verification
All checks pass:
- ✅ `just build` - Full build with linking
- ✅ `just rust-fmt-check` - Formatting
- ✅ `just rust-clippy` - Lints
- ✅ `just rust-test` - Unit tests (3669 tests passed)

The VimL function entry points (`f_digraph_get`, `f_digraph_set`, `f_digraph_getlist`, `f_digraph_setlist`) remain in C due to their heavy dependency on typval manipulation, but they now call Rust for the core logic.
