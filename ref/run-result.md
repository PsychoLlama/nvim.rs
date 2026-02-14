Plan: ~/.claude/plans/woolly-tinkering-crayon.md

5 phases of undo.c migration completed (agent timed out during Phase 6). All committed phases build and pass smoke tests.

| Phase | Commit | Description |
|-------|--------|-------------|
| 1 | `ed9a3be` | Delete dead serialization write path + legacy undo_time |
| 2 | `7069db1` | Migrate deserialization to Rust |
| 3 | `8d5edfd` | Migrate u_undoredo to Rust |
| 4 | `f30917a` | Migrate u_undo_end, u_saveline to Rust |
| 5 | `9573c27` | Migrate u_get_undo_file_name to Rust |

undo.c reduced from 3,575 → 2,682 lines (~893 lines of real C logic deleted). All checks pass: build, smoke-test.
