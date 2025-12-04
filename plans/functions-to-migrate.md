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
| nvim-ex_docmd | ✅ Swapped | ends_excmd |
| nvim-fileio | ✅ Swapped | time_differs |

### Unswapped Crates (Rust code exists but NOT used from C)
| Crate | Status | Blocker |
|-------|--------|---------|
| nvim-os | ❌ Not swapped | Uses Rust allocator, needs xmalloc integration; C uses libuv |
| nvim-collections (garray) | ❌ Not swapped | Complex data structure, needs careful C integration |
| nvim-collections (hashtab) | Partial | Only hash functions swapped, not full hashtab |

### Migration Blockers

1. **Memory allocation mismatch**: OS crate allocates with Rust `CString`, but nvim uses `xmalloc`. Returning allocated strings requires using nvim's allocator.

2. **libuv dependency**: OS/filesystem functions in C use libuv for portability. Rust would need to either wrap libuv or replace it entirely.

3. **Complex struct types**: frame_T, win_T, buf_T have deep pointer hierarchies. Simple FFI doesn't work; need either opaque pointers with callbacks or full struct mirroring.
