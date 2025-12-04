# Neovim C-to-Rust Migration Plan

## Executive Summary

This document outlines an incremental strategy to migrate Neovim's ~257,000 lines of C code to Rust. The migration prioritizes maintaining a working, testable system at every step by leveraging FFI boundaries and Rust's `unsafe` capabilities where necessary.

**Key Principles:**

1. **Always Working**: Every milestone produces a buildable, testable Neovim
2. **Incremental Validation**: Each phase has clear acceptance criteria
3. **FFI-First**: Use `unsafe` Rust interop with C during transition
4. **Test Continuity**: Existing ~460 functional tests must pass throughout

---

## Current Architecture Overview

```
┌───────────────────────────────────────────────────────────────┐
│                         main.c (entry)                        │
├────────────┬───────────────┬───────────────┬──────────────────┤
│   os/      │    event/     │   msgpack_rpc/│      tui/        │
│ (40 files) │  (11 files)   │  (10 files)   │   (7 files)      │
│  OS layer  │  Event loop   │     RPC       │  Terminal UI     │
├────────────┴───────────────┴───────────────┴──────────────────┤
│                     Core Editor Engine                        │
│  buffer.c, window.c, memline.c, normal.c, edit.c, eval.c, ... │
├───────────────────────────────────────────────────────────────┤
│   api/           │     lua/          │      eval/             │
│  (30+ files)     │   (18 files)      │    (29 files)          │
│  RPC API layer   │  Lua integration  │  VimL evaluation       │
└───────────────────────────────────────────────────────────────┘
```

**Dependencies:** LuaJIT, libuv, tree-sitter, UTF-8proc, libintl, unibilium

---

## Migration Phases

### Phase 0: Infrastructure Setup (Foundation)

**Goal:** Establish Rust build infrastructure alongside existing CMake

#### 0.1 Add Rust Build System

- [x] Add `Cargo.toml` at repository root with workspace configuration
- [x] Create `src/nvim-rs/` directory for Rust crates
- [x] Integrate Cargo into CMake build via custom cmake rules (USE_RUST_MATH option)
- [ ] Ensure `make` builds both C and Rust components automatically
- [x] Set up `cbindgen` for generating C headers from Rust
- [ ] Set up `bindgen` for generating Rust bindings from C headers

#### 0.2 CI Integration

- [x] Add Rust toolchain to GitHub Actions workflows (via nix flake)
- [x] Add `cargo clippy` and `cargo fmt` checks (via justfile)
- [ ] Ensure all existing tests still pass

**Validation:**

```bash
make                    # Builds nvim with Rust crate (no-op initially)
make test               # All ~460 functional tests pass
cargo test              # Rust unit tests pass (empty initially)
```

**Deliverable:** Hybrid C/Rust build system working

---

### Phase 0.5: Memory Allocation Bridge (Critical Foundation)

**Goal:** Establish FFI bridge for memory allocation before migrating any code that allocates

Most nvim code uses `xmalloc`/`xfree` from `nvim/memory.h`. Rust code must interoperate with this allocation system.

#### 0.5.1 Create Memory FFI Module

- [x] Create `nvim-rs/memory/` crate
- [x] Define FFI bindings to nvim's allocator:

```rust
// src/nvim-rs/memory/src/lib.rs
use std::ffi::c_void;
use std::os::raw::c_size_t;

extern "C" {
    pub fn xmalloc(size: c_size_t) -> *mut c_void;
    pub fn xcalloc(count: c_size_t, size: c_size_t) -> *mut c_void;
    pub fn xrealloc(ptr: *mut c_void, size: c_size_t) -> *mut c_void;
    pub fn xmallocz(size: c_size_t) -> *mut c_void;
    pub fn xfree(ptr: *mut c_void);
}
```

- [x] Create safe wrapper types (`NvimBox<T>`, `NvimVec<T>`) that use nvim's allocator
- [x] Implement `Drop` for automatic cleanup

#### 0.5.2 String Interop Types

- [x] Create `NvimString` type for C-compatible strings allocated with `xmalloc`
- [x] Implement conversions: `&str` ↔ `NvimString` ↔ `*const c_char`

**Validation:**

