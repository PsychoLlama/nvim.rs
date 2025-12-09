# Neovim C-to-Rust Migration Plan

## Executive Summary

Incremental migration of Neovim's ~257,000 lines of C to Rust, prioritizing a working system at every step.

**Key Principles:**

1. **Always Working**: Every milestone produces buildable, testable Neovim
2. **Incremental Validation**: Each phase has clear acceptance criteria
3. **FFI-First**: Use `unsafe` Rust interop with C during transition
4. **Test Continuity**: Existing ~460 functional tests must pass throughout

---

## Current Status (Phase 4.4 - 368 rs_* Functions, 49 Event Loop)

**368 Rust functions exported across 35 Rust crates:**

- nvim-math, nvim-charset, nvim-path, nvim-strings, nvim-mbyte
- nvim-memutil, nvim-os, nvim-collections, nvim-encoding
- nvim-utf8proc, nvim-arabic, nvim-grid, nvim-ops, nvim-register
- nvim-spell, nvim-eval, nvim-ex_docmd, nvim-indent, nvim-keycodes
- nvim-profile, nvim-menu, nvim-help, nvim-cmdhist, nvim-fileio
- nvim-version, nvim-window, nvim-buffer, nvim-mark, nvim-ascii
- nvim-search, nvim-api, **nvim-event** (49 functions)

**Build system:**

- Cargo workspace at `src/nvim-rs/`
- CMake integration via USE_RUST_* flags (all enabled)
- cbindgen generates C headers from Rust
- 368 rs_* symbols exported
- 40+ USE_RUST_* defines active across C files (including USE_RUST_EVENT)

**Recent Progress (Phase 4.4):**
- Wired USE_RUST_EVENT to 10 C files with 13+ locations:
  - CMakeLists.txt: defines USE_RUST_EVENT
  - time.c (2): rs_timewatcher_should_skip
  - proc.h: rs_proc_is_stopped
  - proc.c: rs_multiqueue_empty
  - input.c (4): rs_multiqueue_empty, rs_pending_events, rs_rstream_is_closed
  - state.c: rs_multiqueue_empty
  - getchar.c: rs_multiqueue_empty
  - shell.c: rs_multiqueue_empty
  - loop.c: rs_multiqueue_size
  - executor.c: rs_loop_is_closing
- Added rs_rstream_is_closed function (uses accessor chaining)

**Phase 4.2-4.3 Summary:**
- 49 event loop Rust functions covering all major event types
- Opaque handles: LoopHandle, MultiQueueHandle, TimeWatcherHandle, ProcHandle,
  StreamHandle, RStreamHandle, SignalWatcherHandle, SocketWatcherHandle
- All tests passing: timer (18), job (61), channels (14)

**Earlier Progress (Phase 3.52-3.68):**
- Phase 3.52-3.68: Multiple phases completing simple function migration

**Phase 3 Simple Function Migration: COMPLETE ✅**

All viable simple FUNC_ATTR_PURE/FUNC_ATTR_CONST functions have been migrated. Window/buffer validation
functions (26 rs_* functions in nvim-window, 12 in nvim-buffer) are complete. Global state accessor
pattern proven with buf_hide (uses nvim_get_p_hid() and nvim_get_cmdmod_cmod_flags()).

