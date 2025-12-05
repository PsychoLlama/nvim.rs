# Functions to Migrate

Candidates for Rust migration, organized by priority.

## TIER 1 - Migrate First (Trivial, self-contained)

### ex_docmd.c
- [x] `ends_excmd` (line 4622) - Returns true if character is command terminator (NUL, '|', '"', '\n') - MIGRATED Phase 1.22

### fileio.c
- [x] `time_differs` (line 2164) - Compares file modification times with FAT tolerance - MIGRATED Phase 1.23

### shada.c
- [x] `hist_type2char` (line 2175) - Translates history type number to character - MIGRATED Phase 1.24 (added to cmdhist crate)

### quickfix.c (static functions - NOT SUITABLE)
- ~~`qf_stack_empty` (line 923)~~ - Depends on quickfix stack pointer
- ~~`qf_list_empty` (line 931)~~ - Depends on quickfix list pointer

### window.c (frame tree functions)
- [ ] `frame_has_win` (line 3563) - Recursive check if frame contains window
- [ ] `is_bottom_win` (line 3581) - Check if window is at bottom of layout
- [ ] `frame_fixed_height` (line 3699) - Check if frame height is fixed
- [ ] `frame_fixed_width` (line 3733) - Check if frame width is fixed
- [ ] `frame_check_height` (line 7396) - Verify frame heights are correct
- [ ] `frame_check_width` (line 7417) - Verify frame widths are correct

## TIER 2 - Good Candidates (Minor refactoring needed)

### window.c (window/tab validation)
- [ ] `tabpage_win_valid` (line 1684) - Check if window exists in tabpage
- [ ] `win_valid_any_tab` (line 1715) - Check if window valid in any tab
- [ ] `win_find_by_handle` (line 1701) - Find window by handle (needs curtab context)
- [ ] `valid_tabpage` (line 4390) - Check if tabpage pointer is valid

### plines.c (display calculations)
- [ ] `charsize_fast_impl` (line 346) - Character display width calculation
- [ ] `charsize_fast` (line 384) - Wrapper for charsize_fast_impl
- [ ] `in_win_border` (line 408) - Check if column is in window border

### spell.c
- [ ] `spell_mb_isword_class` (line 2509) - Character class word check
- ~~`valid_spelllang` (line 3655)~~ - Wrapper around `valid_name` (already migrated), no value in duplicating

## TIER 3 - Moderate Candidates (Parameter passing required)

### eval/typval.c
- [ ] `tv_list_find` (line 1585) - Find item at index in list (with caching)
- [ ] `tv_list_idx_of_item` (line 1710) - Find index of item in list

### window.c (composition functions)
- [ ] `last_window` (line 2554) - Check if only one window across all tabs
- [ ] `one_window` (line 2564) - Check if only one window in tabpage
- [ ] `current_win_nr` (line 1135) - Get window number (needs curtab context)

### buffer.c
- [ ] `buf_valid` (line 451) - Check if buffer pointer is valid (needs buffer list)

## DO NOT MIGRATE (Global state dependency)

- `last_csearch` (search.c:447) - Returns global state directly
- `bufref_valid` (buffer.c:438) - Depends on global counter
- `only_one_window` (window.c:7155) - Multiple global dependencies

---

## Notes

- Static functions may need to be exposed or have their Rust equivalents called from within the existing C function
- Window/frame functions require understanding of the frame tree structure
- Some functions marked PURE actually depend on global state (curtab, firstwin, etc.)

## Search Results (2025-12-04)

Most remaining FUNC_ATTR_PURE/FUNC_ATTR_CONST functions fall into these categories:
1. Functions accessing global state (p_paste, curbuf, State, etc.)
2. Functions taking complex struct pointers (win_T*, buf_T*, frame_T*)
3. Functions calling external libraries (utf8proc_*)

The trivial pure functions have largely been migrated. Next steps should focus on:
- Building out Rust infrastructure for handling struct types via FFI
- Or identifying new simple functions without the PURE/CONST attributes

## Crate Status Audit (2025-12-04)

