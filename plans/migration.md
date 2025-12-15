# Neovim C-to-Rust Migration Plan

## Current Status

**701 rs\_\* functions migrated**

Run `grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l` to get current count.

### Phase 27: schar_get Functions ✅ COMPLETE

Added schar to string conversion functions:
- `rs_schar_get` - convert schar to NUL-terminated UTF-8 string
- `rs_schar_get_adv` - convert schar to UTF-8, advancing buffer pointer

**Next:** Phase 28 (line buffer infrastructure).

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