```bash
cargo test -p nvim-memory        # Memory wrapper tests
make test                        # Nvim still works
```

**Deliverable:** Safe Rust wrappers for nvim's memory allocation

---

### Phase 1: Pure Utility Functions (Low Risk)

**Goal:** Migrate isolated utility functions with no state dependencies

> **Note:** Many "utility" functions in nvim actually use `xmalloc`/`xfree` and are
> NOT pure. Phase 0.5 must be completed first. Start with truly pure functions.

#### 1.1 Math Utilities (`src/nvim/math.c` → `nvim-rs/math`) ✓ TRULY PURE

`math.c` has minimal dependencies (only `vim_defs.h` for macros). Start here.

- [x] `xfpclassify` - Float classification
- [x] `xisinf` - Infinity check
- [x] `xisnan` - NaN check
- [x] `xctz` - Count trailing zeroes
- [x] `xpopcount` - Population count (set bits)
- [x] `vim_append_digit_int` - Safe digit append with overflow check
- [x] `trim_to_int` - Clamp int64 to int range
- [x] Create C-compatible wrapper functions using `#[no_mangle]`
- [x] Replace C implementations with calls to Rust (USE_RUST_MATH=ON)

#### 1.2 Encoding Utilities (REQUIRES Phase 0.5)

These use `xmalloc` - migrate AFTER memory bridge is ready:

- [x] `src/nvim/base64.c` → `nvim-rs/encoding/base64` (uses xmalloc)
- [x] `src/nvim/sha256.c` → `nvim-rs/encoding/sha256` (uses nvim/memory.h)

#### 1.3 String Utilities (REQUIRES Phase 0.5 + mbyte)

`strings.c` has heavy dependencies (~20 nvim headers). Defer until:
- Phase 0.5 (memory) complete
- Phase 3.3 (mbyte) complete for `utfc_ptr2len` etc.

- [ ] `vim_strsave` - String duplication (uses xmallocz) - defer until C integration
- [ ] `vim_strnsave` - Bounded string copy (uses xmallocz) - defer until C integration
- [x] `vim_strchr` - Character search (ASCII only, multibyte requires mbyte)
- [x] `concat_str` - String concatenation
- [x] `vim_stricmp` / `vim_strnicmp` - Case-insensitive comparison
- [x] `striequal` - Case-insensitive equality check
- [x] `has_non_ascii` - Check for non-ASCII characters - swapped to Rust
- [x] `sort_strings` - Sort array of strings - swapped to Rust
- [x] `vim_strnicmp_asc` - ASCII-only case-insensitive compare - swapped to Rust

#### 1.4 Path Utilities (PARTIAL - some pure, some not)

- [x] `vim_ispathsep` - Path separator check (pure) - swapped to Rust
- [x] `vim_ispathsep_nocolon` - Path separator check excluding colon (pure) - swapped to Rust
- [x] `vim_ispathlistsep` - Path list separator check (pure) - swapped to Rust
- [x] `path_tail` - Get filename from path (pure) - swapped to Rust
- [x] `path_head_length` - Directory prefix length (pure) - swapped to Rust
- [x] `path_is_absolute` - Check if path is absolute (pure) - swapped to Rust
- [x] `path_is_url` - Check for URL scheme separator (pure) - swapped to Rust
- [x] `path_has_drive_letter` - Check for Windows drive letter (pure) - swapped to Rust
- [x] `path_with_url` - Check if path starts with URL scheme (pure) - swapped to Rust
- [x] `is_path_head` - Check if path starts with path head (pure) - swapped to Rust
- [x] `get_past_head` - Get pointer past path head (pure) - swapped to Rust
- [ ] Path normalization functions (may use allocation)

**Validation:**

```bash
make test                              # All tests pass
cargo test -p nvim-math                # Rust unit tests for math module
```

**FFI Pattern:**

```rust
// src/nvim-rs/math/src/lib.rs
use std::os::raw::{c_int, c_uint};

/// Count trailing zeroes in a 64-bit value
#[no_mangle]
pub extern "C" fn xctz(x: u64) -> c_int {
    if x == 0 {
        64
    } else {
        x.trailing_zeros() as c_int
    }
}

/// Count set bits (population count)
#[no_mangle]
pub extern "C" fn xpopcount(x: u64) -> c_uint {
    x.count_ones()
}
```

