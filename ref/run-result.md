Plan: ~/.claude/plans/melodic-meandering-pixel.md

The plan has been executed. Here's a summary of what was accomplished:

## Summary

The getchar Rust migration plan has been executed. The following changes were made:

### Phase 1: Input Buffer Infrastructure
1. **`vungetc()`** - Migrated to Rust (`rs_vungetc`)
   - Added C accessor functions for `old_mouse_*`, `old_KeyStuffed`, `mouse_*` globals
   - Added `nvim_can_get_old_char()` wrapper

2. **`vpeekc()`** - Updated to use Rust accessors
   - Now uses `rs_can_get_old_char()` and `rs_get_old_char()`

3. **`ins_char_typebuf()`** - Migrated to Rust (`rs_ins_char_typebuf`)
   - Calls `rs_special_to_buf` and `rs_ins_typebuf`
   - Added `nvim_add_on_key_ignore_len()` accessor

### Phase 2: Stuff and Redo Buffer Wrappers
1. **Stuff buffer operations**:
   - `rs_stuffReadbuff` - Append string to stuff buffer
   - `rs_stuffcharReadbuff` - Append character to stuff buffer  
   - `rs_stuffnumReadbuff` - Append number to stuff buffer
   - `rs_stuffRedoReadbuff` - Append to redo stuff buffer

2. **Redo buffer operations**:
   - `rs_AppendToRedobuff` - Append string to redo buffer
   - `rs_AppendCharToRedobuff` - Append character to redo buffer
   - `rs_AppendNumberToRedobuff` - Append number to redo buffer

### Commits Made
1. `b13f8b2f5e` - feat(getchar): migrate vungetc to Rust
2. `5fed05f398` - feat(getchar): migrate vpeekc and ins_char_typebuf to Rust  
3. `4dfa3f0d2f` - feat(getchar): add Rust wrappers for stuff buffer operations
4. `4f8ae733f6` - feat(getchar): add Rust wrappers for redo buffer operations
5. `7a21a5daea` - docs: update getchar crate description in ref/crates.md

All builds pass, Rust tests pass (3491 tests), clippy and fmt checks pass.
