Plan: ~/.claude/plans/wiggly-sauteeing-waterfall.md

All 8 phases of the quickfix migration plan are complete:

| Phase | Function | Lines | Commit |
|-------|----------|-------|--------|
| 1 | `vgr_match_buflines` | ~102 | (earlier session) |
| 2 | `qf_add_entries` | ~93 | (earlier session) |
| 3 | `qf_fill_buffer` | ~119 | (earlier session) |
| 4 | `qf_init_ext` | ~107 | 78dd97cf |
| 5 | `qf_jump_edit_buffer` | ~87 | 3b2154dc |
| 6 | `vgr_process_files` | ~124 | f1060a54 |
| 7 | `ex_vimgrep` | ~87 | 07a411cc |
| 8 | `ex_helpgrep` | ~104 | d8915335 |

~811 lines of C logic migrated to Rust across 8 commits, all passing build, smoke-test, formatting, clippy, and unit tests.
