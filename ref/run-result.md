Plan: ~/.claude/plans/jolly-mapping-shamir.md

All 7 phases of the plan are complete. Here's a summary:

## Plan Execution Complete

**6 heavy C functions migrated to Rust** from `src/nvim/normal.c`:

| Phase | Function | Lines | New Rust File |
|-------|----------|-------|---------------|
| 1A | `find_ident_at_pos` | 93 | `lib.rs` (inline) |
| 1B | `clear_showcmd` | 81 | `showcmd.rs` |
| 2A | NormalState accessor infra | ~30 accessors | (C only) |
| 2B | `normal_get_additional_char` | 159 | `additional_char.rs` |
| 3 | `normal_finish_command` | 107 | `finish_command.rs` |
| 4A | `normal_check` | 92 | `check.rs` |
| 4B | `normal_execute` | 181 | `normal_execute.rs` |

**Key patterns used:**
- Opaque handle FFI (never dereference C structs in Rust)
- `_Static_assert` for every migrated constant (~33 total)
- Labeled blocks (`'finish: { break 'finish; }`) replacing `goto`
- `CharTarget` enum replacing `int *cp` pointer-to-struct-field
- Complex C code kept in thin wrapper functions

**All 5 verification checks pass** for each commit:
- `just build`, `just smoke-test`, `just rust-fmt-check`, `just rust-clippy`, `just rust-test` (4076 tests)

**Critical fix applied:** Hallucinated MODE_* constants (MODE_REPLACE, MODE_LREPLACE, MODE_LANGMAP, MODE_NORMAL_BUSY) were all wrong — verified and corrected against `state_defs.h`.