---

### Phase 2: OS Abstraction Layer (Clear Boundary)

**Goal:** Migrate `src/nvim/os/` - the platform abstraction layer

This layer has well-defined interfaces and minimal coupling to editor internals.

#### 2.1 Environment & System Info

- [x] `src/nvim/os/env.c` → `nvim-rs/os/env`
  - `os_getenv`, `os_setenv`, `os_unsetenv`
  - `os_get_hostname`, `os_get_user_name`
  - `os_get_pid`
- [x] `src/nvim/os/time.c` → `nvim-rs/os/time`
  - `os_hrtime`, `os_utime`, `os_localtime_r`

#### 2.2 Filesystem Operations

- [x] `src/nvim/os/fs.c` → `nvim-rs/os/fs`
  - `os_file_exists`, `os_isdir`, `os_can_exe`
  - `os_getperm`, `os_setperm`, `os_file_is_readable`
  - `os_rename`, `os_copy`, `os_remove`
  - `os_mkdir`, `os_rmdir`, `os_scandir`
- [ ] `src/nvim/os/fileio.c` → `nvim-rs/os/fileio`
  - File read/write with proper error handling

#### 2.3 Dynamic Loading

- [ ] `src/nvim/os/dl.c` → `nvim-rs/os/dl`
  - `os_dlopen`, `os_dlsym`, `os_dlclose`
  - Use `libloading` crate

#### 2.4 Memory Allocation

- [ ] `src/nvim/os/mem.c` → custom allocator integration
  - Careful: global allocator affects everything
  - May need `#[global_allocator]` or keep C allocator initially

**Validation:**

```bash
TEST_FILE=test/unit/os/fs_spec.lua make unittest
TEST_FILE=test/unit/os/env_spec.lua make unittest
TEST_FILE=test/functional/core/fileio_spec.lua make functionaltest
```

---

### Phase 3: Data Structures (Foundational)

**Goal:** Migrate core data structures that underpin the editor

#### 3.1 Hash Table (`src/nvim/hashtab.c`)

- [x] Implement `HashMap`-compatible structure in Rust
- [x] Expose C-compatible API via FFI
- [ ] Used throughout codebase - careful migration

#### 3.2 Growing Array (`src/nvim/garray.c`)

- [x] Map to `Vec<T>` with C-compatible wrapper
- [x] Provide `ga_init`, `ga_grow`, `ga_append`, `ga_clear`

#### 3.3 Multibyte/UTF-8 (`src/nvim/mbyte.c`)

- [x] UTF-8 encoding/decoding (utf_ptr2char, utf_char2bytes, utf_ptr2len) - swapped to Rust
- [x] UTF-8 byte length tables (utf8len_tab, utf8len_tab_zero) - implemented in Rust
- [x] `utf_char2len` - Get UTF-8 byte length for codepoint - swapped to Rust
- [x] `utf_byte2len` - Get UTF-8 length from first byte - swapped to Rust
- [x] `utf_ptr2len_len` - Get UTF-8 length with size limit - swapped to Rust
- [x] `utf_valid_string` - Validate UTF-8 string - swapped to Rust
- [x] `utf_eat_space` - Whether space is not allowed before/after character - swapped to Rust
- [x] `utf_allow_break_before` - Whether line break is allowed before character - swapped to Rust
- [x] `utf_allow_break_after` - Whether line break is allowed after character - swapped to Rust
- [ ] Character width calculation (utf_char2cells - requires display tables)
- [ ] Composing character handling (utfc_ptr2len - requires grapheme state)
- [ ] Encoding conversion (iconv integration)

#### 3.4 Mark Tree (`src/nvim/marktree.c`)

- [ ] Interval tree for extmarks
- [ ] Complex but self-contained data structure
- [ ] Critical for LSP and highlighting performance

**Validation:**

```bash
TEST_FILE=test/unit/marktree_spec.lua make unittest
# Run full test suite - data structures are foundational
make test
```

---

### Phase 4: Event Loop & Async I/O (Core Infrastructure)

**Goal:** Migrate the libuv-based event system

