# Neovim C-to-Rust Migration Plan

## Executive Summary

Incremental migration of Neovim's ~257,000 lines of C to Rust, prioritizing a working system at every step.

**Key Principles:**
1. **Always Working**: Every milestone produces buildable, testable Neovim
2. **Incremental Validation**: Each phase has clear acceptance criteria
3. **FFI-First**: Use `unsafe` Rust interop with C during transition
4. **Test Continuity**: Existing ~460 functional tests must pass throughout

---

## Current Status (Phase 3.3 Complete)

**135+ functions migrated across 30 Rust crates:**
- nvim-math, nvim-charset, nvim-path, nvim-strings, nvim-mbyte
- nvim-memutil, nvim-os, nvim-collections, nvim-encoding
- nvim-utf8proc, nvim-arabic, nvim-grid, nvim-ops, nvim-register
- nvim-spell, nvim-eval, nvim-ex_docmd, nvim-indent, nvim-keycodes
- nvim-profile, nvim-menu, nvim-help, nvim-cmdhist, nvim-fileio
- nvim-version, nvim-window, nvim-buffer

**Build system:**
- Cargo workspace at `src/nvim-rs/`
- CMake integration via USE_RUST_* flags
- cbindgen generates C headers from Rust
- 271 rs_* symbols exported

---

## Completed Phases

### Phase 0: Infrastructure ✅
- [x] Cargo.toml workspace configuration
- [x] CMake/Cargo integration (USE_RUST_* options)
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

## Remaining Phases (Future Work)

### Phase 3: Complex Struct FFI (In Progress)
**Phase 3.1: Opaque Window Handle Infrastructure ✅**
- [x] WinHandle opaque type for win_T* pointers
- [x] C accessor function pattern (nvim_win_get_locked)
- [x] rs_win_locked() proof of concept
- [x] USE_RUST_WINDOW conditional compilation

**Phase 3.2: Opaque Buffer Handle Infrastructure ✅**
- [x] BufHandle opaque type for buf_T* pointers
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

**Phase 3.5: Window/frame function exploration (blocked)**
Window and frame functions require infrastructure not yet in place:
- Global state access: `firstwin`, `curtab`, `first_tabpage`, `aucmd_win`
- Iteration macros: `FOR_ALL_WINDOWS_IN_TAB`, `FOR_ALL_FRAMES`, `FOR_ALL_TAB_WINDOWS`
- Multiple opaque handles: `FrameHandle`, `TabpageHandle` (in addition to `WinHandle`)
- Linked list traversal patterns

**Blocked targets (need Phase 4+ infrastructure):**
- `win_valid`, `win_valid_any_tab`, `only_one_window` - need global state + iteration
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

| Component | Risk | Mitigation |
|-----------|------|------------|
| Memory layout | ABI mismatch | repr(C), extensive FFI testing |
| Event loop | Deadlocks | Keep libuv initially |
| Regex engine | Compatibility | Port existing, not replace |
| VimL eval | Complex state | Last to migrate |

---

## File Inventory (Largest)

| File | Lines | Phase |
|------|-------|-------|
| regexp.c | 16,262 | 14 |
| ex_docmd.c | 8,318 | 11 |
| quickfix.c | 7,776 | 14 |
| window.c | 7,599 | 9 |
| eval.c | 6,931 | 12 |
| normal.c | 6,670 | 10 |

---

## Quick Reference

**Build:** `just build`
**Test:** `just test`
**Rust tests:** `cargo test --workspace`
**Check:** `cargo clippy && cargo fmt --check`
**Symbols:** `nm build/bin/nvim | grep "T rs_" | wc -l` (currently 269)
