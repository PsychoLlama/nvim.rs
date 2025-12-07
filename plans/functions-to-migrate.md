# Functions to Migrate

## Status: Simple Function Migration COMPLETE (Phase 2.84)

**113+ functions migrated across 27 Rust crates.**

All FUNC_ATTR_PURE/CONST simple functions have been exhaustively migrated. Remaining candidates require infrastructure investment.

---

## Remaining Candidates (Require Infrastructure)

### Category 1: Complex Struct FFI (win_T, buf_T, frame_T)

**window.c** (frame tree functions):
- `frame_has_win` - Recursive frame/window check
- `is_bottom_win` - Window layout position
- `frame_fixed_height/width` - Frame dimension checks
- `tabpage_win_valid` - Window in tabpage validation
- `win_valid_any_tab` - Window validation across tabs

**buffer.c**:
- `buf_valid` - Buffer pointer validation (needs buffer list)

**plines.c** (display calculations):
- `charsize_fast_impl` - Character display width (uses win_T*)
- `in_win_border` - Window border check

### Category 2: Global State Dependencies

**cursor_shape.c** (shape_table global):
- `cursor_is_block_during_visual`
- `cursor_mode_uses_syn_id`
- `cursor_get_mode_idx`

**version.c** (static version arrays):
- `min_vim_version`, `highest_patch`, `has_vim_patch`, `has_nvim_version`

**textformat.c**:
- `has_format_option` - Uses p_paste and curbuf->b_p_fo

### Category 3: Static Data Tables

**digraph.c**:
- `digraph_get`, `getexactdigraph` - Use user_digraphs global and static table

**keycodes.c**:
- `get_special_key_code` - Uses key_names_table static array

**autocmd.c**:
- `event_nr2name` - Uses event_names static array

### Category 4: Directory Iteration Pattern

**os/fs.c**:
- `os_scandir` / `os_closedir` - Directory iterator with Directory struct

---

## Blocked OS Functions

These OS functions haven't been migrated due to complex dependencies:

| Function | Blocker |
|----------|---------|
| `os_chdir` | Uses verbose_enter/leave, ui_call_chdir |
| `os_fsync` | Updates g_stats.fsync counter |
| `os_can_exe` | PATH searching with helper functions |
| `os_mkdir_recurse` | Uses xmalloc, path helpers |
| `os_readv` | Uses struct iovec (vectored I/O) |

---

## Migrated Crates Summary

| Crate | Key Functions |
|-------|---------------|
| nvim-math | xfpclassify, xctz, xpopcount, num_divide, num_modulus, calc_percentage |
| nvim-charset | skipwhite, skipdigits, hex2nr, vim_isfilec, byte2cells, ptr2cells, char2cells |
| nvim-path | vim_ispathsep, path_tail, path_is_url, path_is_absolute |
| nvim-strings | vim_stricmp, vim_strchr, has_non_ascii, valid_name, vim_strup |
| nvim-mbyte | utf_char2len, utf_ptr2char, utf_char2cells, utf_ptr2cells, utf_fold, mb_strnicmp |
| nvim-memutil | xstrchrnul, strcnt, strequal, hash_hash, time_to_bytes |
| nvim-os | 43 functions: os_path_exists, os_isdir, os_open, os_read, os_write, os_fileinfo, etc. |
| nvim-collections | hashtab (hash_init/find/add/remove), garray (ga_init/grow/concat) |
| nvim-encoding | base64_encode/decode, sha256_* |
| nvim-utf8proc | FFI bindings to utf8proc library |
| nvim-arabic | arabic_combine, arabic_maycombine |
| nvim-grid | schar_high, schar_get_ascii, schar_from_char |
| nvim-ops | op_on_lines, op_is_change, get_op_type, get_op_char |
| nvim-register | valid_yank_reg |
| nvim-spell | spell_valid_case, byte_in_str |
| nvim-eval | eval_isnamec, skip_luafunc_name |
| nvim-ex_docmd | ends_excmd, find_nextcmd, is_loclist_cmd |
| nvim-indent | tabstop_padding, indent_size_ts |
| nvim-keycodes | name_to_mod_mask, handle_x_keys, is_mouse_key |
| nvim-profile | profile_zero/add/sub/cmp/divide |
| nvim-menu | menu_is_winbar/popup/toolbar/separator |
| nvim-help | help_heuristic |
| nvim-cmdhist | hist_char2type, hist_type2char |
| nvim-fileio | time_differs, is_dev_fd_file |

---

## Next Steps (Phase 3 Infrastructure)

To unlock remaining functions, choose one:

1. **Complex Struct FFI**: Define opaque handles for win_T, buf_T, frame_T with accessor callbacks
2. **Static Table Access**: Expose key_names_table, event_names, shape_table via FFI
3. **Directory Iterator**: Implement Rust iterator pattern for os_scandir

Each requires significant infrastructure work before additional functions can migrate.
