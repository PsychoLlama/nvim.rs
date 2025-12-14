# Neovim C-to-Rust Migration Plan

## Current Status

**692 rs\_\* functions migrated**

Run `grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l` to get current count.

### Phase 24: Lua Callback FFI ✅ COMPLETE

Added FFI so Rust can call Lua callbacks via `nlua_call_ref()`:

- C wrappers: `nvim_nlua_call_ref()`, `nvim_nlua_call_ref_ctx()` in executor.c
- Rust: `lua_call_ref()`, `lua_call_ref_ctx()` in nvim-lua crate
- Re-exports API types (Object, Array, Error, LuaRef)

**Unlocks:** 12 files with callback dependencies (highlight.c, decoration_provider.c, autocmd.c, etc.)

### Next Target: grid.c

Screen character (`schar_T`) operations. Already has 3 Rust functions, ~37 total functions.

**Target functions:**

- `schar_from_str`, `schar_from_buf` - string to schar conversion
- `schar_get`, `schar_get_adv` - schar to string extraction
- `schar_len`, `schar_cells` - character metrics
- `schar_get_first_codepoint` - Unicode handling
- `schar_cache_*` - glyph cache management
- `grid_*` functions - grid operations (later phases)

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
