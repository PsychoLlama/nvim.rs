# Neovim C-to-Rust Migration

## Crate Structure

All Rust code in `src/nvim-rs/`. Each crate handles a specific domain:

| Crate        | Purpose                                            |
| ------------ | -------------------------------------------------- |
| api          | API types (Object, Dict, Array), type conversions  |
| arabic       | Arabic text shaping, combining char support        |
| ascii        | ASCII character predicates                         |
| autocmd      | Autocommand state, pattern parsing, event checking |
| buffer       | Buffer type/state queries, bufref validation       |
| buffer_updates | Buffer update tracking                           |
| charset      | Character classification, skip*, str2nr, case folding |
| cmdhist      | History type conversion                            |
| cmdline      | Command line state queries                         |
| collections  | Data structures (garray, hashtab)                  |
| context      | Context stack management                           |
| cursor_shape | Cursor mode/shape queries                          |
| decoration   | Decoration/virtual text system, DecorState access  |
| diff         | Diffopt flag queries                               |
| drawline     | Line drawing and rendering helpers                 |
| drawscreen   | Window separator drawing, connector functions      |
| edit         | Edit mode state queries                            |
| encoding     | Base64, SHA-256                                    |
| eval         | VimL name validation, tristate                     |
| event        | Event loop, libuv wrappers, multiqueue             |
| ex_docmd     | Ex command parsing helpers                         |
| ex_eval      | Exception state checking                           |
| fileio       | File time comparison                               |
| fold         | Fold method checks, fold state queries             |
| fuzzy        | Fuzzy matching algorithm (fzy-based)               |
| getchar      | Typeahead buffer state                             |
| grid         | Screen character (schar_T) operations              |
| help         | Help tag heuristics                                |
| highlight    | Color/attr system, HlEntry storage, namespace mgmt |
| indent       | Tab/indent size calculation                        |
| insexpand    | CTRL-X completion mode checking                    |
| keycodes     | Key modifier parsing                               |
| linematch    | Line matching algorithm for diff alignment         |
| lua          | Lua executor state, callback FFI                   |
| mark         | Mark indices, position comparison                  |
| math         | Floating-point classification, bit ops             |
| mbyte        | UTF-8/multibyte encoding, char width               |
| memory       | xmalloc/xfree wrappers                             |
| memutil      | Memory/string utilities                            |
| menu         | Menu type classification                           |
| msgpack      | MessagePack serialization primitives               |
| ops          | Operator type queries                              |
| os           | OS abstractions (env, time, fs)                    |
| path         | Path manipulation, separators                      |
| plines       | Physical line display calculations                 |
| popupmenu    | Popup menu visibility state queries                |
| profile      | Profiling time arithmetic                          |
| register     | Register name validation                           |
| search       | Character search state, search/substitute tracking |
| spell        | Spell check utilities                              |
| statusline   | Status line rendering helpers                      |
| strings      | String comparison, case conversion, escape utils   |
| tui          | Terminal UI, terminfo formatting                   |
| typval       | VimL typval_T type checking and value extraction   |
| ugrid        | Unicode grid (UGrid) operations for TUI            |
| unpacker     | MessagePack unpacking                              |
| utf8proc     | utf8proc FFI bindings                              |
| version      | Version compatibility checks                       |
| window       | Window state, validation, frame layout             |

## Remaining C Code (By Design)

### Highlight

- `dict2hlattrs` - Uses generated API keyset `Dict(highlight)`, complex to port
- `ns_get_hl` callback logic - Blocked by dict2hlattrs
- `highlight_changed` - Calls into syntax/screen subsystems

### Grid

- `grid_alloc/grid_free` - Tightly coupled to C memory management
- `win_grid_alloc` - Window system integration
- `grid_draw_border` - VirtText iteration, decoration system FFI complexity

## Functions Requiring Infrastructure

### Complex Struct FFI (win_T, buf_T)

