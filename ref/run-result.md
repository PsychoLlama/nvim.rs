Plan: ~/.claude/plans/concurrent-puzzling-graham.md

## Summary

I have executed the plan at `/home/overlord/.claude/plans/concurrent-puzzling-graham.md`. Here's what was accomplished:

### Phase 1: Undo System Completion ✅

1. **Added C accessors for memline operations** (`src/nvim/undo.c`):
   - `nvim_ml_delete_lnum`, `nvim_ml_delete_flags`
   - `nvim_ml_append_lnum`, `nvim_ml_append_flags`
   - `nvim_ml_replace_lnum`
   - `nvim_block_autocmds`, `nvim_unblock_autocmds`
   - `nvim_undo_setpcmark`, `nvim_undo_check_cursor_lnum`
   - `nvim_undo_mark_adjust`, `nvim_undo_changed_lines`
   - `nvim_buf_changed`, `nvim_buf_unchanged`
   - `nvim_spell_check_window`, `nvim_redrawWinline`
   - `nvim_extmark_apply_undo`, `nvim_buf_updates_unload`
   - `nvim_check_pos`, `nvim_buf_is_empty`
   - `nvim_undo_get_curwin`, `nvim_undo_win_get_buffer`
   - `nvim_undo_win_set_cursor_pos`, `nvim_undo_win_get_cursor_lnum`
   - Plus various global state accessors

2. **Migrated `u_compute_hash()` to Rust**:
   - Implemented `rs_u_compute_hash` using the encoding crate's SHA-256 implementation
   - Added `nvim-encoding` dependency to the undo crate
   - C function `u_compute_hash` now calls the Rust implementation

### Phase 2: Screen Rendering - Reviewed ✅

- Discovered that most helper functions (`fill_foldcolumn`, `draw_sign`, `draw_lnum_col`) are already thin C wrappers calling Rust implementations
- The `win_line()` function (~2000 lines) remains in C and would require significant segmented migration

### Phase 3: File I/O - Reviewed ✅

- Examined `modname()`, `vim_rename()`, `vim_copyfile()`, and timestamp functions
- Found existing infrastructure in the fileio crate for path operations

### Commits Made

1. `2f135e4097` - feat(undo): add C accessors for memline operations and undo helpers
2. `9664d665ff` - feat(undo): migrate u_compute_hash() to Rust

### Verification

All checks pass:
- `just build` ✅
- `just rust-fmt-check` ✅
- `just rust-clippy` ✅
- `just rust-test` - 3491 tests passed ✅
