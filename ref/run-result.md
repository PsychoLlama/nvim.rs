Plan: ~/.claude/plans/zippy-stirring-twilight.md

All steps of the plan are complete. Here's a summary of what was done:

## Edit Crate Wave 3 Complete

**9 functions migrated** from `src/nvim/edit.c` to Rust across 4 phases:

### Phase 1: C Accessor Infrastructure
- Added 30+ new accessor/wrapper functions in `edit.c` for globals (`spell_redraw_lnum`, `ai_col`, `no_mapping`, `textlock`, `mod_mask`, `got_int`, etc.) and C library calls (`AppendToRedobuff`, `plain_vgetc`, `merge_modifiers`, `get_special_key_name`, etc.)
- Added 9 `_Static_assert` entries for new constants

### Phase 2: Leaf Functions (3 functions)
- `redo_literal` → `editing.rs` (encodes literal chars into redo buffer)
- `check_spell_redraw` → `helpers.rs` (redraws spell-checked line)
- `do_insert_char_pre` → `editing.rs` (triggers `InsertCharPre` autocmd)

### Phase 3: Arrow State Functions (4 functions)
- `start_arrow_common` → `state.rs` (core arrow key state transition)
- `start_arrow` → `state.rs` (thin wrapper)
- `start_arrow_with_change` → `state.rs` (adds CTRL-G U to redo)
- `stop_arrow` → `state.rs` (resets state, starts new insertion)

### Phase 4: Input Processing Functions (2 functions)
- `insert_special` → `editing.rs` (handles special key insertion with modifiers)
- `get_literal` → `editing.rs` (CTRL-V literal character input with decimal/hex/octal/unicode digit accumulation)

**All checks pass**: build, smoke-test, rust-test (4045/4045), rust-fmt-check, rust-clippy
