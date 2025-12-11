# Neovim C-to-Rust Migration Plan

## Current Status

**588 rs_* functions migrated** (as of 2025-12-11)

Run `grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l` to get current count.

### Recent Work (December 2025)

**Phase 7.2 - API Free Functions** (completed)
- Added recursive free functions to nvim-api crate:
  - `rs_api_free_object`: Recursively free an Object and all nested data
  - `rs_api_free_array`: Free Array and all items recursively
  - `rs_api_free_dict`: Free Dict and all key-value pairs recursively

**Phase 7.0-7.1 - API Helper Functions** (completed)
- Added Rust implementations to nvim-api crate:
  - `rs_api_typename`: Get type name string for ObjectType
  - `rs_cchar_to_string`: Single char to String
  - `rs_cstr_to_string`: Copy C string to String
  - `rs_cstr_as_string`: C string to String (no copy)
  - `rs_cbuf_to_string`: Copy buffer to String
  - `rs_cstrn_to_string`: Copy C string with maxsize
  - `rs_cstrn_as_string`: C string with maxsize (no copy)
  - `rs_api_free_string`: Free String data
  - `rs_api_object_to_bool`: Coerce Object to bool
  - `rs_copy_string`: Copy String with arena support
  - `rs_api_clear_error`: Clear error and free message
  - `rs_error_set`: Check if error is set
  - `rs_string_to_cstr`: Copy String to NUL-terminated C string
  - `rs_ga_take_string`: Take garray memory as String

**Phase 6.0 - Rust MessagePack Unpacker** (completed)
- Implemented pure Rust msgpack unpacker using `rmp` crate
- New crate: nvim-unpacker in src/nvim-rs/unpacker/
- Replaces C mpack-based unpack() function with rs_unpack()
- 48 USE_RUST_* flags now enabled (added USE_RUST_UNPACKER)

**Previous Work**:
- Phase 5.1-5.31 - Static variable accessor pattern (569 functions)
- Phases 0-4 - Build infrastructure, memory bridge, OS functions, event loop

---

## Architecture

### Rust Crate Structure

All Rust code lives in `src/nvim-rs/`. The main crate re-exports all FFI functions.

| Crate | Purpose | Key Functions |
|-------|---------|---------------|
| nvim-api | API helpers | buffer/window option validation |
| nvim-arabic | Arabic text | combining chars, shaping |
| nvim-ascii | ASCII classification | isdigit, isspace, toupper, etc. |
| nvim-autocmd | Autocommands | is_autocmd_blocked |
| nvim-buffer | Buffer operations | bt_* buffer type checks |
| nvim-charset | Character sets | chartab lookups, hex/nr conversion |
| nvim-cmdline | Command line | overstrike, at_end |
| nvim-cmdhist | Command history | hist_char2type |
| nvim-collections | Data structures | garray, hashtab, queue |
| nvim-context | Context stack | ctx_size |
| nvim-cursor-shape | Cursor shapes | cursor_is_block_during_visual, cursor_mode_uses_syn_id |
| nvim-diff | Diff options | diffopt_* flags |
| nvim-encoding | Base64, SHA256 | encode/decode |
| nvim-eval | Eval helpers | num_divide, num_modulus |
| nvim-event | Event loop | multiqueue, watchers, loop accessors |
| nvim-ex-docmd | Ex commands | ends_excmd, find_nextcmd, is_loclist_cmd |
| nvim-ex-eval | Exception handling | aborted_in_try |
| nvim-fileio | File I/O | time_differs |
| nvim-getchar | Typeahead buffer | stuff_empty, typebuf_*, using_script |
| nvim-grid | Screen grid | schar operations |
| nvim-help | Help system | help tag lookup |
| nvim-indent | Indentation | tabstop calculations |
| nvim-insexpand | Insert completion | ctrl_x_mode_*, compl_status_* |
| nvim-keycodes | Key codes | name_to_mod_mask, handle_x_keys |
| nvim-lua | Lua integration | nlua_is_deferred_safe |
| nvim-mark | Marks/positions | lt, equalpos, mark_*_index |
| nvim-math | Math utilities | xfpclassify, xisinf, xisnan |
| nvim-mbyte | Multibyte/UTF-8 | utf_* functions |
| nvim-memutil | Memory utilities | hash functions |
| nvim-menu | Menu system | menu helpers |
| nvim-msgpack | MessagePack | mpack_* packer functions |
| nvim-unpacker | MessagePack | rs_unpack unpacker |
| nvim-ops | Operators | format helpers |
| nvim-os | OS functions | env, fs, time (43 functions) |
| nvim-path | Path utilities | path manipulation |
| nvim-profile | Profiling | profile_* time functions |
| nvim-register | Registers | register index/validation |
| nvim-search | Search | search helpers |
| nvim-spell | Spell check | langmap, langword |
| nvim-strings | String utilities | vim_str* functions |
| nvim-utf8proc | UTF-8 processing | utf8proc bindings |
| nvim-version | Version info | version string |
| nvim-window | Window operations | win_valid, frame_* functions |