This is a critical phase - the event loop touches everything.

#### 4.1 Event Loop Wrapper

- [ ] `src/nvim/event/loop.c` → `nvim-rs/event/loop`
- [ ] Options:
  - A) Wrap libuv with Rust (keep C compatibility)
  - B) Replace with `tokio`/`async-std` (larger change)
- [ ] Recommend Option A initially for compatibility

#### 4.2 Stream Handling

- [ ] `src/nvim/event/rstream.c` → Rust read streams
- [ ] `src/nvim/event/wstream.c` → Rust write streams
- [ ] `src/nvim/event/stream.c` → Base stream utilities

#### 4.3 Process Management

- [ ] `src/nvim/event/proc.c` → `nvim-rs/event/proc`
- [ ] `src/nvim/event/libuv_proc.c` → libuv process wrapper
- [ ] Job control for `:terminal`, `:!cmd`, etc.

#### 4.4 Async Primitives

- [ ] `src/nvim/event/multiqueue.c` → Event queue
- [ ] `src/nvim/event/signal.c` → Signal handling
- [ ] `src/nvim/event/time.c` → Timer management
- [ ] `src/nvim/event/socket.c` → Socket handling

**Validation:**

```bash
TEST_FILE=test/functional/core/job_spec.lua make functionaltest
TEST_FILE=test/functional/terminal/ make functionaltest
# Terminal and async tests are critical
```

---

### Phase 5: MessagePack RPC Layer (API Boundary)

**Goal:** Migrate the RPC protocol implementation

Clean API boundary - external UIs communicate through this layer.

#### 5.1 MessagePack Encoding/Decoding

- [ ] `src/nvim/msgpack_rpc/packer.c` → Use `rmp` crate
- [ ] `src/nvim/msgpack_rpc/unpacker.c` → Use `rmp` crate

#### 5.2 Channel Management

- [ ] `src/nvim/msgpack_rpc/channel.c` → `nvim-rs/rpc/channel`
- [ ] `src/nvim/msgpack_rpc/server.c` → `nvim-rs/rpc/server`

#### 5.3 API Dispatch

- [ ] Code generation for API dispatch (modify `gen_api_dispatch.lua`)
- [ ] Generate Rust dispatch code alongside C

**Validation:**

```bash
TEST_FILE=test/functional/api/ make functionaltest
# Test with external UIs (nvim-qt, neovide)
```

---

### Phase 6: API Layer (External Interface)

**Goal:** Migrate `src/nvim/api/` - the public API surface

#### 6.1 API Type System

- [ ] Port `src/nvim/api/private/defs.h` types to Rust
- [ ] `Object`, `Array`, `Dict`, `String`, `Integer`, etc.
- [ ] Implement `From`/`Into` traits for C interop

#### 6.2 Core API Functions

- [ ] `src/nvim/api/vim.c` → Core editor API
- [ ] `src/nvim/api/buffer.c` → Buffer API
- [ ] `src/nvim/api/window.c` → Window API
- [ ] `src/nvim/api/tabpage.c` → Tab API
- [ ] `src/nvim/api/options.c` → Option API

#### 6.3 Extended API

- [ ] `src/nvim/api/extmark.c` → Extmark API
- [ ] `src/nvim/api/ui.c` → UI events API
- [ ] `src/nvim/api/command.c` → Command API
- [ ] `src/nvim/api/autocmd.c` → Autocmd API

**Validation:**

```bash
# Full API test suite
TEST_FILE=test/functional/api/ make functionaltest
# Lua API bindings
TEST_FILE=test/functional/lua/ make functionaltest
```

---

### Phase 7: Terminal UI (User-Facing)

**Goal:** Migrate `src/nvim/tui/` - terminal rendering

#### 7.1 TUI Core

- [ ] `src/nvim/tui/tui.c` → `nvim-rs/tui`
- [ ] Consider using `crossterm` or direct terminfo
- [ ] `src/nvim/tui/terminfo.c` → terminfo database handling

#### 7.2 Input Processing

- [ ] `src/nvim/tui/input.c` → Terminal input parsing
- [ ] Key sequence decoding

**Validation:**

