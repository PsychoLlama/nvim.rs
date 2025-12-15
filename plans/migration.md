# Neovim C-to-Rust Migration Plan

## Current Status

**716 rs\_\* functions migrated**

Run `grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l` to get current count.

### Phase 36: Grid Scrolling ✅ COMPLETE

Rust implementations for grid scrolling functions:
- `rs_grid_ins_lines` - insert lines by scrolling down
- `rs_grid_del_lines` - delete lines by scrolling up
- `linecopy_impl` - copy portion of line within grid (internal)

C wrapper: `nvim_ui_call_grid_scroll()` for UI scroll notifications.

**Note:** Phase 35 (grid allocation) skipped - memory allocation functions are tightly coupled to C's xmalloc/xfree.

**Next:** Phase 37 (window grid) or remaining grid.c functions.

### Phase 34: Grid Operations ✅ COMPLETE

Rust implementations for grid manipulation functions:
- `rs_grid_adjust` - viewport coordinate adjustment
- `rs_grid_clear_line` - clear line with spaces
- `rs_grid_invalidate` - mark all rows invalid
- `rs_grid_getchar` - get character from grid
- `rs_grid_clear` - clear rectangular region

C accessors added (3 new): `nvim_gridview_get_target/row_offset/col_offset`.

### Phase 33: Arabic Shaping ✅ COMPLETE

Rust implementation of Arabic contextual shaping for display lines:
- `rs_line_do_arabic_shape` - apply Arabic shaping to line buffer

C wrapper: `nvim_arabic_shape()` in arabic.c for Rust to call `arabic_shape()`.

### Phase 32: grid_put_linebuf Implementation ✅ COMPLETE

Full Rust implementation of the core rendering function (~170 lines):
- `rs_grid_put_linebuf` - move buffered line to grid with delta detection

C accessors added (15 new) for grid arrays, fields, globals, and functions.

### Phase 31: Grid Line Flush Functions ✅ COMPLETE

Added Rust implementations for grid line flushing:
- `rs_grid_line_flush` - commit line buffer to UI
- `rs_grid_line_flush_if_valid_row` - safe flush with row validation

C accessors added:
- `nvim_screengrid_get_rows()` - get grid row count
- `nvim_get_rdb_flags()` - get rdb_flags global

### Phase 30: Grid Line Content Functions ✅ COMPLETE

Added Rust implementations for grid line content manipulation:
- `rs_grid_line_put_schar` - put single schar at column
- `rs_grid_line_fill` - fill column range with schar
- `rs_grid_line_clear_end` - set clear range for line
- `rs_grid_line_cursor_goto` - move cursor to column

C accessor: `nvim_screengrid_get_handle()` for ScreenGrid handle.

### Phase 29: Grid Line State Accessors ✅ COMPLETE

Added C accessors for grid_line_* static state (10 get/set pairs + 1 UI wrapper):
- Grid pointer, row, coloff, maxcol
- first, last, clear_to, bg_attr, clear_attr, flags
- `nvim_ui_grid_cursor_goto()` - UI cursor positioning

**Next:** Phase 30 (grid line content functions).

### Phase 28: Line Buffer Infrastructure ✅ COMPLETE

Added C accessors for line buffer globals:
- `nvim_get_linebuf_char()` - linebuf_char array
- `nvim_get_linebuf_attr()` - linebuf_attr array
- `nvim_get_linebuf_vcol()` - linebuf_vcol array
- `nvim_get_linebuf_scratch()` - scratch buffer
- `nvim_get_linebuf_size()` - buffer size

### Phase 27: schar_get Functions ✅ COMPLETE

Added schar to string conversion functions:
- `rs_schar_get` - convert schar to NUL-terminated UTF-8 string
- `rs_schar_get_adv` - convert schar to UTF-8, advancing buffer pointer

### Phase 26: Glyph Cache Rust Implementation ✅ COMPLETE

