## Guidelines

- `rust-migration` is your main branch.
- Commit your work regularly.
- Before committing, ensure these pass:
  - `just rust-fmt-check` - formatting
  - `just rust-clippy` - lints
  - `just rust-test` - unit tests
- Use `just` to run builds, tests, and static analysis.
- Keep the `justfile` up to date.

## Reference

Migration documentation: `ref/migration.md` (crate structure and remaining work)

**Keep `ref/migration.md` updated**: When adding a new Rust crate, add it to the crate table.

## Build System

- Rust code: `src/nvim-rs/` workspace with per-domain crates
- CMake: `USE_RUST_*` flags in `src/nvim/CMakeLists.txt`
- `cbindgen` generates C headers from Rust
- Static library: `target/release/libnvim_rs.a`

## Key Patterns

**Opaque Handle**: Rust accesses C struct fields via accessor functions:

```c
// In C (e.g., window.c)
int nvim_win_get_width(win_T *wp) { return wp->w_width; }
```

```rust
// In Rust
extern "C" { fn nvim_win_get_width(wp: WinHandle) -> c_int; }
```

**Direct Replacement**: Replace C functions with Rust calls and delete the old C code:

```c
// Before: C implementation
int foo(int x) { return x + 1; }

// After: Call Rust, delete C implementation
int foo(int x) { return rs_foo(x); }
```

## Quick Commands

```bash
# Count migrated functions
grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l

# Find functions in a crate
grep -n "pub.*extern.*fn rs_" src/nvim-rs/<crate>/src/lib.rs

# Check USE_RUST flags
grep "USE_RUST_" src/nvim/CMakeLists.txt

# Find C accessor functions
grep -rn "^[a-z].*nvim_.*get_\|^[a-z].*nvim_.*set_" src/nvim --include="*.c"
```