```bash
TEST_FILE=test/functional/ui/ make functionaltest
# Manual testing: visual inspection of rendering
```

---

### Phase 8: Buffer & Text Storage (Core Editor)

**Goal:** Migrate the text representation layer

This is the heart of the editor - careful migration required.

#### 8.1 Memory Line Storage

- [ ] `src/nvim/memline.c` (4,247 lines) → Text storage engine
- [ ] `src/nvim/memfile.c` → Swap file handling
- [ ] B-tree structure for line storage

#### 8.2 Buffer Management

- [ ] `src/nvim/buffer.c` (4,250 lines) → Buffer lifecycle
- [ ] Buffer list management
- [ ] File loading/saving integration

#### 8.3 Undo System

- [ ] `src/nvim/undo.c` → Undo tree
- [ ] Complex branching undo history

**Validation:**

```bash
TEST_FILE=test/functional/core/fileio_spec.lua make functionaltest
TEST_FILE=test/functional/editor/buffer_spec.lua make functionaltest
TEST_FILE=test/functional/legacy/undo_spec.lua make functionaltest
```

---

### Phase 9: Window & Display (Rendering)

**Goal:** Migrate window management and screen rendering

#### 9.1 Window Management

- [ ] `src/nvim/window.c` (7,599 lines) → Window layout
- [ ] Split handling, window navigation
- [ ] Floating windows (`winfloat.c`)

#### 9.2 Screen Rendering

- [ ] `src/nvim/drawscreen.c` → Full screen redraw
- [ ] `src/nvim/drawline.c` → Line rendering
- [ ] `src/nvim/grid.c` → Grid management

#### 9.3 Highlighting

- [ ] `src/nvim/highlight.c` → Highlight attributes
- [ ] `src/nvim/highlight_group.c` → Highlight groups
- [ ] `src/nvim/syntax.c` (5,673 lines) → Legacy syntax

**Validation:**

```bash
TEST_FILE=test/functional/ui/screen_basic_spec.lua make functionaltest
TEST_FILE=test/functional/ui/float_spec.lua make functionaltest
TEST_FILE=test/functional/ui/highlight_spec.lua make functionaltest
```

---

### Phase 10: Modal Editing (User Interaction)

**Goal:** Migrate input processing and modal behavior

#### 10.1 Normal Mode

- [ ] `src/nvim/normal.c` (6,670 lines) → Normal mode commands
- [ ] Motion commands
- [ ] Operator handling

#### 10.2 Insert Mode

- [ ] `src/nvim/edit.c` (4,358 lines) → Insert mode
- [ ] `src/nvim/insexpand.c` (6,581 lines) → Completion

#### 10.3 Command Line

- [ ] `src/nvim/ex_getln.c` (5,007 lines) → Command line editing
- [ ] `src/nvim/cmdexpand.c` (4,261 lines) → Command completion

**Validation:**

```bash
TEST_FILE=test/functional/editor/mode_insert_spec.lua make functionaltest
TEST_FILE=test/functional/editor/mode_cmdline_spec.lua make functionaltest
# Extensive manual testing for modal behavior
```

---

### Phase 11: Ex Commands (Command Processing)

**Goal:** Migrate the Ex command infrastructure

#### 11.1 Command Dispatcher

- [ ] `src/nvim/ex_docmd.c` (8,318 lines) → Command parsing & dispatch
- [ ] Command range handling
- [ ] Command modifiers

#### 11.2 Command Implementations

- [ ] `src/nvim/ex_cmds.c` (5,080 lines) → Individual commands
- [ ] `src/nvim/ex_cmds2.c` → More commands
- [ ] `src/nvim/usercmd.c` → User-defined commands

**Validation:**

```bash
TEST_FILE=test/functional/ex_cmds/ make functionaltest
TEST_FILE=test/functional/legacy/ make functionaltest
```

---

### Phase 12: VimL Evaluation Engine (Scripting)

**Goal:** Migrate the VimL interpreter

This is one of the largest and most complex subsystems.

#### 12.1 Expression Evaluator

- [ ] `src/nvim/eval.c` (6,931 lines) → Expression evaluation
- [ ] `src/nvim/eval/typval.c` → Type system
- [ ] `src/nvim/eval/vars.c` → Variable handling