Full Rust rewrite of the glyph cache (replacing C's `Set(glyph)`):

- `GlyphCache` struct with HashMap for glyph storage and lookup
- `rs_schar_from_buf` - buffer to schar (writes to Rust cache)
- `rs_schar_cache_clear_if_full` - check/clear if >2^21 entries
- `rs_schar_cache_clear` - clear cache, call callbacks

C accessor wrappers for Rust to call:
- `nvim_decor_check_invalid_glyphs()` - invalidate decoration glyphs
- `nvim_check_chars_options()` - regenerate char options

**Next:** Phase 27 (schar_get functions), then remaining grid.c functions.

### Phase 25: schar_T Core Functions ✅ COMPLETE

Added 4 schar_T functions to nvim-grid crate:

- `rs_schar_from_str` - string to schar conversion
- `rs_schar_len` - get byte length
- `rs_schar_cells` - get display width (1 or 2)
- `rs_schar_get_first_codepoint` - extract first Unicode codepoint

Added C accessor `nvim_glyph_cache_get()` for Rust to read from glyph cache.

### Phase 24: Lua Callback FFI ✅ COMPLETE

Added FFI so Rust can call Lua callbacks via `nlua_call_ref()`:

- C wrappers: `nvim_nlua_call_ref()`, `nvim_nlua_call_ref_ctx()` in executor.c
- Rust: `lua_call_ref()`, `lua_call_ref_ctx()` in nvim-lua crate
- Re-exports API types (Object, Array, Error, LuaRef)

**Unlocks:** 12 files with callback dependencies (highlight.c, decoration_provider.c, autocmd.c, etc.)

### Highlight Migration: ✅ COMPLETE

All core highlight functions have Rust implementations via `USE_RUST_HIGHLIGHT`:

- Entry storage, caches, URL storage (Rust only)
- Namespace storage and logic (Rust only)
- Core computation, attribute combination, UI lookup
- API conversion (hlattrs2dict, hl_inspect, object_to_color)

**Remaining C (low priority):**

- `dict2hlattrs` - Uses generated API keyset `Dict(highlight)`, complex to port
- `ns_get_hl` callback logic - Blocked by dict2hlattrs (uses it to parse Lua result)
- `highlight_changed` - Calls into syntax/screen subsystems

---

## Architecture

### Crate Structure

All Rust code in `src/nvim-rs/`. Complete crate list:

| Crate        | Purpose                                            |
| ------------ | -------------------------------------------------- |
| api          | API types (Object, Dict, Array), type conversions  |
| arabic       | Arabic combining char support                      |
| ascii        | ASCII character predicates                         |
| autocmd      | Autocommand state, pattern parsing                 |
| buffer       | Buffer type/state queries                          |
| charset      | Character classification, skip functions           |
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
| os           | OS abstractions (env, time)                        |
| path         | Path manipulation, separators                      |
| profile      | Profiling time arithmetic                          |
| register     | Register name validation                           |
| search       | Character search state                             |
| spell        | Spell check utilities                              |
| strings      | String comparison, case conversion                 |
| tui          | Terminal UI, terminfo formatting                   |
| unpacker     | MessagePack unpacking                              |
| utf8proc     | utf8proc FFI bindings                              |
| version      | Version compatibility checks                       |
| window       | Window state queries, validation                   |

### Build System

- CMake: `USE_RUST_*` flags in `src/nvim/CMakeLists.txt`
- `cbindgen` generates headers
- Static library `libnvim_rs.a`

### Key Patterns

**Opaque Handle**: C accessor for struct fields

```c
int nvim_get_foo_field(void) { return foo.field; }
```

**Conditional Compilation**:

```c
#ifdef USE_RUST_FOO
  return rs_foo();
#else
  // C fallback
#endif
```

---

## Quick Commands

```bash
# Count functions
grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l

# Find functions in a crate
grep -n "pub.*extern.*fn rs_" src/nvim-rs/highlight/src/lib.rs

# Check USE_RUST flags
grep "USE_RUST_" src/nvim/CMakeLists.txt
```
