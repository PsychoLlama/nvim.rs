Plan: ~/.claude/plans/swirling-foraging-swan.md

All 6 phases of the mark.c migration plan are complete. Here's the summary:

## Mark Migration Summary

### Commits (6 phases)
| Phase | Commit | Functions | C Lines Deleted |
|-------|--------|-----------|-----------------|
| 1 | FFI + Memory ops | 7 | ~50 |
| 2 | Simple win/buf ops | 8 | ~80 |
| 3 | Mark get/set | 8 (2 deferred) | ~240 |
| 4 | Jumplist/changelist nav | 7 | ~250 |
| 5 | Mark adjustment core | 4 | ~240 |
| 6 | Ex commands + remaining | 3 (5 deferred) | ~120 |
| **Total** | | **37 migrated** | **~980** |

### Deferred Functions
**Phase 3:** `setmark`/`setmark_pos` — pointer comparison `pos == &curwin->w_cursor` can't be replicated in Rust FFI.

**Phase 6 (presentation-layer):** `ex_marks`, `show_one_mark`, `ex_jumps`, `ex_changes`, `mark_line` — these are display functions that would just wrap C message subsystem calls (`msg_puts_title`, `msg_putchar`, `msg_outtrans`, `snprintf` into `IObuff`) with no meaningful logic to move to Rust.

**Plan-deferred (11 functions):** VimL list iterators, ShaDa iterators, autocommand-triggering functions — heavy `list_T`/`dict_T` type dependencies.

### What Was Built
- Opaque `BufHandle`, `WinHandle`, `TabHandle` types
- ~100+ C accessor functions for struct field access
- `_Static_assert` verification for all shared constants
- 37 `#[no_mangle] pub extern "C"` Rust functions replacing C implementations