### Fully Swapped Crates (functions called from C)
| Crate | Status | Functions |
|-------|--------|-----------|
| nvim-math | ✅ Swapped | xfpclassify, xisinf, xisnan, xctz, xpopcount, num_divide, num_modulus, etc. |
| nvim-charset | ✅ Swapped | skipwhite, skipdigits, hex2nr, transchar_hex, etc. |
| nvim-path | ✅ Swapped | vim_ispathsep, path_tail, path_is_url, etc. |
| nvim-strings | ✅ Swapped | vim_stricmp, vim_strchr, has_non_ascii, valid_name, etc. |
| nvim-mbyte | ✅ Swapped | utf_char2len, utf_ptr2char, utf_printable, etc. |
| nvim-memutil | ✅ Swapped | xstrchrnul, xmemscan, strcnt, strequal, hash_hash, etc. |
| nvim-indent | ✅ Swapped | tabstop_padding, indent_size_ts |
| nvim-keycodes | ✅ Swapped | name_to_mod_mask, handle_x_keys |
| nvim-profile | ✅ Swapped | profile_zero, profile_add, profile_sub, etc. |
| nvim-menu | ✅ Swapped | menu_is_winbar, menu_is_popup, etc. |
| nvim-help | ✅ Swapped | help_heuristic |
| nvim-encoding | ✅ Swapped | base64_encode, base64_decode, sha256_* |
| nvim-cmdhist | ✅ Swapped | hist_char2type, hist_type2char |
| nvim-ex_docmd | ✅ Swapped | ends_excmd, find_nextcmd, check_nextcmd |
| nvim-fileio | ✅ Swapped | time_differs |

### Partially Swapped Crates
| Crate | Status | Functions |
|-------|--------|-----------|
| nvim-os | ✅ Partial | os_get_pid (1.26), os_get_hostname (1.27), os_time (1.29), os_hrtime (1.30), os_sleep (1.31) |
| nvim-collections (hashtab) | ✅ Swapped (2.2) | hash_hash, hash_hash_len, hash_init, hash_clear, hash_find, hash_find_len, hash_lookup, hash_add_item, hash_remove, hash_lock, hash_unlock |
| nvim-collections (garray) | ✅ Swapped (2.1) | ga_init, ga_set_growsize, ga_clear, ga_grow, ga_append, ga_append_via_ptr, ga_concat, ga_concat_len |

### Unswapped Crates (Rust code exists but NOT used from C)
| Crate | Status | Blocker |
|-------|--------|---------|
| nvim-os (env/fs) | 🔧 Ready but blocked | Memory allocator fixed (uses NvimString), but C uses libuv for env functions |

### Migration Blockers

1. **Memory allocation mismatch**: ~~OS crate allocates with Rust `CString`~~ **FIXED (Phase 1.25)**: `rs_os_getenv` now uses `NvimString` which allocates with `xmallocz`.

2. **libuv dependency**: OS/filesystem functions in C use libuv for portability. Rust's `std::env` differs in edge cases (error codes, Unicode handling on Windows). Options:
   - Keep C implementation for critical functions
   - Wrap libuv calls in Rust using libuv-sys crate
   - Gradually replace where behavior matches

3. **Complex struct types**: frame_T, win_T, buf_T have deep pointer hierarchies. Simple FFI doesn't work; need either opaque pointers with callbacks or full struct mirroring.

## Additional Search (2025-12-04 session 2)

Searched the following files for unexplored pure functions:
- `arabic.c` - Functions use `p_arshape`, `p_tbidi` globals
- `autocmd.c` - All functions access global autocmd lists
- `buffer.c` - `bt_*` functions (bt_help, bt_normal, bt_quickfix, etc.) all take `buf_T*` structs
- `cmdexpand.c` - Functions use `wop_flags` global
- `digraph.c` - `digraph_get` uses `user_digraphs` global and static tables
- `mapping.c` - Functions use `p_cpo` global, call `replace_termcodes`
- `plines.c` - Functions access window struct internals (`wp->w_view_width`)
- `register.c` - Returns global pointers
- `viml/parser/expressions.c` - Inline functions with static lookup tables

**Conclusion**: All remaining FUNC_ATTR_PURE/CONST functions fall into unsuitable categories. The Phase 1 pure function migration is complete.

## Session 3 Summary (2025-12-04)

**Phases 1.26 and 1.27**: Successfully swapped two OS layer functions to Rust:
- `os_get_pid` - Process ID retrieval
- `os_get_hostname` - Hostname retrieval

