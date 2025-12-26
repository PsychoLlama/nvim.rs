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
| diff         | Diffopt flag queries                               |
| drawline     | Line drawing (lcs_ext, margin_columns, foldcolumn, rightmost_vcol, draw_col_fill, draw_foldcolumn, draw_sign, draw_lnum_col, cursor_line_highlight, line_number_attr) |
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
| plines       | Display calculations, scrolloff, showbreak, win_col_off |
| popupmenu    | Popup menu visibility state queries                |
| profile      | Profiling time arithmetic                          |
| register     | Register name validation                           |
| search       | Character search state, search/substitute tracking |
| spell        | Spell check utilities                              |
| statusline   | Status line helpers (stl_connected, fillchar_status, col_print, tabwidth_calc) |
| strings      | String comparison, case conversion, escape utils   |
| tui          | Terminal UI, terminfo formatting                   |
| typval       | VimL typval_T type checking and value extraction   |
| ugrid        | Unicode grid (UGrid) operations for TUI            |
| unpacker     | MessagePack unpacking                              |
| utf8proc     | utf8proc FFI bindings                              |
| version      | Version compatibility checks                       |
| window       | Window state queries, validation                   |

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

**plines.c** (display calculations):
- `charsize_fast_impl` - Character display width (uses win_T*)
- `in_win_border` - Window border check

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
