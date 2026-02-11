Plan: ~/.claude/plans/lucky-hugging-alpaca.md

All 4 phases of the buffer.c Wave 2 migration plan are complete. Here's the summary:

## Wave 2 Migration Results

### Phase 1 (state.rs) — 3 functions
- `rs_buf_clear_file` — reset buffer file state
- `rs_buf_inc_changedtick` — increment b:changedtick
- `rs_wipe_buffer` — wipe buffer with optional autocmd blocking

### Phase 2 (list.rs) — 5 functions  
- `rs_buflist_findnr` — find buffer by number
- `rs_buflist_nr2name` — get name for buffer number
- `rs_buflist_findname` — find buffer by filename
- `rs_buflist_findname_exp` — find buffer by name after expansion
- `buflist_findname_file_id_impl` — internal Rust helper (C static kept for `buflist_new`)

### Phase 3 (lib.rs) — 3 functions
- `rs_otherfile_buf_4` — check if filename differs from buffer's file
- `rs_fname_expand` — expand filename to full path
- `rs_buflist_add` — add file to buffer list

### Phase 4 (lib.rs) — 3 functions
- `rs_buflist_altfpos` — set alternate cursor position
- `rs_buflist_findlnum` — find stored line number for buffer
- `rs_set_buflisted` — toggle buflisted flag with autocmds

### Deferred
`fileinfo` (90 lines) and `maketitle` (70 lines) + `value_change` (15 lines) were deferred per the plan's risk mitigation — they require extensive FFI bindings for messaging globals, `_()` translation macros, `NGETTEXT`, and the complex `build_stl_str_hl` 12-parameter interface.

### Stats
- **14 functions migrated** across 4 commits
- All checks passing: build, smoke-test, rust-fmt-check, rust-clippy, rust-test (4045 tests)
