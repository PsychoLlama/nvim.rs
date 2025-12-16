# Neovim C-to-Rust Migration Plan

## Current Status

**744 rs\_\* functions migrated**

Run `grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l` to get current count.

### Phase 56: Typval Crate ✅ COMPLETE

Added new `nvim-typval` crate with type checking and value extraction for VimL `typval_T`.
This is foundational for migrating `eval.c` (6.9k lines, 139 typval_T references).

New Rust functions (17):
- Type predicates: `rs_tv_is_number`, `rs_tv_is_string`, `rs_tv_is_float`, etc.
- Value extraction: `rs_tv_get_number_simple`, `rs_tv_get_float_simple`, etc.
- Truthiness: `rs_tv_is_empty`, `rs_tv_is_truthy`

New C accessors in `typval.c` (10):
- `nvim_tv_get_type`, `nvim_tv_get_number`, `nvim_tv_get_bool`, etc.

### Phase 52: Dead Code Cleanup 10 ✅ COMPLETE

Removed 206 lines of dead C fallback code from event/*.c files:
- socket.c: socket_watcher_* accessor macros, loop_get_fast_events
- signal.c: signal_watcher_* accessor macros, loop_get_fast_events
- time.c: timewatcher_* accessor macros, loop_get_fast_events
- proc.c: proc_*, rstream_*, stream_*, loop_* accessor macros (55 lines)
- libuv_proc.c: rstream_*, stream_*, proc_* accessor macros
- loop.c: multiqueue_size macro wrapper removal
- multiqueue.c: multiqueue_empty, multiqueue_size function wrappers
- stream.c: stream_* accessor macros
- rstream.c: stream_*, rstream_* accessor macros
- wstream.c: stream_* accessor macros

### Phase 51: Dead Code Cleanup 9 ✅ COMPLETE

Removed 159 lines of dead C fallback code from scattered modules:
- macros_defs.h: 2 USE_RUST_ASCII conditionals (TOUPPER_ASC, TOLOWER_ASC, ASCII_IS* macros)
- tui/tui.c: 1 USE_RUST_EVENT conditional (rstream_did_eof)
- tui/terminfo.c: 1 USE_RUST_TUI conditional (terminfo_is_term_family, terminfo_is_bsd_console)
- tui/input.c: 1 USE_RUST_TUI conditional (handle_termkey_modifiers, handle_more_modifiers)
- api/private/defs.h: 1 USE_RUST_API conditional (is_internal_call)
- api/private/validate.h: 1 USE_RUST_API conditional (extern declarations)
- api/private/validate.c: 2 USE_RUST_API conditionals (api_err_invalid, api_err_exp)
- event/proc.h: 2 USE_RUST_EVENT conditionals (proc_get_exepath, proc_is_stopped)

### Phase 50: Dead Code Cleanup 8 ✅ COMPLETE

Removed 212 lines of dead C fallback code from header files and small modules:
- ascii_defs.h: 12 USE_RUST_ASCII conditionals (macros + 9 inline functions)
- os/time.c: 4 USE_RUST_OS conditionals (os_hrtime, os_sleep, os_time)
- os/env.c: 4 USE_RUST_OS conditionals (os_get_pid, os_get_hostname)
- mark.h: 3 USE_RUST_MARK conditionals (mark_global_index, mark_local_index)
- mark_defs.h: 5 USE_RUST_MARK conditionals (lt, equalpos, ltoreq, clearpos, EMPTY_POS)
- queue_defs.h: 7 USE_RUST_QUEUE conditionals (QUEUE_* inline functions)

### Phase 49: Dead Code Cleanup 7 ✅ COMPLETE

Removed 187 lines of dead C fallback code from files with 3-6 USE_RUST conditionals:
- eval.c/math.h: num_divide, num_modulus, is_power_of_two
- tui.c: term detection types, truecolor check, terminfo patching
- ops.c: get_op_type, op_on_lines, op_is_change, get_op_char, get_extra_op_char
- menu.c: menu_is_winbar, menu_is_popup, menu_is_toolbar, menu_is_menubar, menu_is_separator
- ex_docmd.c: ends_excmd, find_nextcmd, check_nextcmd, is_loclist_cmd, get_pressedreturn
- cmdexpand.c/ex_getln.c: cmdline_fuzzy_complete, cmdline_overstrike, cmdline_at_end, is_in_cmdwin

### Phase 48: Dead Code Cleanup 6 ✅ COMPLETE

Removed 298 lines of dead C fallback code from files with 4-count USE_RUST conditionals:
- autocmd.c: aucmd_pattern_length, aucmd_next_pattern, is_autocmd_blocked
- cmdhist.c/shada.c: hist_char2type, hist_type2char
- cursor_shape.c: cursor_is_block_during_visual, cursor_mode_uses_syn_id, cursor_get_mode_idx
- spell.c: spell_valid_case, byte_in_str, valid_spelllang
- indent.c: tabstop_padding, indent_size_no_ts, indent_size_ts
- ex_eval.c: aborting, should_abort, aborted_in_try

### Phase 47: Dead Code Cleanup 5 ✅ COMPLETE

Removed 383 lines of dead C fallback code from files with 1-3 USE_RUST conditionals:
- arabic.c: GRID wrapper
- mbyte.c/h: MBYTE wrappers
- help.c: help_heuristic function
- option.c: get_fileformat (BUFFER)
- register.c/h: valid_yank_reg and inline functions
- os/mem.c, os/input.c, os/proc.c: OS_* functions
- msgpack_rpc/unpacker.c: unpack function
- context.c: ctx_size, ctx_free
- fileio.c: is_dev_fd_file, time_differs
- lua/executor.c: nlua_is_deferred_safe and wrappers
- search.c: last_csearch* functions
- insexpand.c: ctrl_x_mode_*, compl_status_* functions

### Phase 46: Dead Code Cleanup 4 ✅ COMPLETE

Removed 408 lines of dead C fallback code from eight migrated modules:
- math.c: 7 functions cleaned (xfpclassify, xisinf, xisnan, xctz, xpopcount, vim_append_digit_int, trim_to_int)
- memory.c: 8 functions + 1 macro cleaned
- hashtab.c: 2 functions cleaned (hash_hash, hash_hash_len)
- version.c: 3 functions cleaned (has_nvim_version, min_vim_version, highest_patch)
- arabic.c: 2 functions cleaned (arabic_maycombine, arabic_combine)
- keycodes.c: 2 functions cleaned (name_to_mod_mask, handle_x_keys)
- mouse.c: 1 function cleaned (is_mouse_key)
- option.c: 7 functions cleaned (find_tty_option_end, valid_name, is_tty_option, default_fileformat, skip_to_option_part, csh_like_shell, fish_like_shell)

### Phase 45: Dead Code Cleanup 3 ✅ COMPLETE

Removed 743 lines of dead C fallback code from four migrated modules:
- os/fs.c: -430 lines (41 #else blocks removed, including complex nested platform-specific code)
- path.c: -145 lines (14 #else blocks removed)
- strings.c: -108 lines (12 #else blocks removed)
- profile.c: -60 lines (11 #else blocks removed)

### Phase 44: Dead Code Cleanup 2 ✅ COMPLETE

Removed 1888 lines of dead C fallback code from files with complete Rust implementations:
- mbyte.c: -1132 lines (47 #else blocks removed)
- charset.c: -308 lines (35 #else blocks removed)
- window.c: -240 lines (22 #else blocks removed)
- api/private/helpers.c: -208 lines (20 #else blocks removed)

### Phase 43: Dead Code Cleanup ✅ COMPLETE

Removed 1132 lines of dead C fallback code from files with complete Rust implementations:
- grid.c: -738 lines (36 #else blocks removed)
- highlight_group.c: -269 lines (14 #else blocks removed)
- buffer.c: -88 lines (extern wrapper cleanup)
- diff.c: -43 lines (extern wrapper cleanup)

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
