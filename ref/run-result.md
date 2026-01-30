Plan: ~/.claude/plans/jolly-doodling-giraffe.md

The plan has been executed successfully. Here's a summary of what was accomplished:

## Summary

Successfully migrated 5 cursor validation functions from C to Rust:

1. **`check_pos`** - Validates position (lnum/col) against buffer bounds
2. **`check_cursor_lnum`** - Validates cursor line number with fold handling at end of file
3. **`check_cursor_col`** - Validates cursor column with complex mode-dependent rules (Insert, Terminal, Visual modes, virtualedit)
4. **`check_cursor`** - Composite function calling lnum + col validation
5. **`check_visual_pos`** - Validates Visual mode selection position

### Key Implementation Details:

- **Added C accessor functions** for Rust to access required state:
  - Buffer line count and line length accessors
  - Visual position getters/setters
  - Fold checking wrapper (`nvim_check_folding_at_end`)
  - Cursor coladd setter
  - `mark_mb_adjustpos` wrapper for multibyte safety
  - Virtual column range getter for virtualedit handling

- **Original C functions** in `cursor.c` are now thin wrappers that call the corresponding `rs_*` Rust functions

### Verification:
- ✅ `just build` - Full build with linking passes
- ✅ `just rust-fmt-check` - Formatting check passes
- ✅ `just rust-clippy` - Lint check passes  
- ✅ `just rust-test` - All 3672 unit tests pass
