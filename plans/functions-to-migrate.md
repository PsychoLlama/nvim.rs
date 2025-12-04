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
| nvim-os | ✅ Partial | os_get_pid (Phase 1.26), os_get_hostname (Phase 1.27) - More functions ready but blocked by libuv differences |
| nvim-collections (hashtab) | ✅ Partial | hash_hash, hash_hash_len - Full hashtab not swapped |

### Unswapped Crates (Rust code exists but NOT used from C)
| Crate | Status | Blocker |
|-------|--------|---------|
| nvim-os (env/fs) | 🔧 Ready but blocked | Memory allocator fixed (uses NvimString), but C uses libuv for env functions |
| nvim-collections (garray) | ❌ Not swapped | Complex data structure, needs careful C integration |

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