#### 12.2 Built-in Functions

- [ ] `src/nvim/eval/funcs.c` → 300+ built-in functions
- [ ] `src/nvim/eval/userfunc.c` → User function handling

#### 12.3 VimL Parser

- [ ] `src/nvim/viml/parser/` → VimL parser
- [ ] Consider using a parser generator

**Validation:**

```bash
TEST_FILE=test/functional/vimscript/ make functionaltest
TEST_FILE=test/functional/eval/ make functionaltest
TEST_FILE=test/old/testdir/ make oldtest
```

---

### Phase 13: Lua Integration (Modern Scripting)

**Goal:** Migrate Lua runtime integration

#### 13.1 Lua Executor

- [ ] `src/nvim/lua/executor.c` → Lua code execution
- [ ] `src/nvim/lua/converter.c` → Type conversion
- [ ] Decide: mlua, rlua, or direct FFI

#### 13.2 Lua APIs

- [ ] `src/nvim/lua/stdlib.c` → Standard library
- [ ] `src/nvim/lua/treesitter.c` → Tree-sitter API
- [ ] `src/nvim/lua/fs.c` → Filesystem API

**Validation:**

```bash
TEST_FILE=test/functional/lua/ make functionaltest
TEST_FILE=test/functional/treesitter/ make functionaltest
```

---

### Phase 14: Search & Navigation (Editor Features)

**Goal:** Migrate search, tags, and navigation

#### 14.1 Regular Expressions

- [ ] `src/nvim/regexp.c` (16,262 lines) → Regex engine
- [ ] Options: Port existing or use `regex` crate
- [ ] Must maintain Vim regex compatibility

#### 14.2 Search

- [ ] `src/nvim/search.c` → Search implementation
- [ ] `src/nvim/quickfix.c` (7,776 lines) → Quickfix list

#### 14.3 Tags

- [ ] `src/nvim/tag.c` → Tag navigation
- [ ] `src/nvim/tagfunc.c` → Tag functions

**Validation:**

```bash
TEST_FILE=test/functional/legacy/search_spec.lua make functionaltest
TEST_FILE=test/functional/legacy/quickfix_spec.lua make functionaltest
```

---

### Phase 15: Auxiliary Features (Completion)

**Goal:** Migrate remaining subsystems

#### 15.1 Spelling

- [ ] `src/nvim/spell.c` → Spell checking
- [ ] `src/nvim/spellfile.c` (5,751 lines) → Spell file handling
- [ ] `src/nvim/spellsuggest.c` → Suggestions

#### 15.2 Diff

- [ ] `src/nvim/diff.c` (4,324 lines) → Diff mode
- [ ] Integration with bundled xdiff

#### 15.3 Folding

- [ ] `src/nvim/fold.c` → Code folding

#### 15.4 Autocommands

- [ ] `src/nvim/autocmd.c` → Autocommand system

**Validation:**

```bash
make test  # Full test suite
make oldtest  # Vim compatibility
```

---

### Phase 16: Final Integration & Cleanup

**Goal:** Remove C code, finalize pure Rust implementation

#### 16.1 Remove FFI Wrappers

- [ ] Replace `extern "C"` functions with pure Rust calls
- [ ] Remove `unsafe` blocks where possible
- [ ] Audit remaining `unsafe` for soundness

#### 16.2 Optimize

- [ ] Profile and optimize hot paths
- [ ] Leverage Rust's zero-cost abstractions
- [ ] Memory usage optimization

#### 16.3 Documentation

- [ ] API documentation with `rustdoc`
- [ ] Architecture documentation
- [ ] Contributing guide for Rust

**Validation:**

```bash
make test           # All tests pass
make benchmark      # Performance acceptable
cargo clippy        # No warnings
cargo audit         # No security issues
```

---

## Testing Strategy

### Continuous Validation

Every commit during migration must pass:

```bash
# Quick validation (run on every commit)
make                    # Build succeeds
cargo test             # Rust unit tests pass
make functionaltest    # Core functional tests

# Full validation (run before merge)
make test              # All tests pass
make oldtest           # Vim compatibility
VALGRIND=1 make test   # Memory safety
```

### Test Categories

