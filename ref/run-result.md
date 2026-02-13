Plan: ~/.claude/plans/humble-tickling-crystal.md

All 6 phases of the `help.c` migration are complete. Here's the summary:

## Migration Complete

**Commits:**
1. `f141585307` — Phase 1: `check_help_lang`, `help_compare`
2. `87b6a843ba` — Phase 2: `find_help_tags`, `cleanup_help_tags`
3. `e1683f4ce9` — Phase 3: `ex_exusage`, `ex_viusage`
4. `8fce0d8668` — Phase 4: `prepare_help_buffer`
5. `f9634928d3` — Phase 5: `ex_helpclose`, `ex_help`
6. `8f2536c742` — Phase 6: `helptags_one`, `do_helptags`, `helptags_cb`, `ex_helptags`, `get_local_additions`

**Result:**
- All 14 functions migrated from C to Rust
- `help.c` reduced from ~1,110 lines of logic to ~215 lines of thin wrappers + C accessor functions
- `src/nvim-rs/help/src/lib.rs` grew from 218 lines to ~1,630 lines
- `just build`, `just smoke-test`, `just rust-fmt-check`, `just rust-clippy`, `just rust-test` (4,120 tests) all pass

**Phase 6 highlights:**
- Used `Vec<*mut c_char>` instead of `garray_T` for tag collection in `helptags_one`
- Used `Vec<[u8; 2]>` instead of `garray_T` for language detection in `do_helptags`
- Encapsulated `vimconv_T` encoding conversion in a C accessor (`nvim_help_convert_help_line`)
- Wrapped `ExpandInit`/`ExpandOne` in C accessor (`nvim_help_expand_dir`) to avoid exposing `expand_T`
- `rs_helptags_cb` passed directly as function pointer to `do_in_path` via FFI
- Cleaned up the `#if 0` dead code block from old `find_help_tags`
