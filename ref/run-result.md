Plan: ~/.claude/plans/lazy-hopping-lemon.md

8 phases of `change.c` migration completed (agent timed out before final cleanup). All committed phases build and pass smoke tests.

**Commits (8 total):**

| Phase | Commit | Description |
|-------|--------|-------------|
| 1 | `f74762c` | FFI foundation + accessor wrappers |
| 2 | `6c1468d` | Recording functions (changed_internal, change_warning, changed) |
| 3 | `139f806` | Invalidation + redraw functions |
| 4 | `fc0de3c` | Core event functions (changed_bytes, inserted_bytes, changed_lines) |
| 5 | `f7a7aae` | Insertion primitives (ins_char_bytes, ins_char, ins_bytes_len, ins_bytes, ins_str) |
| 6 | `9c2717b` | Deletion primitives |
| 7 | `0347433` | Line operations + appended/deleted wrappers |
| 8 | `0f2e394` | Remaining deletions + open_line |

**Result:** Most change.c logic now in Rust across 8 phases. All checks pass: build, smoke-test.