### Build System

- CMake integrates Rust via `USE_RUST_*` flags in `src/nvim/CMakeLists.txt`
- All `USE_RUST_*` flags are enabled by default
- `cbindgen` generates C headers from Rust code
- Static library `libnvim_rs.a` linked into nvim binary

### Key Patterns

**Opaque Handle Pattern**: For accessing C struct fields from Rust:
```c
// C accessor in foo.c
int nvim_get_foo_field(void) { return foo.field; }

// Rust FFI
extern "C" { fn nvim_get_foo_field() -> c_int; }
```

**Conditional Compilation**:
```c
#ifdef USE_RUST_FOO
  return rs_foo();
#else
  // original C implementation
#endif
```

---

## Migration Phases

### Completed

| Phase | Description | Status |
|-------|-------------|--------|
| 0 | Build infrastructure (Cargo, CMake, cbindgen) | ✅ |
| 0.5 | Memory allocation bridge (NvimBox, NvimVec) | ✅ |
| 1 | Pure utility functions (math, encoding, strings) | ✅ |
| 2 | OS & data structures (garray, hashtab, fs) | ✅ |
| 3 | Complex struct FFI (window, buffer, frame handles) | ✅ |
| 4 | Event loop accessors (watchers, streams, loop fields) | ✅ |
| 5.1-5.31 | Static variable accessor pattern | ✅ |
| 6.0 | MessagePack unpacker (rs_unpack) | ✅ |
| 7.0 | API helper functions (string, typename) | ✅ |

### Phase 6 Complete

**Phase 6.0** replaced the C mpack-based unpacker with a pure Rust implementation using the `rmp` crate. The Rust unpacker:
- Handles all msgpack types (nil, bool, int, float, string, binary, array, map, ext)
- Uses arena_alloc() for memory allocation (compatible with C code)
- Properly handles Buffer/Window/Tabpage EXT types

All 48 `USE_RUST_*` flags are now enabled.

### Next Phase

Focus on deeper integration work or expand existing crates with non-pure functions that are still amenable to FFI.

### Future Phases (Roadmap)

| Phase | Target | Notes |
|-------|--------|-------|
| 7 | API Layer | api/*.c |
| 8 | Terminal UI | tui/*.c |
| 9 | Buffer & Text | memline.c, buffer.c, undo.c |
| 10 | Window & Display | window.c, drawscreen.c |
| 11-15 | Editor Core | normal.c, eval.c, regexp.c |
| 16 | Final Cleanup | Remove FFI, reduce unsafe |

---

## Testing

```bash
# Quick validation
just build && cargo clippy

# Run specific test file
TEST_FILE=test/functional/api/buffer_spec.lua cmake --build build --target functionaltest

# Full test suite
just test
```

---

## Key Decisions

1. **Opaque handles over bindgen**: The accessor pattern is more incremental and avoids bindgen complexity.

2. **Keep libuv**: Wrap libuv with Rust accessors rather than replacing with tokio (for now).

3. **Static initializers stay in C**: Macros like `RGB_()` can't call functions, so they remain in C.

4. **Test-driven**: Every migration must pass existing functional tests.

---

## Quick Commands

```bash
# Count migrated functions
grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l

# Find functions in a crate
grep -n "pub.*extern.*fn rs_" src/nvim-rs/buffer/src/lib.rs

# Check USE_RUST flags
grep "USE_RUST_" src/nvim/CMakeLists.txt

# List all crates
ls src/nvim-rs/
```
