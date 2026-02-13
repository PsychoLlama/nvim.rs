Plan: ~/.claude/plans/sorted-skipping-noodle.md

All 3 phases of the `cursor.c` completion migration are done. Here's the summary:

**3 commits on `rust-migration`:**

| Commit | Description |
|--------|-------------|
| Phase 1 | Convert 7 C functions to thin `rs_*` wrappers; fix `gchar_cursor` circular FFI |
| Phase 2 | Migrate `coladvance_force` to Rust; add `nvim_coladvance2_addspaces` and `nvim_coladvance_force` C accessors |
| Phase 3 | Cleanup and update metadata |

**Changes:**
- `rs_gchar_cursor` now calls `nvim_cursor_get_pos_ptr` + `rs_utf_ptr2char` directly (breaks circular FFI)
- `nvim_gchar_cursor` in `normal.c` made self-contained (calls `utf_ptr2char` + `get_cursor_pos_ptr`)
- 7 C functions converted to thin wrappers: `gchar_cursor`, `char_before_cursor`, `pchar_cursor`, `get_cursor_line_ptr`, `get_cursor_pos_ptr`, `get_cursor_line_len`, `get_cursor_pos_len`
- New `rs_coladvance_force` in cursor crate
- New C accessors: `nvim_coladvance2_addspaces`, `nvim_coladvance_force`
- Change crate's dangling `nvim_coladvance_force` FFI now resolves at link time

**Post-migration state of `cursor.c`:**
- ~15 thin `rs_*` wrappers (all public API functions)
- ~28 C accessor functions (`nvim_*` functions for Rust FFI access)
- 1 static function (`coladvance2` — 155 lines, kept in C deliberately)
- 0 lines of unmigrated public logic