**Current Status**: 130+ Rust functions linked into nvim binary, 16+ crates with swapped functions.

**Remaining OS functions blocked**: Most other OS layer functions (`os_getenv`, `os_setenv`, filesystem operations) use libuv which has subtly different behavior than Rust's `std`. Swapping these would require either:
1. Using libuv-sys crate in Rust to match exact behavior
2. Verifying behavior matches in all edge cases
3. Accepting potential subtle differences

**Next migration targets would require**:
- Complex struct FFI (win_T, buf_T, list_T, dict_T)
- Global state access patterns
- Callback/event loop integration

## Session 4 Summary (2025-12-04)

**Phase 1.28**: Successfully swapped two more ex_docmd functions to Rust:
- `find_nextcmd` - Find next command after '|' or '\n' separator
- `check_nextcmd` - Check if at command separator after whitespace

These are pure string scanning functions with no global state dependencies.

**Exhaustive search performed**: Searched for additional candidates:
- `rem_backslash` - Uses `vim_isfilec` which accesses `g_chartab` global
- `mb_charlen` - Uses `utfc_ptr2len` for composing character handling
- `vim_strsize` - Uses `utfc_ptr2len` and `ptr2cells`
- `os_shell_is_cmdexe` - Uses `os_getenv_noalloc` for global env access

**Conclusion**: No more simple pure functions identified. All remaining candidates:
1. Access global state (`g_chartab`, `curbuf`, options)
2. Use complex UTF-8 composing character functions (`utfc_ptr2len`)
3. Call external libraries (utf8proc, libuv)
4. Take struct pointers (win_T*, buf_T*)

Phase 1 pure function migration is complete. Future migration requires:
- Complex struct FFI infrastructure
- Or accepting libuv behavior differences for OS functions

## Session 5 Summary (2025-12-04)

**Exhaustive verification of remaining candidates:**

Searched all `.c` files with FUNC_ATTR_PURE/CONST attributes:
- All files containing PURE/CONST functions also contain USE_RUST patterns
- This confirms all suitable pure functions have already been migrated

**Files now using Rust (20 total C files):**
- base64.c, charset.c, cmdhist.c, eval.c, ex_docmd.c, fileio.c
- hashtab.c, help.c, indent.c, keycodes.c, math.c, mbyte.c
- memory.c, menu.c, option.c, path.c, profile.c, sha256.c
- shada.c, strings.c
- os/env.c, os/time.c

**Functions verified as unsuitable:**
- `arabic_maycombine` - Uses `p_arshape`, `p_tbidi` globals
- `cursor_is_block_during_visual` - Uses `shape_table` global
- `min_vim_version`, `highest_patch`, `has_vim_patch` - Access static version arrays
- `has_format_option` - Uses `p_paste` and `curbuf->b_p_fo` globals
- `os_now` - Uses `&main_loop.uv` global (libuv event loop)

**Phase 1 Status: COMPLETE**

All trivial self-contained functions have been identified and migrated. The migration has reached a natural stopping point where remaining candidates require:
1. Complex struct FFI (win_T, buf_T, frame_T)
2. Global state access patterns
3. libuv integration for OS functions
4. Event loop/callback infrastructure

**Next Phase Options:**
1. **Phase 2A**: Build Rust FFI for complex structs (win_T, buf_T)
2. **Phase 2B**: Wrap libuv-sys for OS layer parity
3. **Phase 2C**: Migrate non-PURE functions that are otherwise simple

## Session 6 Summary (2025-12-04)

**Extended search for Phase 2 candidates:**

Searched for non-PURE functions that might still be simple:
- OS functions (`os/input.c`, `os/stdpaths.c`, etc.) - all use libuv or global state
- String comparison functions (`vim_stricmp`, `vim_strnicmp`) - use `TOLOWER_LOC` locale macro
- Eval functions (`encode_check_json_key`, `find_internal_func`) - use `typval_T*` structs or global arrays
- Register functions in headers - already inline, minimal benefit from migration

**Inline functions reviewed (in headers):**
- `is_literal_register`, `op_reg_index`, `is_append_register`, `get_register_name` - pure but already inlined
- `ascii_is*` functions - header inlines, already optimized
- `mt_*` marktree functions - use `MTKey` struct

