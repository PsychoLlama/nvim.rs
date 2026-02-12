Plan: ~/.claude/plans/precious-prancing-wilkes.md

All 5 phases are complete. Here's the summary:

## Migration Complete: ops.c Heavy Functions to Rust

All 5 phases committed successfully on the `rust-migration` branch:

| Commit | Phase | Function | Lines |
|--------|-------|----------|-------|
| `6e0eec2fa5` | 1 | `cursor_pos_info` | ~245 |
| `ba3f15ca42` | 2 | `do_addsub` | ~352 |
| `08417b5871` | 3 | `op_replace` | ~233 |
| `813990a8a8` | 4 | `op_delete` | ~317 |
| `3b2022654c` | 5 | `do_pending_operator` | ~638 |

**Total: ~1,785 lines of C logic migrated to Rust**

All checks pass:
- `just build` — clean (no new warnings)
- `just smoke-test` — passes
- `just rust-fmt-check` — passes
- `just rust-clippy` — passes
- `just rust-test` — 4,099 tests pass
- Targeted functional tests (operators, increment/decrement, delete, replace, fold, put, join) — all ops-related tests pass; fold_spec failures are pre-existing
