Plan: ~/.claude/plans/reflective-squishing-lark.md

All 5 phases of the indent_c migration plan are complete. Here's the summary:

## Completed: indent_c.c Rust Migration (5 Phases)

| Phase | Commit | Description |
|-------|--------|-------------|
| 1 | `df6be3a` | Wire up 60 existing Rust functions |
| 2 | `a4a4ce7` | Migrate pure logic helpers |
| 3 | `7a53681` | FFI infrastructure + finder functions |
| 4 | `1c3e59d` | Complex state machines |
| 5 | `19c3ea0` | parse_cino() cinoptions parser |

### Results
- **C file**: ~4140 → 2868 lines (~31% reduction)
- **Rust `#[no_mangle]` functions**: 89 total in the indent_c crate
- **All checks pass**: build, smoke-test, rust-fmt-check, rust-clippy, rust-test (3928 tests)

### What stays in C
- `get_c_indent()` (1752 lines) — massive state machine
- `in_cinkeys()` (208 lines) — complex option parsing
- `do_c_expr_indent()` / `f_cindent()` — trivial entry points
- C accessor functions for FFI
