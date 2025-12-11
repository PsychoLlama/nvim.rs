# Neovim C-to-Rust Migration Plan

## Current Status

**569 rs_* functions migrated** (as of 2025-12-11)

Run `grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l` to get current count.

### Recent Work (December 2025)

Working on migrating pure functions that access static variables via C accessors.

**Pattern**: For functions that read static/global state, create a C accessor function (e.g., `nvim_get_foo()`) that Rust can call via FFI.

**Recent Phases**:
- Phase 5.22 - `cmdline_fuzzy_complete()` (uses nvim_get_wop_flags accessor)
- Phase 5.23 - `aborting()` (uses nvim_get_did_emsg, nvim_get_got_int, nvim_get_did_throw accessors)
- Phase 5.24 - `should_abort()` (uses nvim_get_trylevel, nvim_get_emsg_silent accessors)
- Phase 5.25 - `is_in_cmdwin()` (uses nvim_get_cmdwin_type, nvim_get_cmdline_type accessors)
- Phase 5.26 - `last_csearch()` (uses nvim_get_lastc_bytes accessor)
- Phase 5.27 - `_hash_key_removed()` (wrapper to existing rs_hash_key_removed)
- Phase 5.28 - Enable USE_RUST_ASCII (ASCII_ISLOWER, ASCII_ISUPPER, TOUPPER_ASC, etc.)
- Phase 5.29 - Enable USE_RUST_QUEUE (QUEUE_EMPTY, QUEUE_INIT, QUEUE_ADD, etc.)
- Phase 5.30 - Enable USE_RUST_API (is_internal_call)
- Phase 5.31 - Enable USE_RUST_MARK (lt, equalpos, ltoreq, clearpos, EMPTY_POS)

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

### Phase 5 Complete

**Phase 5** (static variable accessor pattern) is complete with 569 functions migrated.

All simple `FUNC_ATTR_PURE`/`FUNC_ATTR_CONST` functions are done. All 47 `USE_RUST_*` flags in CMakeLists.txt are enabled. Remaining candidates either:
- Are `static` (internal only, not externally visible)
- Access complex structs (typval_T, win_T, buf_T) requiring extensive FFI
- Access generated arrays (event_names, cmdidx values)
- Iterate over global window/buffer lists
- Access multiple interdependent globals

### Next Phase

**Phase 6 (MessagePack)**: The msgpack packer is already migrated. The unpacker (`unpacker.c`) uses mpack library callbacks and is more complex.

Alternative: Focus on deeper integration work or expand existing crates with non-pure functions that are still amenable to FFI.

### Future Phases (Roadmap)

| Phase | Target | Notes |
|-------|--------|-------|
| 6 | MessagePack RPC | packer.c, unpacker.c → rmp crate |
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
