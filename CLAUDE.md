## Guidelines

- `rust-migration` is your main branch.
- Commit your work regularly.
- Before committing, ensure these pass:
  - `just build` - full build with linking
  - `just smoke-test` - verify nvim starts without crashing
  - `just rust-fmt-check` - formatting
  - `just rust-clippy` - lints
  - `just rust-test` - unit tests
- Use `just` to run builds, tests, and static analysis.
- Keep the `justfile` up to date.

## Reference

`ref/crates.md` lists all Rust crates and their domains. Check it first to see if code belongs in an existing crate or warrants a new one.

**Maintaining `ref/crates.md`**: Add new crates to the table. Keep purpose descriptions high-level (domain, not specific functions). No other content belongs in this file.

## Build System

- Rust code: `src/nvim-rs/` workspace with per-domain crates
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

**Direct Replacement**: Replace C functions with Rust and delete the old C code:

```c
// Before: C implementation
int foo(int x) { return x + 1; }

// After: thin wrapper calling Rust
int foo(int x) { return rs_foo(x); }
```

## Regexp Migration

- Every regexp migration commit must add test cases to `src/nvim-rs/test/regexp_patterns.txt` and regenerate the corpus with `just regexp-baseline`.
- `just regexp-baseline` must pass before committing regexp changes.
- When the regexp crate exists: `just rust-test` must include regexp corpus tests.
- Fuzz targets live in `fuzz/fuzz_targets/`. Run with `just regexp-fuzz` (requires nightly).
- **Large file warning**: `src/nvim-rs/test/regexp_corpus.json` is a large generated file. Never read it directly - use `jq` via Bash to inspect or add entries (e.g., `jq length`, `jq '.[0:3]'`, `jq '.[-1]'`).

## Quick Commands

```bash
# Count migrated functions
grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l

# Find functions in a crate
grep -n "pub.*extern.*fn rs_" src/nvim-rs/<crate>/src/lib.rs

# Find C accessor functions
grep -rn "^[a-z].*nvim_.*get_\|^[a-z].*nvim_.*set_" src/nvim --include="*.c"
```
