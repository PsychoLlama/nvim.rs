Plan: ~/.claude/plans/jolly-snuggling-raven.md

All 8 phases of indent.c migration complete:

| Phase | Commit | Function | Lines Deleted |
|-------|--------|----------|---------------|
| 1 | `fdcf1fd` | set_indent() | ~250 |
| 2 | `0a09ebd` | copy_indent() | ~100 |
| 3 | `468962a` | get_breakindent_win() | ~120 |
| 4 | `8da8a5a` | ins_try_si() | ~70 |
| 5 | `d52e0d6` | op_reindent() | ~75 |
| 6 | `88b28aa` | change_indent() | ~160 |
| 7 | `f188188` | ex_retab() | ~180 |
| 8 | `476c5be` | get_lisp_indent() | ~165 |

indent.c reduced from 1,727 → 581 lines (~1,146 lines of real C logic deleted and rewritten in Rust). All checks pass: build, smoke-test.