| Category         | Count | Purpose                |
| ---------------- | ----- | ---------------------- |
| Functional tests | ~460  | End-to-end behavior    |
| Unit tests       | ~50   | Component isolation    |
| Old tests        | ~100  | Vim compatibility      |
| Benchmarks       | ~20   | Performance regression |

### FFI Testing

For each migrated module, add:

1. **Rust unit tests** - Test pure Rust logic
2. **FFI boundary tests** - Test C↔Rust interop
3. **Integration tests** - Test in full Nvim context

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_implementation() {
        // Pure Rust test
    }

    #[test]
    fn test_ffi_roundtrip() {
        // C-compatible interface test
    }
}
```

### Regression Detection

```bash
# Compare before/after each phase
./build/bin/nvim --version          # Version info
./build/bin/nvim --startuptime /tmp/startup.log  # Startup time
hyperfine './build/bin/nvim +q'     # Benchmark
```

---

## Risk Mitigation

### High-Risk Areas

| Component     | Risk                | Mitigation                          |
| ------------- | ------------------- | ----------------------------------- |
| Memory layout | ABI incompatibility | Extensive FFI testing, `repr(C)`    |
| Event loop    | Deadlocks           | Keep libuv initially, migrate later |
| Regex engine  | Compatibility       | Port existing, extensive test suite |
| VimL eval     | Complex state       | Last to migrate, thorough testing   |

### Rollback Strategy

Each phase should be independently revertible:

1. Keep C code in separate branch until phase validated
2. Feature flags for Rust vs C implementation
3. Automated bisect-friendly commits

---

## Timeline Considerations

This migration is a multi-year effort. Rough ordering by complexity:

1. **Phases 0-3** (Foundation): Can proceed in parallel, low risk
2. **Phases 4-5** (Infrastructure): Sequential, moderate risk
3. **Phases 6-7** (Interface): Can proceed in parallel after Phase 5
4. **Phases 8-11** (Core): Sequential, high complexity
5. **Phases 12-13** (Scripting): High complexity, extensive testing
6. **Phases 14-16** (Completion): Cleanup and optimization

---

## Success Criteria

### Per-Phase Gates

- [ ] All existing tests pass
- [ ] No performance regression >10%
- [ ] Memory usage comparable
- [ ] `cargo clippy` clean
- [ ] Documentation updated

### Final Goals

- [ ] 100% Rust (except LuaJIT FFI)
- [ ] Memory safety without runtime cost
- [ ] Maintainable, idiomatic Rust code
- [ ] All ~460 functional tests passing
- [ ] Vim compatibility preserved
- [ ] External UI compatibility preserved

---

## Appendix: File Inventory

### Largest C Files (Migration Complexity)

| File          | Lines  | Phase |
| ------------- | ------ | ----- |
| `regexp.c`    | 16,262 | 14    |
| `ex_docmd.c`  | 8,318  | 11    |
| `quickfix.c`  | 7,776  | 14    |
| `window.c`    | 7,599  | 9     |
| `eval.c`      | 6,931  | 12    |
| `normal.c`    | 6,670  | 10    |
| `insexpand.c` | 6,581  | 10    |
| `option.c`    | 6,424  | 6     |
| `spellfile.c` | 5,751  | 15    |
| `syntax.c`    | 5,673  | 9     |

### Module Dependencies

```
os/ ──────────────────────────────┐
event/ ───────────────────────────┤
                                  ├──► api/ ──► msgpack_rpc/
data structures (hashtab, etc.) ──┤              │
                                  │              ▼
buffer/memline ───────────────────┼──► tui/ ◄── ui/
                                  │
window/screen ────────────────────┤
                                  │
eval/ ────────────────────────────┼──► lua/
                                  │
normal/edit/ex_cmds ──────────────┘
```

---

## Getting Started

```bash
# Clone and build baseline
git clone https://github.com/neovim/neovim
cd neovim
make CMAKE_BUILD_TYPE=RelWithDebInfo

# Run tests to establish baseline
make test

# Begin Phase 0: Add Rust infrastructure
# (See Phase 0 tasks above)
```

---

_This document should be updated as the migration progresses. Each completed phase should be checked off and any lessons learned documented._
