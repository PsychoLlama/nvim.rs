Plan: ~/.claude/plans/abstract-stirring-wilkinson.md

All 5 phases of the edit.c migration plan are complete. Here's a summary:

## Migration Complete

| Phase | Functions | Commit | Status |
|-------|-----------|--------|--------|
| 1: Self-Contained Helpers | 10 | `c54b41c7ef` | Done |
| 2: Replace Stack Operations | 7 | `65db8d780a` | Done |
| 3: Cursor Movement | 7 | `39ceffc636` | Done |
| 4: Insert-Mode Key Handlers | 18 | `9a93f91476` | Done |
| 5: Complex Editing | 6 | `0f3c297acf` | Done |
| **Total** | **48 functions** | | |

**Test results**: 3942 tests passing, all checks clean (build, smoke-test, fmt, clippy, rust-test).

**Key patterns used**:
- **Full Rust implementation**: Replace stack, cursor movement, navigation key handlers, `stuff_inserted`
- **Delegated C wrappers**: Complex functions with deep UI/charsize/digraph/format dependencies (`ins_eol`, `ins_ctrl_v`, `ins_copychar`, `ins_ctrl_ey`, `ins_digraph`, and the Phase 4b control keys)

**New modules created**: `helpers.rs`, `movement.rs`, `key_handlers.rs`, `editing.rs`
