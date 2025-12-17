# Neovim C-to-Rust Migration Plan

## Current Status

**794 rs\_\* functions migrated**

Run `grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l` to get current count.

Latest: Phase 65 completed - `tv_check_num`, `tv_check_str`, `tv_check_str_or_nr` type validators migrated to Rust.

### Highlight Migration: ✅ COMPLETE (Dead Code Removed)

All core highlight functions have Rust implementations. The `USE_RUST_HIGHLIGHT` conditional compilation has been removed - all C fallback code deleted (615 lines removed from highlight.c).

**Remaining C (low priority):**
- `dict2hlattrs` - Uses generated API keyset `Dict(highlight)`, complex to port
- `ns_get_hl` callback logic - Blocked by dict2hlattrs
- `highlight_changed` - Calls into syntax/screen subsystems

### Grid Migration: Phases 25-39 ✅ COMPLETE

All grid.c functions that can reasonably be migrated are now in Rust.

**Remaining in C (by design):**
- `grid_alloc/grid_free` - Tightly coupled to C memory management
- `win_grid_alloc` - Window system integration
- `grid_draw_border` - VirtText iteration, decoration system FFI complexity

---

## Completed Phases Summary

| Phase | Name | Key Functions |
|-------|------|---------------|
| 65 | Type Validators | `rs_tv_check_num`, `rs_tv_check_str`, `rs_tv_check_str_or_nr` |
| 64 | Type Checkers | 22 `rs_tv_check_for_*` argument validation functions |
| 63 | Blob Set | `rs_tv_blob_set` |
| 62 | Blob Equal | `rs_tv_blob_equal` |
| 61 | List Reverse | `rs_tv_list_reverse` |
| 60 | List Find | `rs_tv_list_find`, `rs_tv_list_find_ni` |
| 59 | Typval List Ops | `rs_tv_list_len`, `rs_tv_list_first`, `rs_tv_list_last`, etc. |
| 58 | Typval Blob | `rs_tv_blob_len`, `rs_tv_blob_get`, `rs_tv_blob_set` |
| 57 | Typval Dict | `rs_tv_dict_len`, `rs_tv_dict_is_watched` |
| 56 | Typval Basics | `rs_tv_type`, `rs_tv_get_float`, `rs_tv_check_str` |
| 55 | Blob Copy | `rs_tv_blob_copy` |
| 54 | Typval Clear | `rs_tv_clear`, `rs_tv_item_lock` |
| 53 | Typval Infrastructure | Opaque handles for list_T, dict_T, blob_T |
| 52 | Migration Resume | Re-established workflow after dead code cleanup |
| 51 | Dead Code Cleanup 9 | Removed 159 lines from macros_defs.h, tui/*, api/private/*, event/proc.h |
| 50 | Dead Code Cleanup 8 | Removed 212 lines from ascii_defs.h, os/time.c, os/env.c, mark.h, mark_defs.h, queue_defs.h |
| 49 | Dead Code Cleanup 7 | Removed 187 lines from eval, tui, ops, menu, ex_docmd, cmdexpand/ex_getln |
| 48 | Dead Code Cleanup 6 | Removed 298 lines from autocmd, cmdhist, cursor_shape, spell, indent, ex_eval |
| 47 | Dead Code Cleanup 5 | Removed 383 lines from 16 misc files (arabic, mbyte, help, register, os/*, etc) |
| 46 | Dead Code Cleanup 4 | Removed 408 lines from math.c, memory.c, hashtab.c, version.c, arabic.c, keycodes.c, mouse.c, option.c |
| 45 | Dead Code Cleanup 3 | Removed 743 lines from os/fs.c, path.c, strings.c, profile.c |
| 44 | Dead Code Cleanup 2 | Removed 1888 lines from mbyte.c, charset.c, window.c, helpers.c |
| 43 | Dead Code Cleanup | Removed 1132 lines from grid.c, highlight_group.c, buffer.c, diff.c |
| 42 | Garray Complete | `rs_ga_clear_strings`, `rs_ga_concat_strings` |
| 41 | Hashtab Complete | `rs_hash_clear_all`, removed dead C code |
| 40 | Encoding Cleanup | Removed dead SHA-256 + Base64 fallback code |
| 39 | Grid Handle Assignment | `rs_grid_assign_handle`, `rs_get_bordertext_col` |
| 38 | Grid Line Puts | `rs_grid_line_puts` |
| 37 | Grid Line Start/Mirror | `rs_grid_line_start`, `rs_linebuf_mirror` |
| 36 | Grid Scrolling | `rs_grid_ins_lines`, `rs_grid_del_lines` |
| 34 | Grid Operations | `rs_grid_adjust`, `rs_grid_clear`, `rs_grid_getchar` |
| 33 | Arabic Shaping | `rs_line_do_arabic_shape` |
| 32 | grid_put_linebuf | `rs_grid_put_linebuf` |
| 31 | Grid Line Flush | `rs_grid_line_flush` |
| 30 | Grid Line Content | `rs_grid_line_put_schar`, `rs_grid_line_fill` |
| 29 | Grid Line State | C accessors for grid_line_* state |
| 28 | Line Buffer Infra | C accessors for linebuf_* globals |
| 27 | schar_get | `rs_schar_get`, `rs_schar_get_adv` |
| 26 | Glyph Cache | `rs_schar_from_buf`, `GlyphCache` struct |
| 25 | schar_T Core | `rs_schar_from_str`, `rs_schar_len`, `rs_schar_cells` |
| 24 | Lua Callback FFI | `lua_call_ref()` in nvim-lua crate |

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
| typval       | VimL typval_T type checking and value extraction   |
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
