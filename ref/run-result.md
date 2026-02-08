Plan: ~/.claude/plans/tingly-fluttering-rain.md

All 6 phases of the plan are complete. Here's a summary:

## Regexp Scanner/Lexer Migration — Complete

**6 commits, 13 functions migrated to Rust:**

| Phase | Commit | Functions |
|-------|--------|-----------|
| 1 | `6477908` | 25 C accessor functions for parse state globals |
| 2 | `f139f2c` | `gethexchrs`, `getdecchrs`, `getoctchrs` |
| 3 | `5101872` | `initchr`, `save_parse_state`, `restore_parse_state` |
| 4 | `043eaf8` | `peekchr`, `skipchr`, `skipchr_keepstart`, `getchr`, `ungetchr` |
| 5 | `5410100` | `read_limits` |
| 6 | `c49c304` | `cstrncmp`, `cstrchr` (+ `mb_decompose` inlined in Rust) |

**All checks pass for every commit:** build, smoke-test, rust-fmt-check, rust-clippy, rust-test (3463 tests).