Remaining unmigrated functions are blocked by:
- Complex struct internals (typval_T, win_T internals beyond accessors)
- Static initializers (RGB_, schar_from_ascii - can't use function calls)
- Side effects (functions that modify state, not pure)
- Global state arrays (breakat_flags, g_chartab, shape_table)

---

## Completed Phases

### Phase 0: Infrastructure ✅

- [x] Cargo.toml workspace configuration
- [x] CMake/Cargo integration (USE*RUST*\* options)
- [x] cbindgen for C header generation
- [x] CI integration via nix flake

### Phase 0.5: Memory Allocation Bridge ✅

- [x] FFI bindings to xmalloc/xfree
- [x] NvimBox<T>, NvimVec<T>, NvimString wrappers

### Phase 1: Pure Utility Functions ✅

- [x] Math utilities (math.c)
- [x] Encoding (base64.c, sha256.c)
- [x] String utilities (strings.c partial)
- [x] Path utilities (path.c)
- [x] Character set (charset.c)
- [x] Indent (indent.c)
- [x] Keycodes (keycodes.c)
- [x] Command history (cmdhist.c)
- [x] Profile (profile.c)
- [x] Menu (menu.c)
- [x] Help (help.c)
- [x] Ex commands (ex_docmd.c partial)
- [x] Memory utilities (memory.c)

### Phase 2: OS & Data Structures ✅

- [x] Growing array (garray.c) → nvim-collections
- [x] Hash table (hashtab.c) → nvim-collections
- [x] 43 OS filesystem functions → nvim-os
- [x] UTF-8/multibyte (mbyte.c partial) → nvim-mbyte
- [x] utf8proc bindings → nvim-utf8proc
- [x] Arabic text → nvim-arabic
- [x] Grid/screen chars → nvim-grid
- [x] Operators → nvim-ops
- [x] Registers → nvim-register
- [x] Spell checking → nvim-spell
- [x] Eval helpers → nvim-eval

---

## Next Steps: Unblocking Further Migration

### Priority 1: Global State Accessors ✅ COMPLETE

All core global accessors implemented:
- Window: `nvim_get_curwin()`, `nvim_get_firstwin()`, `nvim_get_lastwin()`, `nvim_win_get_next()`
- Buffer: `nvim_get_curbuf()`, `nvim_get_lastbuf()`, `nvim_buf_get_prev()`
- Tabpage: `nvim_get_curtab()`, `nvim_get_first_tabpage()`, `nvim_tabpage_get_next()`, `nvim_tabpage_get_firstwin()`

Validation functions migrated: `win_valid`, `win_valid_any_tab`, `valid_tabpage`, `buf_valid`, `one_window`, `last_window`

### Priority 2: libuv Wrappers for Event Loop

**Status**: Not started (Phase 4+ work)
**Approach**: Keep libuv, create Rust wrappers around libuv handles. Replace later if needed.

### Priority 3: Complex Struct FFI via bindgen

**Status**: Evaluated - continuing with accessor pattern
**Decision**: The opaque handle + accessor pattern is working well and is more incremental than bindgen. We can add accessors as needed for each new function, avoiding the complexity of bindgen.

### Phase 3.21: FrameHandle + Frame Tree Functions ✅

- [x] FrameHandle opaque type for frame_T* pointers
- [x] FR_LEAF, FR_ROW, FR_COL constants
- [x] C accessors: nvim_frame_get_layout, nvim_frame_get_win, nvim_frame_get_child, nvim_frame_get_next
- [x] C accessors: nvim_win_get_wfh, nvim_win_get_wfw (window fixed height/width options)
- [x] rs_frame_has_win: recursive check if window is in frame tree
- [x] rs_frame_fixed_height: recursive check if any window has winfixheight
- [x] rs_frame_fixed_width: recursive check if any window has winfixwidth

### Phase 3.22: Window/Tabpage Counting ✅

- [x] rs_win_count: count windows in current tabpage
- [x] rs_tabpage_index: get 1-based index of tabpage

### Phase 3.23: valid_tabpage_win ✅

- [x] rs_valid_tabpage_win: check if tabpage has valid window
- [x] Uses win_valid_any_tab_impl for window validation

### Phase 3.24: Wire up valid_tabpage_win ✅

- [x] Add USE_RUST_WINDOW block to valid_tabpage_win in window.c

### Phase 3.25: Wire up frame tree functions ✅

- [x] rs_frame_has_win wired to frame_has_win
- [x] rs_frame_fixed_height wired to frame_fixed_height
- [x] rs_frame_fixed_width wired to frame_fixed_width
- [x] Frame tree recursive traversal now uses Rust implementation

### Phase 3.26: is_bottom_win ✅

- [x] Added nvim_frame_get_parent accessor for fr_parent field
- [x] Added nvim_win_get_frame accessor for w_frame field
- [x] Implemented is_bottom_win_impl in Rust (frame tree traversal)
- [x] Used to determine if window is at bottom of column

### Phase 3.27: frame_check_height/width ✅

- [x] Added nvim_frame_get_height accessor for fr_height field
- [x] Added nvim_frame_get_width accessor for fr_width field
- [x] Implemented frame_check_height_impl (verify FR_ROW children heights)
- [x] Implemented frame_check_width_impl (verify FR_COL children widths)
- [x] Used for window layout consistency validation

### Phase 3.28: win_find_by_handle ✅

- [x] Added nvim_win_get_handle accessor for window handle field
- [x] Implemented win_find_by_handle_impl (iterate curtab windows)
- [x] Returns window by handle in current tabpage

### Phase 3.29: win_find_tabpage ✅

- [x] Implemented win_find_tabpage_impl (FOR_ALL_TAB_WINDOWS pattern)
- [x] Iterates all tabpages/windows to find which tabpage contains a window
- [x] No new accessors needed (uses existing iteration helpers)

### Remaining Blockers

Functions that still can't be migrated:
- Static initializer macros (`RGB_`, `schar_from_ascii`) - can't use function calls
- Global state arrays (`breakat_flags`, `g_chartab`) - need bindgen or manual FFI
- Complex struct internals (`typval_T`) - need more accessors

### Immediate Next Actions

1. Continue migrating window/buffer functions using accessor pattern
2. Add more accessors as needed for new functions
3. Consider libuv wrappers when event loop work begins (Phase 4+)

---

## Remaining Phases (Future Work)

### Phase 3: Complex Struct FFI (In Progress)

**Phase 3.1: Opaque Window Handle Infrastructure ✅**

- [x] WinHandle opaque type for win_T\* pointers
- [x] C accessor function pattern (nvim_win_get_locked)
- [x] rs_win_locked() proof of concept
- [x] USE_RUST_WINDOW conditional compilation

**Phase 3.2: Opaque Buffer Handle Infrastructure ✅**

- [x] BufHandle opaque type for buf_T\* pointers
- [x] C accessor functions (nvim_buf_get_buftype, nvim_buf_get_help)
- [x] Buffer type checks: bt_prompt, bt_normal, bt_quickfix, bt_terminal, bt_nofile, bt_help
- [x] USE_RUST_BUFFER conditional compilation

**Phase 3.3: Additional Buffer Type Checks ✅**

- [x] nvim_buf_get_terminal accessor for buf->terminal field
- [x] bt_nofilename: checks nofile, acwrite, terminal, or prompt buffers
- [x] bt_dontwrite: checks nowrite, nofile, terminal, or prompt buffers
- [x] Updated cbindgen exports

**Phase 3.4: Simple function exploration complete**
Most simple pure functions (`FUNC_ATTR_PURE`, `FUNC_ATTR_CONST`) are now migrated.

**Phase 3.6: Additional window accessors ✅**

- [x] nvim_win_get_floating accessor for w_floating field
- [x] nvim_win_get_pvw accessor for w_p_pvw (preview window) field
- [x] rs_win_floating() and rs_win_pvw() Rust implementations

**Phase 3.7: Additional buffer type checks ✅**

- [x] bt_nofileread: checks nofile, quickfix, terminal, or prompt buffers
- [x] rs_bt_nofileread() Rust implementation using existing accessors

**Phase 3.8: File format accessor ✅**

- [x] nvim_buf_get_fileformat accessor for b_p_ff[0] field
- [x] nvim_buf_get_bin accessor for b_p_bin field
- [x] rs_get_fileformat() Rust implementation
- [x] EOL_UNIX, EOL_DOS, EOL_MAC constants in buffer crate

**Phase 3.9: Simple function exploration complete ✅**
Comprehensive search confirms Phase 3.4 finding: most simple pure functions are migrated.
Remaining FUNC_ATTR_PURE/FUNC_ATTR_CONST functions require infrastructure not yet in place:

- Global state access (ctrl_x_mode, user_digraphs, event_names, etc.)
- Complex struct access (typval_T, fuzzyItem_T, fuzmatch_str_T)
- Locale-dependent functions (TOLOWER_LOC)
- Functions already migrated: num*divide, num_modulus, ends_excmd, is_loclist_cmd,
  striequal, vim_strnicmp_asc, vim_strchr, tabstop_padding, indent_size_no_ts,
  indent_size_ts, utf_printable, utf_class_tab, utf_eat_space, utf_allow_break*\*

**Phase 3.10: Register inline functions ✅**

- [x] rs_is_literal_register(): checks if register is \*, +, or alphanumeric
- [x] rs_op_reg_index(): converts register name to y_regs array index
- [x] rs_is_append_register(): checks if register name is uppercase (append mode)
- [x] rs_get_register_name(): converts register index back to character name
- [x] USE_RUST_REGISTER conditional compilation in register.h
- [x] DELETION_REGISTER, STAR_REGISTER, PLUS_REGISTER constants

**Phase 3.11: Math and mark inline functions ✅**

- [x] New nvim-mark crate with pos_T FFI struct
- [x] rs_mark_global_index(): convert global mark name (A-Z, 0-9) to index
- [x] rs_mark_local_index(): convert local mark name (a-z, ", ^, .) to index
- [x] rs_lt(): compare positions (a < b)
- [x] rs_equalpos(): check position equality
- [x] rs_ltoreq(): compare positions (a <= b)
- [x] USE_RUST_MARK conditional compilation in mark.h and mark_defs.h
- [x] USE_RUST_MATH conditional compilation in math.h for is_power_of_two

**Phase 3.12: ASCII character classification functions ✅**

- [x] New nvim-ascii crate for pure ASCII character classification
- [x] rs_ascii_iswhite(): checks space or tab
- [x] rs_ascii_iswhite_or_nul(): space, tab, or NUL
- [x] rs_ascii_iswhite_nl_or_nul(): space, tab, newline, or NUL
- [x] rs_ascii_isdigit(): decimal digit 0-9
- [x] rs_ascii_isxdigit(): hexadecimal digit 0-9, a-f, A-F
- [x] rs_ascii_isbdigit(): binary digit 0-1
- [x] rs_ascii_isodigit(): octal digit 0-7
- [x] rs_ascii_isspace(): whitespace (\f, \n, \r, \t, \v, space)
- [x] rs_ascii_isident(): identifier char (alphanumeric or underscore)
- [x] rs_ascii_isupper(), rs_ascii_islower(), rs_ascii_isalpha(), rs_ascii_isalnum()
- [x] USE_RUST_ASCII conditional compilation in ascii_defs.h

**Phase 3.13: UTF-8 trailing byte detection ✅**

- [x] rs_utf_is_trail_byte(): check if byte is UTF-8 trailing byte (10xxxxxx)
- [x] USE_RUST_MBYTE conditional compilation in mbyte.h

**Phase 3.14: ASCII case conversion ✅**

- [x] rs_ascii_toupper(): convert lowercase to uppercase (a-z -> A-Z)
- [x] rs_ascii_tolower(): convert uppercase to lowercase (A-Z -> a-z)
- [x] USE_RUST_ASCII conditional compilation for TOUPPER_ASC/TOLOWER_ASC macros in macros_defs.h

**Phase 3.15: ASCII character ordinals and utilities ✅**

- [x] rs_char_ord(): get ordinal index of letter (0-25)
- [x] rs_char_ord_low(): get ordinal of lowercase letter
- [x] rs_char_ord_up(): get ordinal of uppercase letter
- [x] rs_rot13(): ROT13 encoding
- [x] rs_meta(): set meta bit (bit 7)
- [x] rs_ctrl_chr(): convert to control character equivalent
- [x] USE_RUST_ASCII conditional compilation in ascii_defs.h

**Phase 3.16: Position and RGB utilities ✅**

- [x] rs_empty_pos(): check if position is empty (all fields are 0)
- [x] rs_rgb(): pack RGB values into 24-bit integer (available but not wired to C macro due to static initializer use)
- [x] USE_RUST_MARK conditional compilation in macros_defs.h

**Phase 3.17: Global state accessors and window validation ✅**

- [x] C accessor functions for global state:
  - nvim_get_curwin(), nvim_get_firstwin(), nvim_get_lastwin()
  - nvim_get_curbuf(), nvim_get_curtab()
  - nvim_win_get_next() for window list traversal
  - nvim_tabpage_get_firstwin() for tabpage window access
- [x] TabpageHandle opaque type for tabpage_T* pointers
- [x] rs_win_valid(): check if window exists in current tabpage
- [x] rs_tabpage_win_valid(): check if window exists in given tabpage
- [x] rs_one_window(): check if only one window exists
- [x] win_valid() and tabpage_win_valid() wired to Rust implementations

**Phase 3.18: Tabpage iteration and cross-tab window validation ✅**

- [x] nvim_get_first_tabpage(): get first_tabpage global
- [x] nvim_tabpage_get_next(): get tp_next field from tabpage
- [x] rs_win_valid_any_tab(): check if window exists in any tabpage
- [x] win_valid_any_tab() wired to Rust implementation
- [x] FOR_ALL_TABS iteration pattern now available in Rust

**Phase 3.19: Tabpage and window validation functions ✅**

- [x] rs_valid_tabpage(): check if tabpage pointer is valid
- [x] rs_one_tabpage(): check if only one tabpage exists
- [x] rs_one_window_in_tab(): check if window is only non-floating window in tabpage
- [x] rs_last_window(): check if window is last non-floating window
- [x] valid_tabpage(), one_window(), last_window() wired to Rust implementations

**Phase 3.20: Buffer validation ✅**

- [x] nvim_get_lastbuf(): get lastbuf global
- [x] nvim_buf_get_prev(): get b_prev field from buffer
- [x] rs_buf_valid(): check if buffer pointer is valid
- [x] buf_valid() wired to Rust implementation

**Phase 3 Simple Function Migration: COMPLETE ✅**

Summary: 357 `rs_*` functions now exported across 34+ Rust crates. All viable
simple pure functions (FUNC_ATTR_PURE, FUNC_ATTR_CONST) have been migrated.

Remaining candidates blocked by:
- Static initializer macros (RGB\_, schar_from_ascii) cannot use function calls
- Global state arrays (breakat_flags, g_chartab, ctrl_x_mode, shape_table)
- Functions accessing globals (cursor_shape.c, insexpand.c, digraph.c)
- Complex struct field access (typval_T, win_T internals)

These remaining targets require Phase 4+ infrastructure (global state accessors,
struct binding generation, or full module replacement).

**Phase 3.5: Window/frame function exploration ✅**
Window validation, tabpage iteration, and frame tree functions are complete.

- [x] FrameHandle opaque type for frame_T* pointers
- [x] Frame accessors: nvim_frame_get_layout, nvim_frame_get_win, nvim_frame_get_child, nvim_frame_get_next, nvim_frame_get_parent, nvim_frame_get_height, nvim_frame_get_width
- [x] Window frame accessors: nvim_win_get_frame, nvim_win_get_wfh, nvim_win_get_wfw
- [x] rs_frame_fixed_height(): check if frame has fixed height (recursive)
- [x] rs_frame_fixed_width(): check if frame has fixed width (recursive)
- [x] rs_frame_has_win(): check if frame tree contains window (recursive)
- [x] rs_is_bottom_win(): check if window is at bottom of column
- [x] rs_frame_check_height(), rs_frame_check_width(): validate frame dimensions
- [x] rs_frame2win(): find window in frame tree
- [x] USE_RUST_WINDOW conditional compilation enabled

**Remaining targets for future work:**

- Global state access for `aucmd_win` (add more accessors as needed)
- plines.c display calculations - need window/buffer accessors with options

### Phase 4: Event Loop & Async I/O

**Status**: IN PROGRESS - Phase 4.1 complete

**Phase 4.1: Event Loop Wrapper Infrastructure ✅** (2025-12-08)

Created nvim-event crate with opaque handle pattern for libuv wrappers:

- [x] nvim-event crate with opaque handle types:
  - LoopHandle for Loop* (event loop context)
  - MultiQueueHandle for MultiQueue* (hierarchical event queue)
  - TimeWatcherHandle for TimeWatcher* (timer wrapper)
- [x] Event struct matching C event/defs.h
- [x] C accessor functions:
  - Loop: nvim_loop_get_events, nvim_loop_get_fast_events, nvim_loop_is_closing
  - MultiQueue: nvim_multiqueue_empty, nvim_multiqueue_size
  - TimeWatcher: nvim_timewatcher_get_data, nvim_timewatcher_get_events, nvim_timewatcher_is_blockable
- [x] Rust wrapper functions:
  - rs_loop_is_closing, rs_loop_get_events, rs_loop_get_fast_events
  - rs_multiqueue_empty, rs_multiqueue_size
  - rs_timewatcher_events_pending, rs_timewatcher_should_skip
- [x] USE_RUST_EVENT flag enabled in CMakeLists.txt
- [x] time_watcher_cb now uses rs_timewatcher_should_skip() for blockable check

**Approach**: Wrap libuv, don't replace it (yet)
- Keep existing libuv usage intact
- Use opaque handle + accessor pattern (proven with WinHandle/BufHandle)
- Migrate helper functions incrementally
- Consider async runtime replacement in later phases

**Architecture**:
```
event/defs.h → uv.h (libuv types)
    ↓
TimeWatcher, SignalWatcher, Stream, RStream, Proc
    ↓
event/loop.c (main event loop)
```

**Next candidates**:
- More TimeWatcher functions (time_watcher_init, time_watcher_start, etc.)
- multiqueue helper functions
- Loop helper functions

Files:
- event/loop.c - libuv wrapper or tokio replacement
- event/rstream.c, wstream.c - stream handling
- event/proc.c - process management

### Phase 5: MessagePack RPC

- msgpack_rpc/packer.c, unpacker.c → rmp crate
- msgpack_rpc/channel.c, server.c

### Phase 6: API Layer

- api/vim.c, buffer.c, window.c, tabpage.c
- API type system (Object, Array, Dict, String)

### Phase 7: Terminal UI

- tui/tui.c - terminal rendering
- tui/input.c - input processing

### Phase 8: Buffer & Text Storage

- memline.c - text storage engine
- buffer.c - buffer lifecycle
- undo.c - undo tree

### Phase 9: Window & Display

- window.c - window layout
- drawscreen.c, drawline.c - rendering
- highlight.c, syntax.c - highlighting

### Phase 10-15: Editor Core

- normal.c, edit.c - modal editing
- ex_docmd.c, ex_cmds.c - commands
- eval.c - VimL evaluation
- lua/executor.c - Lua integration
- regexp.c - regex engine
- spell.c, diff.c, fold.c - features

### Phase 16: Final Cleanup

- Remove FFI wrappers
- Reduce unsafe blocks
- Performance optimization

---

## Testing Strategy

```bash
# Quick validation (every commit)
just build                  # Build succeeds
cargo clippy               # No warnings
just functionaltest        # Core tests

# Full validation (before merge)
just test                  # All tests pass
```

---

## FFI Pattern Reference

```rust
// Example: Pure function export
#[no_mangle]
pub extern "C" fn rs_function_name(arg: c_int) -> c_int {
    // Rust implementation
}

// Example: C global access
extern "C" {
    static g_chartab: [u64; 4];
    static mut cw_table: *const CwEntry;
}

// Example: C callback
extern "C" {
    fn utfc_ptr2len(p: *const u8) -> c_int;
}
```

---

## Risk Areas

| Component     | Risk          | Mitigation                     |
| ------------- | ------------- | ------------------------------ |
| Memory layout | ABI mismatch  | repr(C), extensive FFI testing |
| Event loop    | Deadlocks     | Keep libuv initially           |
| Regex engine  | Compatibility | Port existing, not replace     |
| VimL eval     | Complex state | Last to migrate                |

---

## File Inventory (Largest)

| File       | Lines  | Phase |
| ---------- | ------ | ----- |
| regexp.c   | 16,262 | 14    |
| ex_docmd.c | 8,318  | 11    |
| quickfix.c | 7,776  | 14    |
| window.c   | 7,599  | 9     |
| eval.c     | 6,931  | 12    |
| normal.c   | 6,670  | 10    |

---

## Quick Reference

**Build:** `just build`
**Test:** `just test`
**Rust tests:** `cargo test --workspace`
**Check:** `cargo clippy && cargo fmt --check`
**Symbols:** `nm build/bin/nvim | grep " T rs_" | wc -l` (currently 357)
