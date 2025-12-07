# Neovim C-to-Rust Migration Plan

## Executive Summary

Incremental migration of Neovim's ~257,000 lines of C to Rust, prioritizing a working system at every step.

**Key Principles:**
1. **Always Working**: Every milestone produces buildable, testable Neovim
2. **Incremental Validation**: Each phase has clear acceptance criteria
3. **FFI-First**: Use `unsafe` Rust interop with C during transition
4. **Test Continuity**: Existing ~460 functional tests must pass throughout

---

## Current Status (Phase 2.84 Complete)

**113+ functions migrated across 27 Rust crates:**
- nvim-math, nvim-charset, nvim-path, nvim-strings, nvim-mbyte
- nvim-memutil, nvim-os, nvim-collections, nvim-encoding
- nvim-utf8proc, nvim-arabic, nvim-grid, nvim-ops, nvim-register
- nvim-spell, nvim-eval, nvim-ex_docmd, nvim-indent, nvim-keycodes
- nvim-profile, nvim-menu, nvim-help, nvim-cmdhist, nvim-fileio

**Build system:**
- Cargo workspace at `src/nvim-rs/`
- CMake integration via USE_RUST_* flags
- cbindgen generates C headers from Rust

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

### Phase 3: Complex Struct FFI
**Blocked by:** Need opaque handle patterns for win_T, buf_T, frame_T

Candidates once infrastructure ready:
- window.c frame functions
- buffer.c validation functions
- plines.c display calculations

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
**Symbols:** `nm build/bin/nvim | grep "T rs_" | wc -l` (currently 249)
