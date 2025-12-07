# Neovim C-to-Rust Migration Plan

## Executive Summary

Incremental migration of Neovim's ~257,000 lines of C to Rust, prioritizing a working system at every step.

**Key Principles:**

1. **Always Working**: Every milestone produces buildable, testable Neovim
2. **Incremental Validation**: Each phase has clear acceptance criteria
3. **FFI-First**: Use `unsafe` Rust interop with C during transition
4. **Test Continuity**: Existing ~460 functional tests must pass throughout

---

## Current Status (Phase 3.17 Complete)

**160+ functions migrated across 32 Rust crates:**

- nvim-math, nvim-charset, nvim-path, nvim-strings, nvim-mbyte
- nvim-memutil, nvim-os, nvim-collections, nvim-encoding
- nvim-utf8proc, nvim-arabic, nvim-grid, nvim-ops, nvim-register
- nvim-spell, nvim-eval, nvim-ex_docmd, nvim-indent, nvim-keycodes
- nvim-profile, nvim-menu, nvim-help, nvim-cmdhist, nvim-fileio
- nvim-version, nvim-window, nvim-buffer, nvim-mark, nvim-ascii

**Build system:**

- Cargo workspace at `src/nvim-rs/`
- CMake integration via USE*RUST*\* flags
- cbindgen generates C headers from Rust
- 310 rs\_\* symbols exported

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

The simple pure function migration is substantially complete. The remaining blockers require infrastructure changes:

### Priority 1: Global State Accessors

Most remaining functions need access to globals like `curbuf`, `curwin`, `firstwin`, `curtab`.

**Approach**: C accessor functions (safe, incremental)

```c
// In nvim/window.c
win_T *nvim_get_curwin(void) { return curwin; }
win_T *nvim_get_firstwin(void) { return firstwin; }
buf_T *nvim_get_curbuf(void) { return curbuf; }
```

This immediately unblocks dozens of window.c and buffer.c functions.

### Priority 2: libuv Wrappers for Event Loop

**Approach**: Keep libuv, create Rust wrappers around libuv handles. Replace later if needed.

### Priority 3: Complex Struct FFI via bindgen

**Approach**: Use bindgen to generate `win_T`, `buf_T`, `frame_T` definitions. This gives direct field access while maintaining correctness.

### Immediate Next Actions

1. Add `nvim_get_curwin()`, `nvim_get_firstwin()`, `nvim_get_curbuf()` accessors
2. Migrate `win_valid()` and `only_one_window()` as proof of concept
3. Evaluate bindgen for struct generation

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

**Phase 3 Simple Function Migration: Substantially Complete**
Most simple pure functions (FUNC_ATTR_PURE, FUNC_ATTR_CONST) have been migrated.
Remaining blockers for further simple function migration:

- Static initializer macros (RGB\_, schar_from_ascii) cannot use function calls
- Global state arrays (breakat_flags, g_chartab, etc.)
- Complex struct field access (typval_T, win_T internals)
  Further progress requires Phase 4+ infrastructure.

**Phase 3.5: Window/frame function exploration (partially unblocked)**
Window validation now works. Remaining items:

- Global state access for `first_tabpage`, `aucmd_win` (add more accessors as needed)
- Iteration macros: `FOR_ALL_FRAMES`, `FOR_ALL_TAB_WINDOWS` (need FOR_ALL_TABS support)
- FrameHandle opaque type (for frame tree traversal)

**Remaining targets:**

- `win_valid_any_tab` - needs FOR_ALL_TABS iteration
- `frame_fixed_height`, `frame_fixed_width` - need FrameHandle + recursive iteration
- `frame_has_win` - needs FrameHandle + linked list traversal
- plines.c display calculations - need window/buffer accessors with options

### Phase 4: Event Loop & Async I/O

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
**Symbols:** `nm target/release/libnvim_rs.a | grep " T rs_" | wc -l` (currently 299)
