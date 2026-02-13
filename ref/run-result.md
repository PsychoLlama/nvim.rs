Plan: ~/.claude/plans/snazzy-coalescing-iverson.md

7 of 8 planned phases for `path.c` migration completed before the execute agent timed out. All 7 committed phases build and pass smoke tests.

**Commits (7 total):**

| Phase | Commit | Description |
|-------|--------|-------------|
| 1 | `180edf7` | 8 simple functions + constants |
| 2 | `81263cf` | 9 path resolution functions |
| 3 | `90e6b85` | 8 comparison/matching/misc functions |
| 4 | `930898b` | addfile, scandir_next_with_dots |
| 5 | `b369b47` | simplify_filename |
| 6 | `6a06646` | do_path_expand, path_expand |
| 7 | `51f619f` | expand_path_option, uniquefy_paths, gen_expand_wildcards, expand_wildcards |

**Result:** ~35 functions migrated across 7 phases. Most path logic now in Rust. Some remaining functions may still have C logic (agent timed out before final cleanup phase). All checks pass: build, smoke-test.