**Confirmed findings:**
- All non-inlined PURE/CONST functions are either migrated or unsuitable
- Header inline functions provide minimal benefit from migration
- Remaining candidates fall into categories requiring infrastructure work

**Phase 2 would require choosing one of:**
1. **Struct FFI**: Define opaque handle patterns for win_T, buf_T
2. **libuv binding**: Add libuv-sys crate and wrap OS functions
3. **Global state bridge**: Create Rust access patterns for g_chartab, curbuf, etc.

Each option requires significant infrastructure work before additional functions can migrate.

## Session 7 Summary (2025-12-04)

**Phase 2.2**: Successfully swapped all hashtab operations to Rust:
- `hash_init`, `hash_clear` - table lifecycle
- `hash_find`, `hash_find_len`, `hash_lookup` - item lookup
- `hash_add_item`, `hash_remove` - item mutation
- `hash_lock`, `hash_unlock` - resize locking

Key technical detail: Rust imports the C global `hash_removed` to ensure both implementations use the same marker for removed items.

**Current Status**:
- Phase 1 (pure functions): COMPLETE
- Phase 2.1 (garray): COMPLETE
- Phase 2.2 (hashtab): COMPLETE

**Next migration opportunities**:
All trivial pure functions and data structures have been migrated. Remaining options:
1. **OS filesystem functions**: Rust implementations exist in fs.rs but use `std::fs` instead of libuv - may have subtle behavioral differences
2. **Struct FFI**: Would enable window.c frame functions, buffer.c validation
3. **Global state access**: Would enable g_chartab-dependent functions

The migration has reached a natural plateau. Further progress requires choosing an infrastructure investment.

**Phase 2.3**: Successfully swapped two OS filesystem functions to Rust:
- `os_path_exists` - Check if a path exists
- `os_isdir` - Check if a path is a directory

These use Rust's `std::fs` instead of libuv. The 90 filesystem unit tests pass, confirming behavioral equivalence for standard use cases.

**Phase 2.4**: Swapped one more OS filesystem function:
- `os_file_is_readable` - Check if a file is readable (uses `fs::File::open` instead of `access(R_OK)`)

**Phase 2.5**: Swapped one more OS filesystem function:
- `os_isrealdir` - Check if a path is a real directory (not a symlink to one). Uses `symlink_metadata` (lstat) in Rust.

**Phase 2.6**: Swapped one more OS filesystem function:
- `os_file_is_writable` - Returns 0 (not writable), 1 (file writable), or 2 (directory writable). Uses append mode to test actual write access.

**Phase 2.7**: Swapped one more OS filesystem function:
- `os_dirname` - Get current working directory. Uses `std::env::current_dir()` instead of libuv's `uv_cwd()`.

**Phase 2.8**: Swapped one more OS filesystem function:
- `os_rename` - Rename/move a file or directory. Returns OK/FAIL. Uses `std::fs::rename()` instead of libuv's `uv_fs_rename()`.

**Phase 2.9**: Swapped one more OS filesystem function:
- `os_setperm` - Set file permissions (mode bits). Uses `std::fs::set_permissions()` with `PermissionsExt::from_mode()` on Unix. Returns OK/FAIL.

**Phase 2.10**: Swapped one more OS filesystem function:
- `os_getperm` - Get file permissions (mode bits). Uses `std::fs::metadata()` with `MetadataExt::mode()` on Unix. Returns libuv-compatible error codes on failure (e.g., UV_ENOENT = -2).
- Added `io_error_to_uv_error` helper function for consistent error code translation.

**Phase 2.11**: Swapped one more OS filesystem function:
- `os_remove` - Remove a file. Uses `std::fs::remove_file()`. Returns 0 on success, libuv-compatible error code on failure.

**Phase 2.12**: Swapped one more OS filesystem function:
- `os_rmdir` - Remove an empty directory. Uses `std::fs::remove_dir()`. Returns 0 on success, libuv-compatible error code on failure.

**Phase 2.13**: Swapped one more OS filesystem function:
- `os_mkdir` - Create a directory with specified mode. Uses `std::fs::DirBuilder` with `DirBuilderExt::mode()` on Unix. Returns 0 on success, libuv-compatible error code on failure.