**window.c** (frame tree functions - MIGRATED):
- `frame_has_win` - Migrated to Rust (rs_frame_has_win)
- `is_bottom_win` - Migrated to Rust (rs_is_bottom_win)
- `frame_fixed_height/width` - Migrated to Rust
- `frame2win` - Migrated to Rust (rs_frame2win)
- `frame_check_height/width` - Migrated to Rust
- `frame_minheight/minwidth` - Migrated to Rust (rs_frame_minheight, rs_frame_minwidth)
- `win_comp_pos/frame_comp_pos` - Migrated to Rust (rs_win_comp_pos, rs_frame_comp_pos)
- `frame_setheight/setwidth` - Migrated to Rust (rs_frame_setheight, rs_frame_setwidth)
- `win_setheight_win/setwidth_win` - Migrated to Rust (rs_win_setheight_win, rs_win_setwidth_win)
- `frame_add_height` - Migrated to Rust (rs_frame_add_height)
- `frame_add_statusline` - Migrated to Rust (rs_frame_add_statusline)
- `frame_set_vsep` - Migrated to Rust (rs_frame_set_vsep)
- `frame_add_hsep` - Migrated to Rust (rs_frame_add_hsep)
- `frame_fix_width/height` - Migrated to Rust (rs_frame_fix_width, rs_frame_fix_height)

**drawscreen.c** (separator functions - MIGRATED):
- `hsep_connected/vsep_connected` - Migrated to Rust
- `draw_vsep_win/draw_hsep_win` - Migrated to Rust
- `get_corner_sep_connector` - Migrated to Rust
- `draw_sep_connectors_win` - Migrated to Rust

**window.c** (remaining):
- `tabpage_win_valid` - Window in tabpage validation
- `win_valid_any_tab` - Window validation across tabs

**buffer.c**:
- `buf_valid` - Buffer pointer validation (needs buffer list)

**plines.c** (display calculations - partially MIGRATED):
- `win_chartabsize` - Migrated to Rust (rs_win_chartabsize)
- `charsize_fast` - Migrated to Rust (rs_charsize_fast)
- `linesize_fast` - Migrated to Rust (rs_linesize_fast)
- `in_win_border` - Migrated to Rust (rs_in_win_border)
- `charsize_regular` - Migrated to Rust (rs_charsize_regular) - includes marktree accessor pattern
- `linesize_regular` - Migrated to Rust (rs_linesize_regular) - includes character iteration accessors
- `getvcol` - Migrated to Rust (rs_getvcol) - includes visual mode accessors
- `plines_win_nofold` - Migrated to Rust (rs_plines_win_nofold)

### Global State Dependencies

**cursor_shape.c** (shape_table global):
- `cursor_is_block_during_visual`
- `cursor_mode_uses_syn_id`
- `cursor_get_mode_idx`

**version.c** (static version arrays):
- `min_vim_version`, `highest_patch`, `has_vim_patch` - need vim_versions/included_patchsets arrays

**textformat.c**:
- `has_format_option` - Uses p_paste and curbuf->b_p_fo

### Static Data Tables

**digraph.c**:
- `digraph_get`, `getexactdigraph` - Use user_digraphs global and static table

**keycodes.c**:
- `get_special_key_code` - Uses key_names_table static array

**autocmd.c**:
- `event_nr2name` - Uses event_names static array

### Blocked OS Functions

| Function | Blocker |
|----------|---------|
| `os_chdir` | Uses verbose_enter/leave, ui_call_chdir |
| `os_fsync` | Updates g_stats.fsync counter |
| `os_can_exe` | PATH searching with helper functions |
| `os_mkdir_recurse` | Uses xmalloc, path helpers |
| `os_readv` | Uses struct iovec (vectored I/O) |

## Recent Progress: Rendering System Migration

### drawline.c Migrated Functions

