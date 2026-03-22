## Guidelines

- `rust-migration` is your main branch.
- Commit your work regularly.
- Before committing, run `just check` (builds, smoke-tests, formatting, clippy, tests).
- Use `just` to run builds, tests, and static analysis.
- Keep the `justfile` up to date.

## Process Safety

- **Never launch nvim without `timeout -s 9`**. Use `just` recipes which have timeouts built in.
- **Always set `timeout: 300000`** (5 min) on Bash tool calls. For builds, use `timeout: 600000` (10 min).
- **No unbounded processes.** Every launched process must have a kill timeout. No exceptions.

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

## Regexp

- When changing regexp code, run `just regexp-validate` (corpus baseline check) and consider `just regexp-fuzz` (requires nightly) for edge cases.
- **Large file warning**: `src/nvim-rs/test/regexp_corpus.json` is a large generated file. Never read it directly - use `jq` via Bash to inspect or add entries.

## Quick Commands

```bash
# Count migrated functions
grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l

# Find functions in a crate
grep -n "pub.*extern.*fn rs_" src/nvim-rs/<crate>/src/lib.rs

# Find C accessor functions
grep -rn "^[a-z].*nvim_.*get_\|^[a-z].*nvim_.*set_" src/nvim --include="*.c"
```
