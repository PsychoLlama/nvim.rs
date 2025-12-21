# Neovim C-to-Rust Migration

## Crate Structure

All Rust code in `src/nvim-rs/`. Each crate handles a specific domain:

| Crate        | Purpose                                            |
| ------------ | -------------------------------------------------- |
| api          | API types (Object, Dict, Array), type conversions  |
| arabic       | Arabic combining char support                      |
| ascii        | ASCII character predicates                         |
| autocmd      | Autocommand state, pattern parsing                 |
| buffer       | Buffer type/state queries                          |
| charset      | Character classification, skip*, str2nr, case folding |
| cmdhist      | History type conversion                            |
| cmdline      | Command line state queries                         |
| collections  | Data structures (garray, hashtab)                  |
| context      | Context stack management                           |
| cursor_shape | Cursor mode/shape queries                          |
| diff         | Diffopt flag queries                               |
| encoding     | Base64, SHA-256                                    |
| eval         | VimL name validation, tristate                     |
| event        | Event loop, libuv wrappers, multiqueue             |
| ex_docmd     | Ex command parsing helpers                         |
| ex_eval      | Exception state checking                           |
| fileio       | File time comparison                               |
| fuzzy        | Fuzzy matching algorithm (fzy-based)               |
| getchar      | Typeahead buffer state                             |
| grid         | Screen character (schar_T) operations              |
| help         | Help tag heuristics                                |
| highlight    | Color/attr system, HlEntry storage, namespace mgmt |
| indent       | Tab/indent size calculation                        |
| insexpand    | CTRL-X completion mode checking                    |
| keycodes     | Key modifier parsing                               |
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
| profile      | Profiling time arithmetic                          |
| register     | Register name validation                           |
| search       | Character search state                             |
| spell        | Spell check utilities                              |
| strings      | String comparison, case conversion, escape utils   |
| tui          | Terminal UI, terminfo formatting                   |
| typval       | VimL typval_T type checking and value extraction   |
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

### Complex Struct FFI (win_T, buf_T, frame_T)

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