- `advance_color_col` - Color column pointer advancement (rs_advance_color_col)
- `draw_col_fill` - Fill column with character (rs_draw_col_fill)
- `use_cursor_line_highlight` - Cursorline highlight check (rs_use_cursor_line_highlight)
- `draw_foldcolumn` - Foldcolumn setup (rs_draw_foldcolumn)
- `fill_foldcolumn` - Foldcolumn filling (rs_fill_foldcolumn)
- `line_putchar` - UTF-8 char to screen char (rs_line_putchar)
- `draw_col_buf` - Draw character buffer to line (rs_draw_col_buf)
- `apply_cursorline_highlight` - Apply cursorline highlight to line (rs_apply_cursorline_highlight)
- `set_line_attr_for_diff` - Set line attr for diff mode (rs_set_line_attr_for_diff)
- `handle_breakindent` - Breakindent for wrapped text (rs_handle_breakindent)
- `handle_showbreak_and_filler` - Showbreak and filler lines (rs_handle_showbreak_and_filler)
- `has_more_inline_virt` - Check for more inline virtual text (rs_has_more_inline_virt)
- `handle_inline_virtual_text` - Inline virtual text processing (rs_handle_inline_virtual_text)
- `wlv_put_linebuf` - Put line buffer to screen grid (rs_wlv_put_linebuf)

### drawline.c Accessor Functions (for Rust access to winlinevars_T)

Added ~30 accessor functions for winlinevars_T fields:
- `nvim_wlv_get/set_vcol`, `nvim_wlv_get/set_col`, `nvim_wlv_get/set_off`
- `nvim_wlv_get/set_n_extra`, `nvim_wlv_get/set_p_extra`, `nvim_wlv_get/set_sc_extra`
- `nvim_wlv_get/set_char_attr`, `nvim_wlv_get/set_n_attr`, `nvim_wlv_get/set_extra_attr`
- `nvim_wlv_get/set_color_cols`, `nvim_wlv_get/set_line_attr`
- And more for virtual text, skip cells, boguscols, etc.

### decoration.c Migrated Functions

- `decor_virt_pos` - Check if decor has virtual position (rs_decor_virt_pos)
- `decor_virt_pos_kind` - Get virtual text position kind (rs_decor_virt_pos_kind)

### decoration.c Accessor Functions (for Rust access to DecorState)

Added ~25 accessor functions for decoration system:

**DecorState accessors:**
- `nvim_get_decor_state` - Get global decor_state
- `nvim_decor_state_get/set_eol_col`, `nvim_decor_state_get_row`
- `nvim_decor_state_get_current_end`, `nvim_decor_state_get_current`
- `nvim_decor_state_get_conceal`, `nvim_decor_state_get_conceal_char`
- `nvim_decor_state_get_win`, `nvim_decor_state_get_range`

**DecorRange accessors:**
- `nvim_decor_range_get_start_row/col`, `nvim_decor_range_get_end_row/col`
- `nvim_decor_range_get/set_draw_col`, `nvim_decor_range_get_kind`
- `nvim_decor_range_has_virt_pos`, `nvim_decor_range_get_virt_pos_kind`
- `nvim_decor_range_get_virt_text`

**DecorVirtText accessors:**
- `nvim_decor_virt_text_get_hl_mode`, `nvim_decor_virt_text_get_pos`
- `nvim_decor_virt_text_get_width`, `nvim_decor_virt_text_get_flags`
- `nvim_decor_virt_text_get_chunk_count/text/hl_id`

### drawline.c Virtual Text Functions

- `draw_virt_text_item` - Individual virtual text chunk rendering (rs_draw_virt_text_item)
- `draw_virt_text` - Virtual text positioning loop (rs_draw_virt_text)

### drawline.c Line Initialization Functions

- `win_line_start` - Initialize line buffer for rendering (rs_win_line_start)
- `fix_for_boguscols` - Adjust columns for bogus chars (rs_fix_for_boguscols)

### Rendering Functions Pending

**drawscreen.c:**
- `showmode` - Mode indicator display (message system integration)
- `update_screen` - Screen orchestration (large, many side effects)

**statusline.c:**
- `win_redr_status` - Status line redraw
- `draw_tabline` - Tab line rendering
