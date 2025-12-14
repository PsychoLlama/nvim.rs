# Neovim C-to-Rust Migration Plan

## Current Status

**692 rs_* functions migrated**

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

All Rust code in `src/nvim-rs/`. Key crates:

| Crate | Purpose |
|-------|---------|
| nvim-highlight | Color/attr system, HlEntry store |
| nvim-grid | Screen character (schar_T) ops |
| nvim-mbyte | UTF-8/multibyte |
| nvim-event | Event loop, multiqueue |
| nvim-api | API types (Object, Dict, Array) |
| nvim-lua | Lua callback FFI (lua_call_ref) |
| nvim-tui | Terminal UI, terminfo |

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
