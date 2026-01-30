Plan: ~/.claude/plans/quizzical-dreaming-firefly.md

I have successfully completed the migration plan. Here's a summary of what was done:

## Summary

Successfully migrated `cursor_shape.c` parsing logic to Rust:

### Phase 1: Added C Setter Accessors
Added the following setter functions to `cursor_shape.c`:
- `nvim_set_shape_table_shape()`, `nvim_set_shape_table_percentage()`
- `nvim_set_shape_table_blinkwait()`, `nvim_set_shape_table_blinkon()`, `nvim_set_shape_table_blinkoff()`
- `nvim_set_shape_table_id()`, `nvim_set_shape_table_id_lm()`
- Additional accessors: `nvim_get_p_guicursor()`, `nvim_syn_check_group()`, `nvim_ui_mode_info_set()`

### Phase 2: Migrated `clear_shape_table()`
Implemented `rs_clear_shape_table()` in Rust that resets all shape_table entries to defaults.

### Phase 3: Migrated `parse_shape_opt()`
Implemented `rs_parse_shape_opt()` in Rust with full guicursor parsing:
- Two-pass validation (check errors first, then apply changes)
- Mode string parsing (n, v, i, r, c, ci, cr, o, ve, a for "all")
- Shape parsing (block, ver25, hor50)
- Blink timing parsing (blinkwait, blinkon, blinkoff)
- Highlight group parsing with langmap support (group/lm_group)
- Proper error messages matching C (E545, E546, E548, E549)

### Phase 4: Updated C Wrappers
Replaced C implementations with thin wrappers calling the Rust functions.

### Verification
- `just build` - passed
- `just rust-test` - 3717 tests passed
- `just rust-fmt-check` - passed
- `just rust-clippy` - passed
- Manual guicursor tests - all error cases work correctly

The `cursor_shape.c` file now contains only:
- `shape_table[]` definition
- Accessor functions (getters/setters)
- `mode_style_array()` for API serialization (kept in C due to Arena allocation complexity)
- Thin wrappers calling `rs_*` functions
