Plan: ~/.claude/plans/luminous-percolating-cake.md

The plan has been fully executed. Here's a summary of what was accomplished:

## Summary

Successfully completed the migration of all 5 remaining C functions in `cursor.c` to Rust, achieving 100% module completion.

### Migrated Functions

| Function | Rust Implementation | Complexity |
|----------|-------------------|------------|
| `inc_cursor()` | `rs_inc_cursor()` | Low |
| `dec_cursor()` | `rs_dec_cursor()` | Low |
| `pchar_cursor()` | `rs_pchar_cursor()` | Low |
| `get_cursor_rel_lnum()` | `rs_get_cursor_rel_lnum()` | Medium |
| `set_leftcol()` | `rs_set_leftcol()` | Medium-High |

### Files Modified

- **`src/nvim-rs/cursor/src/lib.rs`**: Added 261 lines with 5 new Rust functions and required extern declarations
- **`src/nvim/cursor.c`**: Updated 5 functions to call Rust, added accessor wrapper
- **`src/nvim/move.c`**: Added `nvim_changed_cline_bef_curs()` wrapper
- **`src/nvim/normal.c`**: Added `nvim_dec_cursor()` wrapper

### Verification

All checks passed:
- ✅ `just build` - Full build with linking
- ✅ `just rust-fmt-check` - Formatting
- ✅ `just rust-clippy` - Lints
- ✅ `just rust-test` - 3712 unit tests passed
